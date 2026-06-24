//! `/util download` — pull media from a URL via yt-dlp, optionally trim with
//! ffmpeg, then 2-pass encode targeting 8 MB for Discord attachment upload.
//!
//! Timeout budget: DL(180s) + PROBE(30s) + TRIM(60s) + ENCODE(240s×2) = 750s,
//! fitting within Discord's 15-minute interaction token lifetime.

use std::time::Duration;

use crate::prelude::*;
use tokio::process::Command;
use tokio::sync::Semaphore;
use url::Url;

const MAX_DURATION_SECS: f64 = 300.0;
const TARGET_SIZE_BYTES: u64 = 8 * 1024 * 1024;
const OVERHEAD_MARGIN: f64 = 0.94;
const AUDIO_BITRATE_BPS: u64 = 128 * 1024;
const VIDEO_BITRATE_FLOOR_BPS: u64 = 64_000;

const DL_TIMEOUT: Duration = Duration::from_secs(180);
const PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const TRIM_TIMEOUT: Duration = Duration::from_secs(60);
const ENCODE_TIMEOUT: Duration = Duration::from_secs(240);

/// Global serial queue for the heavy yt-dlp + ffmpeg work: one job at a time.
/// Without it, concurrent invocations stack `libx264 -preset medium` encodes
/// and saturate every CPU core on the shared bot host.
///
/// ponytail: permits=1 (strict serial). A second request waits on the permit
/// instead of running in parallel. Worst-case job is ~12.5 min, so one request
/// queued behind one job still fits Discord's 15-min interaction token. Raise
/// permits only if the host has spare cores and you accept the contention.
static DOWNLOAD_SLOT: Semaphore = Semaphore::const_new(1);

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
    // No `t.co`: it's an open URL shortener that redirects to arbitrary hosts,
    // which yt-dlp would follow — bypassing this allowlist (SSRF). Direct
    // twitter.com / x.com URLs are sufficient.
    "reddit.com",
    "redd.it",
    "vimeo.com",
];

/// Run a command to completion with a timeout. Kills the child on timeout.
async fn run(
    cmd: &mut Command,
    timeout: Duration,
    label: &str,
) -> Result<std::process::Output, String> {
    match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(Ok(o)) => Ok(o),
        Ok(Err(e)) => Err(format!("{label} failed: {e}")),
        Err(_) => Err(format!("{label} timed out after {timeout:?}")),
    }
}

/// Validate a URL is a safe, allowlisted social-media URL.
fn validate_url(raw: &str) -> Result<Url, &'static str> {
    // ponytail: no private-IP or `--`-in-path checks. The domain allowlist is
    // the real security boundary — anything not on it is rejected regardless
    // of IP class or path content. Adding more checks would be belt-and-
    // suspenders for a threat the allowlist already blocks.
    let parsed = Url::parse(raw).map_err(|_| "Could not parse as a URL.")?;

    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("Only http:// and https:// URLs are supported."),
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URLs with credentials (user:pass@) are not allowed.");
    }

    let host = parsed.host_str().ok_or("URL has no host.")?.to_lowercase();
    let allowed = ALLOWED_DOMAINS
        .iter()
        .any(|d| host == *d || host.ends_with(&format!(".{d}")));
    if !allowed {
        return Err(
            "URL host is not in the allowlist. Supported: YouTube, Facebook, Instagram, TikTok, Twitter/X, Reddit, Vimeo.",
        );
    }

    Ok(parsed)
}

/// Validate a timecode: pure digits (seconds, ≤5 digits) or HH:MM:SS / MM:SS.
fn validate_timecode(raw: &str) -> Result<(), &'static str> {
    if raw.is_empty() {
        return Err("Timecode is empty.");
    }
    if raw.chars().all(|c| c.is_ascii_digit()) {
        if raw.len() > 5 {
            return Err("Timecode in seconds is too long (max 5 digits).");
        }
        return Ok(());
    }
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

