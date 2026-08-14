# Security Policy

Clipboard Preview handles data that may be sensitive even though it is a small local utility.

## Supported versions

Security fixes are applied to the latest released version.

## Reporting a vulnerability

Prefer GitHub **Private vulnerability reporting** from the repository Security tab when available. If that option is unavailable, open a minimal public issue asking the maintainer for a private reporting channel **without posting exploit details, clipboard contents, credentials, tokens, or other sensitive data**.

Include the affected version/platform, impact, reproduction conditions, and the smallest safe proof of concept you can provide privately.

## Security design notes

- Clipboard processing is local.
- Persistence is off by default.
- No analytics, telemetry, cloud sync, accounts, or remote clipboard API are included.
- Production code does not intentionally log clipboard contents.
- macOS Accessibility access is relevant only to the global mouse-wheel interaction used by hold/release history selection.
- Signing credentials and certificates must never be committed to the repository.
