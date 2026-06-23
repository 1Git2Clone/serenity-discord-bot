//! `/util download` — pull media from a URL via yt-dlp, optionally trim with
//! ffmpeg, then 2-pass encode targeting 8 MB for Discord attachment upload.
//!
//! Timeout budget: DL(180s) + PROBE(30s) + TRIM(60s) + ENCODE(240s×2) = 750s,
//! fitting within Discord's 15-minute interaction token lifetime.

use std::time::Duration;

use crate::prelude::*;
use tokio::process::{Child, Command};
use url::Url;

/// Maximum source duration in seconds before we refuse to download (5 min).
const MAX_DURATION_SECS: f64 = 300.0;

/// Target file size in bytes (Discord's free-tier attachment limit).
const TARGET_SIZE_BYTES: u64 = 8 * 1024 * 1024;

/// Overhead margin for muxing headers etc. (mirrors compress_video.py).
const OVERHEAD_MARGIN: f64 = 0.94;

/// Timeout for yt-dlp download.
const DL_TIMEOUT: Duration = Duration::from_secs(180);
/// Timeout for ffprobe.
const PROBE_TIMEOUT: Duration = Duration::from_secs(30);
/// Timeout for ffmpeg trim.
const TRIM_TIMEOUT: Duration = Duration::from_secs(60);
/// Timeout for each ffmpeg 2-pass encode.
const ENCODE_TIMEOUT: Duration = Duration::from_secs(240);

/// Domains accepted by `/util download` — yt-dlp-supported social media platforms.
const ALLOWED_DOMAINS: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "facebook.com",
    "fb.watch",
    "fb.com",
    "instagram.com",
    "tiktok.com",
    "twitter.com",
    "x.com",
    "t.co",
    "reddit.com",
    "redd.it",
    "vimeo.com",
];

/// Validate a URL is a safe, allowlisted social-media URL.
///
/// Checks: parseable, http/https scheme, no userinfo, no private IP, domain
/// allowlist, no flag-like `--` content in path or query.
fn validate_url(raw: &str) -> Result<Url, &'static str> {
    if raw.starts_with('-') {
        return Err("URL starts with `-` which looks like a flag — rejected.");
    }
    let parsed = Url::parse(raw).map_err(|_| "Could not parse as a URL.")?;

    // Scheme check
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("Only http:// and https:// URLs are supported."),
    }

    // No credentials in URL
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URLs with credentials (user:pass@) are not allowed.");
    }

    // Host check
    let host = parsed.host_str().ok_or("URL has no host.")?;
    let host_lower = host.to_lowercase();

    // Reject IP addresses that are private/loopback/link-local
    if let Some(ip) = parsed.host() {
        match ip {
            url::Host::Ipv4(addr) => {
                if is_private_ipv4(addr) {
                    return Err("URL points to a private/internal IP address.");
                }
            }
            url::Host::Ipv6(addr) => {
                if is_private_ipv6(addr) {
                    return Err("URL points to a private/internal IP address.");
                }
            }
            url::Host::Domain(_) => {}
        }
    }

    // Domain allowlist
    let allowed = ALLOWED_DOMAINS
        .iter()
        .any(|domain| host_lower == *domain || host_lower.ends_with(&format!(".{domain}")));
    if !allowed {
        return Err(
            "URL host is not in the allowlist. Supported: YouTube, Facebook, Instagram, TikTok, Twitter/X, Reddit, Vimeo.",
        );
    }

    // Reject flag-like content in path/query
    for segment in parsed.path_segments().into_iter().flatten() {
        if segment.starts_with("--") {
            return Err("URL path contains flag-like content (`--`).");
        }
    }
    for (key, _) in parsed.query_pairs() {
        if key.starts_with("--") {
            return Err("URL query contains flag-like content (`--`).");
        }
    }

    Ok(parsed)
}

fn is_private_ipv4(addr: std::net::Ipv4Addr) -> bool {
    addr.is_loopback()       // 127.0.0.0/8
        || addr.is_private() // 10/8, 172.16/12, 192.168/16
        || addr.is_link_local()  // 169.254/16
        || addr.is_unspecified() // 0.0.0.0
        || addr.is_broadcast() // 255.255.255.255
}

