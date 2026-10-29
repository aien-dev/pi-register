# Security policy

## Reporting a vulnerability

Report privately. Do not open a public issue, pull request or discussion about it.

1. GitHub: open this repository's Security tab and choose "Report a vulnerability". This
   creates a private advisory that only the maintainers can see. If that option is not shown,
   private vulnerability reporting has not been enabled for this repository yet; use email.
2. Email: drake@aienos.com with the subject `[pi-register] security report`.

Include the commit or release you tested, the component (path), steps to reproduce, and what an
attacker gains. If the report involves a secret, say where it was exposed and do not send the
secret itself. Encrypted mail is not required; do not post proof-of-concept payloads publicly.

## Response and disclosure

Reports receive best-effort review. No response deadline, fix deadline or embargo duration
is promised here. Agree on disclosure and attribution with the maintainer through the
private reporting channel before publishing sensitive details.

## Scope

`pi-register` is a Rust CLI (`src/main.rs`) that reads a TOML service register and compares it
with what systemd reports.

In scope:

- Register parsing (`toml` + `serde`) that accepts a malformed file silently, or a duplicate-name
  check that can be bypassed.
- A way for `start`, `stop`, `enable` or `disable` to act on a unit that is not in the register, or
  for a register value to be interpreted by `systemctl` as an option rather than a unit name.
- Register lookup that picks up a file the operator did not intend (the fallback order is
  `--register <path>`, then `/etc/pi-register/register.toml`, then `./register.toml`). `status`
  and `show` run every `health` value through `sh -c` with the caller's privileges, so a register
  is code.
- A network call made by `pi-register` itself (AGENTS.md says it makes none).
- Secret values, private addresses, internal hosts or real inventories in tracked files.

Out of scope:

- The behaviour of a `health` command written by the operator, systemd, and the host operating
  system.

## Supported versions

Only the default branch `main`; there is no release (Cargo version `0.1.0`). Reports against older commits are welcome but fixes land on the default branch only.

## Status

An operations tool that supports AienOS; it is not part of AienOS itself (AGENTS.md). There is no
CI workflow in this repository; local build and test results are the only checks. No
certification, no bug bounty. See AUDIT_SCOPE.md for what to review and how to build and test it.