/// Probe media duration in seconds using ffprobe.
async fn probe_duration(path: &std::path::Path) -> Result<f64, Error> {
    let output = run(
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
    .await
    .map_err(Error::from)?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let trimmed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    trimmed
        .parse::<f64>()
        .map_err(|e| format!("failed to parse duration '{trimmed}': {e}").into())
}

/// Trim media to the given start/end timecodes using ffmpeg codec-copy.
async fn trim_media(
    input: &std::path::Path,
    output: &std::path::Path,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<(), Error> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-i").arg(input);
    if let Some(s) = start {
        cmd.args(["-ss", s]);
    }
    if let Some(e) = end {
        cmd.args(["-to", e]);
    }
    cmd.args(["-c", "copy"]).arg(output);

    let status = run(&mut cmd, TRIM_TIMEOUT, "ffmpeg trim")
        .await
        .map_err(Error::from)?;
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
    let total_bps = (TARGET_SIZE_BYTES as f64 * OVERHEAD_MARGIN * 8.0) / duration_secs;
    let mut video_bitrate_bps = (total_bps as u64).saturating_sub(AUDIO_BITRATE_BPS);

    if video_bitrate_bps < VIDEO_BITRATE_FLOOR_BPS {
        tracing::warn!(
            "Target 8 MB is very small for {duration_secs:.1}s — clamping video bitrate to 64 kbps."
        );
        video_bitrate_bps = VIDEO_BITRATE_FLOOR_BPS;
    }

    let null_sink = temp_dir.join("null.mp4");
    let log_prefix = temp_dir.join("ffmpeg2pass").to_string_lossy().to_string();
    let bitrate = video_bitrate_bps.to_string();

    let pass1 = run(
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input)
            .args(["-c:v", "libx264", "-b:v", &bitrate, "-preset", "medium"])
            .args(["-pass", "1", "-passlogfile", &log_prefix])
            .args(["-an", "-f", "mp4"])
            .arg(&null_sink),
        ENCODE_TIMEOUT,
        "ffmpeg pass 1",
    )
    .await
    .map_err(Error::from)?;
    if !pass1.status.success() {
        return Err("ffmpeg pass 1 failed".into());
    }
    let _ = std::fs::remove_file(&null_sink);

    let pass2 = run(
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input)
            .args(["-c:v", "libx264", "-b:v", &bitrate, "-preset", "medium"])
            .args(["-pass", "2", "-passlogfile", &log_prefix])
            .args(["-c:a", "aac", "-b:a", "128k"])
            .arg(output),
        ENCODE_TIMEOUT,
        "ffmpeg pass 2",
    )
    .await
    .map_err(Error::from)?;
    if !pass2.status.success() {
        return Err("ffmpeg pass 2 failed".into());
    }

    let _ = std::fs::remove_file(temp_dir.join("ffmpeg2pass-0.log"));
    let _ = std::fs::remove_file(temp_dir.join("ffmpeg2pass-0.log.mbtree"));
    Ok(())
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

    // Hold the global serial slot for the rest of the command. Bad input is
    // rejected above without ever queueing. The permit releases on every exit
    // path (including `?`) when `_permit` drops.
    let _permit = match DOWNLOAD_SLOT.try_acquire() {
        Ok(permit) => permit,
        Err(_) => {
            ctx.say("⏳ Another download is already processing — you're queued, this may take a few minutes...")
                .await?;
            match DOWNLOAD_SLOT.acquire().await {
                Ok(permit) => permit,
                // Unreachable in practice: the semaphore is never closed.
                Err(_) => {
                    ctx.say("Download queue is unavailable, try again later.")
                        .await?;
                    return Ok(());
                }
            }
        }
    };

    let temp_dir = tempfile::TempDir::new()?;
    let source_template = temp_dir.path().join("%(title)s.%(ext)s");

    tracing::info!("Downloading {url}");
    ctx.say("Downloading media...").await?;

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--no-playlist")
        // `--print` alone implies `--simulate` (nothing downloads), and the
        // bare `filename` field is the pre-merge name. `--no-simulate` plus
        // `after_move:filepath` actually fetches and prints the final path.
        .arg("--no-simulate")
        // Prefer mp4/m4a so the trim step's `-c copy` into .mp4 works; many
        // sources default to vp9/opus webm which won't remux into mp4.
        .args(["--extractor-args", "youtube:player_client=mweb"])
        .args(["-S", "ext:mp4:m4a"])
        .args(["--merge-output-format", "mp4"])
        .args(["--print", "after_move:filepath"])
        .arg("--output")
        .arg(source_template.to_string_lossy().to_string());

    // Optional cookies file for YouTube auth bypass.
    if let Ok(cookies) = std::env::var("YT_DLP_COOKIES_PATH")
        && !cookies.is_empty()
    {
        cmd.args(["--cookies", &cookies]);
    }

    cmd.arg(&url);

    let dl_output = match run(&mut cmd, DL_TIMEOUT, "yt-dlp download").await {
        Ok(o) => o,
        Err(e) => {
            ctx.say(format!("yt-dlp failed: {e}")).await?;
            return Ok(());
        }
    };

    if !dl_output.status.success() {
        let stderr = String::from_utf8_lossy(&dl_output.stderr);
        let error_msg = stderr
            .lines()
            .rfind(|l| l.starts_with("ERROR:"))
            .or_else(|| stderr.lines().rfind(|l| !l.is_empty()))
            .unwrap_or("unknown error");
        ctx.say(format!("yt-dlp failed: {error_msg}")).await?;
        return Ok(());
    }

    let actual_path = std::path::PathBuf::from(String::from_utf8_lossy(&dl_output.stdout).trim());

    let duration = match probe_duration(&actual_path).await {
        Ok(d) => d,
        Err(e) => {
            ctx.say(format!("Failed to probe media: {e}")).await?;
            return Ok(());
        }
    };
    if duration > MAX_DURATION_SECS {
        ctx.say(format!(
            "Source is {duration:.0}s long — exceeds the {MAX_DURATION_SECS:.0}s limit."
        ))
        .await?;
        return Ok(());
    }

    let did_trim = start.is_some() || end.is_some();
    let trimmed_path = if did_trim {
        ctx.say("Trimming...").await?;
        let trimmed = temp_dir.path().join("trimmed.mp4");
        if let Err(e) = trim_media(&actual_path, &trimmed, start.as_deref(), end.as_deref()).await {
            ctx.say(format!("Trim failed: {e}")).await?;
            return Ok(());
        }
        trimmed
    } else {
        actual_path.clone()
    };

    let encode_duration = if did_trim {
        match probe_duration(&trimmed_path).await {
            Ok(d) => d,
            Err(e) => {
                ctx.say(format!("Failed to probe trimmed media: {e}"))
                    .await?;
                return Ok(());
            }
        }
    } else {
        duration
    };

    let output_path = temp_dir.path().join("output.mp4");
    if std::fs::metadata(&trimmed_path)?.len() <= TARGET_SIZE_BYTES {
        tracing::info!("Input already under target size, skipping encode.");
        std::fs::copy(&trimmed_path, &output_path)?;
    } else {
        ctx.say("Compressing (2-pass encode)...").await?;
        if let Err(e) = two_pass_encode(
            &trimmed_path,
            &output_path,
            encode_duration,
            temp_dir.path(),
        )
        .await
        {
            ctx.say(format!("Encode failed: {e}")).await?;
            return Ok(());
        }
    }

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