fn is_private_ipv6(addr: std::net::Ipv6Addr) -> bool {
    addr.is_loopback()         // ::1
        || addr.is_unspecified() // ::
        // fc00::/7 (unique local) and fe80::/10 (link-local)
        || (addr.segments()[0] & 0xfe00) == 0xfc00
        || (addr.segments()[0] & 0xffc0) == 0xfe80
}

/// Validate a timecode string: either pure seconds (1-5 digits) or HH:MM:SS / MM:SS.
fn validate_timecode(raw: &str) -> Result<(), &'static str> {
    if raw.is_empty() {
        return Err("Timecode is empty.");
    }
    if raw.starts_with('-') {
        return Err("Timecode starts with `-` — rejected.");
    }
    if raw.contains(' ') || raw.contains('\t') {
        return Err("Timecode must not contain spaces.");
    }
    // Check for shell metacharacters
    if raw.contains(|c: char| c.is_ascii_control() || "|;&$`'\"\\!@#%^*()[]{}<>?".contains(c)) {
        return Err("Timecode contains invalid characters.");
    }
    // Pure digits: seconds
    if raw.chars().all(|c| c.is_ascii_digit()) {
        if raw.len() > 5 {
            return Err("Timecode in seconds is too long (max 5 digits = ~27 hours).");
        }
        return Ok(());
    }
    // HH:MM:SS or MM:SS
    let parts: Vec<&str> = raw.split(':').collect();
    if (parts.len() == 2 || parts.len() == 3)
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        return Ok(());
    }
    Err("Timecode must be seconds (e.g. 90), MM:SS (e.g. 01:30), or HH:MM:SS (e.g. 01:00:00).")
}

/// Kill a child process if it's still running, ignoring errors.
async fn kill_child(child: &mut Child) {
    if let Ok(Some(_)) = child.try_wait() {
        return; // already exited
    }
    let _ = child.kill().await;
    let _ = child.wait().await;
}

/// Run a command to completion with a timeout. Kills the child on timeout.
async fn run_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
    label: &str,
) -> Result<std::process::Output, String> {
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{label} failed to spawn: {e}"))?;

    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(status)) => {
            if status.success() {
                Ok(std::process::Output {
                    status,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            } else {
                Err(format!("{label} failed with exit code {:?}", status.code()))
            }
        }
        Ok(Err(e)) => Err(format!("{label} process error: {e}")),
        Err(_) => {
            kill_child(&mut child).await;
            Err(format!("{label} timed out after {timeout:?}."))
        }
    }
}

/// Run a command and capture its output, with a timeout.
///
/// Uses `Command::output()` which spawns and waits internally. On timeout the
/// child may be orphaned (OS will reap it). For kill-on-timeout see
/// `run_with_timeout`.
async fn output_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
    label: &str,
) -> Result<std::process::Output, String> {
    tokio::time::timeout(timeout, cmd.output())
        .await
        .map_err(|_| format!("{label} timed out after {timeout:?}."))?
        .map_err(|e| format!("{label} failed: {e}"))
}

