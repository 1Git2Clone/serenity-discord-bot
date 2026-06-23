# Security Policy

## Reporting a vulnerability

Report suspected vulnerabilities privately rather than opening a public issue.
Use GitHub's [private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
on this repository (Security → Report a vulnerability), or contact the
maintainer directly.

<!-- Replace with a real private contact if one is established. -->

Please include the affected version or commit, reproduction steps, and the
impact you observed. We aim to acknowledge reports and will coordinate a fix
and disclosure timeline with you.

## Secrets inventory

The bot handles two sensitive credentials. None are ever logged.

| Secret | Where it lives | Notes |
|---|---|---|
| `BOT_TOKEN` | `.env` / environment | Discord bot token. |
| `AI_API_KEY` | `.env` / environment | AI provider key (hosted backends). |

`.env` is gitignored.