#[cfg(test)]
mod tests {
    use super::{validate_timecode, validate_url};

    #[test]
    fn accepts_allowlisted_hosts() {
        for url in [
            "https://www.youtube.com/watch?v=abc",
            "https://youtu.be/abc",
            "http://x.com/user/status/1",
            "https://vimeo.com/12345",
            "https://v.redd.it/abc",
        ] {
            assert!(validate_url(url).is_ok(), "should accept {url}");
        }
    }

    #[test]
    fn rejects_t_co_shortener() {
        // t.co is an open redirector — must stay off the allowlist (SSRF guard).
        assert!(validate_url("https://t.co/abc123").is_err());
    }

    #[test]
    fn rejects_non_allowlisted_and_lookalikes() {
        for url in [
            "https://example.com/video",
            "https://notyoutube.com/x",
            "https://youtube.com.evil.com/x",
            "https://evil.com/youtube.com",
        ] {
            assert!(validate_url(url).is_err(), "should reject {url}");
        }
    }

    #[test]
    fn rejects_bad_schemes_and_credentials() {
        for url in [
            "ftp://youtube.com/x",
            "file:///etc/passwd",
            "not a url",
            "https://user:pass@youtube.com/x",
        ] {
            assert!(validate_url(url).is_err(), "should reject {url}");
        }
    }

    #[test]
    fn accepts_valid_timecodes() {
        for t in ["0", "90", "01:30", "1:30", "01:00:00", "12:34:56"] {
            assert!(validate_timecode(t).is_ok(), "should accept {t}");
        }
    }

    #[test]
    fn rejects_invalid_timecodes() {
        for t in [
            "", "123456", "1m30s", "-5", "01:", ":30", "1:2:3:4", "01:30:",
        ] {
            assert!(validate_timecode(t).is_err(), "should reject {t}");
        }
    }
}