/// Download media from a URL, optionally trim, and upload as a compressed attachment.
#[poise::command(discard_spare_arguments, slash_command, prefix_command)]
#[tracing::instrument(
    skip(ctx),
    fields(
        category = "discord_command",
        command.name = %ctx.command().name,
        author = %ctx.author().id,
        guild_id = %ctx.guild_id().map(GuildId::get).unwrap_or(0),
    )
)]
pub async fn download(
    ctx: Context<'_>,
    #[description = "Media URL (YouTube, etc.)"] url: String,
    #[description = "Start timecode (e.g. 00:01:30 or 90)"] start: Option<String>,
    #[description = "End timecode (e.g. 00:02:00 or 120)"] end: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Validate inputs before doing any work.
    if let Err(msg) = validate_url(&url) {
        ctx.say(format!("Invalid URL: {msg}")).await?;
        return Ok(());
    }
    if let Some(ref s) = start
        && let Err(msg) = validate_timecode(s)
    {
        ctx.say(format!("Invalid start timecode: {msg}")).await?;
        return Ok(());
    }
    if let Some(ref e) = end
        && let Err(msg) = validate_timecode(e)
    {
        ctx.say(format!("Invalid end timecode: {msg}")).await?;
        return Ok(());
    }

    let temp_dir = tempfile::TempDir::new()?;
    let source_template = temp_dir.path().join("%(title)s.%(ext)s");

    // Step 1: download with yt-dlp CLI
    tracing::info!("Downloading {url}");
    ctx.say("Downloading media...").await?;

    let dl_output = match output_with_timeout(
        Command::new("yt-dlp")
            .arg("--no-playlist")
            .arg("--print")
            .arg("filename")
            .arg("--output")
            .arg(source_template.to_string_lossy().to_string())
            .arg(&url),
        DL_TIMEOUT,
        "yt-dlp download",
    )
    .await
    {
        Ok(o) => o,
        Err(e) => {
            let msg = format!("Download failed: {e}");
            ctx.say(&msg).await?;
            return Err(e.into());
        }
    };

    if !dl_output.status.success() {
        let stderr = String::from_utf8_lossy(&dl_output.stderr);
        ctx.say(format!("yt-dlp failed: {stderr}")).await?;
        return Ok(());
    }

    let actual_path = std::path::PathBuf::from(String::from_utf8_lossy(&dl_output.stdout).trim());

    // Step 2: probe duration
    let duration = match probe_duration(&actual_path).await {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("Failed to probe media duration: {e}");
            ctx.say(&msg).await?;
            return Err(e);
        }
    };
    if duration > MAX_DURATION_SECS {
        let msg = format!(
            "Source is {:.0}s long — exceeds the {:.0}s limit. Pick a shorter clip.",
            duration, MAX_DURATION_SECS
        );
        ctx.say(msg).await?;
        return Ok(());
    }

    // Step 3: trim if start/end given
    let did_trim = start.is_some() || end.is_some();
    let trimmed_path = if did_trim {
        ctx.say("Trimming...").await?;
        let trimmed = temp_dir.path().join("trimmed.mp4");
        match trim_media(&actual_path, &trimmed, start.as_deref(), end.as_deref()).await {
            Ok(()) => trimmed,
            Err(e) => {
                ctx.say("Trimming failed.").await?;
                return Err(e);
            }
        }
    } else {
        actual_path.clone()
    };

    // Step 4: probe trimmed duration for accurate bitrate calculation
    let encode_duration = if did_trim {
        match probe_duration(&trimmed_path).await {
            Ok(d) => d,
            Err(e) => {
                ctx.say("Failed to probe trimmed clip duration.").await?;
                return Err(e);
            }
        }
    } else {
        duration
    };

    // Step 5: skip encode if already under target size
    let output_path = temp_dir.path().join("output.mp4");
    let input_size = std::fs::metadata(&trimmed_path)?.len();
    if input_size <= TARGET_SIZE_BYTES {
        tracing::info!(
            "Input is already {:.1} MB (under {:.1} MB target). Skipping encode.",
            input_size as f64 / (1024.0 * 1024.0),
            TARGET_SIZE_BYTES as f64 / (1024.0 * 1024.0),
        );
        std::fs::copy(&trimmed_path, &output_path)?;
    } else {
        // Step 6: 2-pass encode targeting 8 MB
        ctx.say("Compressing (2-pass encode)...").await?;
        if let Err(e) = two_pass_encode(
            &trimmed_path,
            &output_path,
            encode_duration,
            temp_dir.path(),
        )
        .await
        {
            ctx.say("Compression failed.").await?;
            return Err(e);
        }
    }

    // Step 7: upload
    let file_size = std::fs::metadata(&output_path)?.len();
    tracing::info!("Uploading {file_size} bytes");

    let attachment = serenity::CreateAttachment::path(&output_path).await?;
    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "Here's your clip ({:.1} MB):",
                file_size as f64 / (1024.0 * 1024.0)
            ))
            .attachment(attachment),
    )
    .await?;

    Ok(())
}

