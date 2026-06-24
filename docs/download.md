# `/util download` — YouTube auth

## The problem

YouTube blocks downloads from datacenter IPs (Hetzner, etc.) with:

```
ERROR: [youtube] ...: Sign in to confirm you're not a bot.
```

The bot runs on a Hetzner VPS. Google treats its IP range as bot space.

## The fix: cookies

Export cookies from a browser logged into YouTube on a residential IP, point
the bot at them via `YT_DLP_COOKIES_PATH`. yt-dlp uses the cookie session for
auth, and the data transfer happens from the server.

### How to export

1. Open a **private/incognito** browser window
2. Log into YouTube (or create a throwaway Google account)
3. In the same tab, navigate to `https://www.youtube.com/robots.txt`
4. Install a cookie export extension like
   [Get cookies.txt LOCALLY](https://chrome.google.com/webstore/detail/get-cookiestxt-locally/cclelndahbckbenkjhflpdbgdldlbecc)
   (Chrome) or the Firefox equivalent
5. Export `youtube.com` cookies to a file
6. **Close the incognito window** — never open that session again
7. `scp` the file to the server
8. Set `YT_DLP_COOKIES_PATH=./cookies.txt` in the bot's `.env`
9. Restart the bot

### Why this works

YouTube rotates cookies aggressively on open browser tabs. By exporting from
an incognito session and never reopening it, the cookies are frozen — yt-dlp
using them looks like API traffic, not a browser, so they don't get rotated.
They typically last months.

### When they expire

If the bot starts getting the "Sign in" error again, re-export fresh cookies
and scp them up. The session is still valid — you just need to re-export.

## Alternatives considered

| Approach | Why not |
|----------|---------|
| **PO tokens** (bgutil-ytdlp-pot-provider) | YouTube binds PO tokens to video IDs now — need a new one per download. The provider daemon needs Node.js + maintenance. Brittle. |
| **Cobalt** (self-hosted) | Same Hetzner IP problem — YouTube blocks the outbound IP regardless of the client software. Would need a residential proxy anyway. |
| **Residential proxy** | Works, but adds $3-5/mo and another moving part. Cookies are free and simpler for a single-user bot. |
| **Tailscale exit node** | The work-laptop exit node is offline and routing bot traffic through someone's work machine isn't great. |