/// Probe media duration in seconds using ffprobe.
async fn probe_duration(path: &std::path::Path) -> Result<f64, Error> {
    let output = output_with_timeout(
        Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "csv=p=0",
            ])
            .arg(path),
        PROBE_TIMEOUT,
        "ffprobe",
    )
    .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe failed: {stderr}").into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    trimmed
        .parse::<f64>()
        .map_err(|e| format!("failed to parse duration '{trimmed}': {e}").into())
}

/// Trim media to the given start/end timecodes using ffmpeg.
async fn trim_media(
    input: &std::path::Path,
    output: &std::path::Path,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<(), Error> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");
    cmd.arg("-i");
    cmd.arg(input);
    if let Some(s) = start {
        cmd.args(["-ss", s]);
    }
    if let Some(e) = end {
        cmd.args(["-to", e]);
    }
    // Copy codecs for fast trim (no re-encode on the trim step).
    cmd.args(["-c", "copy"]);
    cmd.arg(output);

    let status = run_with_timeout(&mut cmd, TRIM_TIMEOUT, "ffmpeg trim").await?;
    if !status.status.success() {
        return Err("ffmpeg trim failed".into());
    }
    Ok(())
}

/// 2-pass encode targeting TARGET_SIZE_BYTES.
///
/// Mirrors `compress_video.py` from
/// <https://github.com/1Git2Clone/dotfiles/blob/main/dot-config/programs/py_scripts/compress_video.py>.
async fn two_pass_encode(
    input: &std::path::Path,
    output: &std::path::Path,
    duration_secs: f64,
    temp_dir: &std::path::Path,
) -> Result<(), Error> {
    // Total bitrate budget, then subtract 128 kbps for audio.
    let total_bps = (TARGET_SIZE_BYTES as f64 * OVERHEAD_MARGIN * 8.0) / duration_secs;
    let audio_bitrate_bps = 128 * 1024;
    let mut video_bitrate_bps = (total_bps as u64).saturating_sub(audio_bitrate_bps);

    // Sanity floor: prevent negative or glitchy low bitrates (mirrors compress_video.py).
    if video_bitrate_bps < 64_000 {
        tracing::warn!(
            "Target 8 MB is very small for {duration_secs:.1}s — clamping video bitrate to 64 kbps."
        );
        video_bitrate_bps = 64_000;
    }

    let null_sink = temp_dir.join("null.mp4");
    let log_prefix = temp_dir.join("ffmpeg2pass").to_string_lossy().to_string();

    // Pass 1: analyze, no audio, no output file.
    let pass1 = run_with_timeout(
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input)
            .args(["-c:v", "libx264"])
            .args(["-b:v", &video_bitrate_bps.to_string()])
            .args(["-preset", "medium"])
            .args(["-pass", "1"])
            .args(["-passlogfile", &log_prefix])
            .args(["-an", "-f", "mp4"])
            .arg(&null_sink),
        ENCODE_TIMEOUT,
        "ffmpeg pass 1",
    )
    .await?;

    if !pass1.status.success() {
        return Err("ffmpeg pass 1 failed".into());
    }

    // Clean up null sink from pass 1.
    let _ = std::fs::remove_file(&null_sink);

    // Pass 2: encode with audio.
    let pass2 = run_with_timeout(
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input)
            .args(["-c:v", "libx264"])
            .args(["-b:v", &video_bitrate_bps.to_string()])
            .args(["-preset", "medium"])
            .args(["-pass", "2"])
            .args(["-passlogfile", &log_prefix])
            .args(["-c:a", "aac"])
            .args(["-b:a", "128k"])
            .arg(output),
        ENCODE_TIMEOUT,
        "ffmpeg pass 2",
    )
    .await?;

    if !pass2.status.success() {
        return Err("ffmpeg pass 2 failed".into());
    }

    // Clean up ffmpeg2pass log files from temp_dir.
    let _ = std::fs::remove_file(temp_dir.join("ffmpeg2pass-0.log"));
    let _ = std::fs::remove_file(temp_dir.join("ffmpeg2pass-0.log.mbtree"));

    Ok(())
}
