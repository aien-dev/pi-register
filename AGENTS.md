# pi-register (a register of the services on a Linux server)

> `pi-register` is a small Rust command-line tool that reads a hand-kept list of the services a Debian/Ubuntu systemd machine runs and why, and compares it with what systemd reports. The tracked `register.toml` is a generic example only. It is an operations tool that supports AienOS, the whole AIEN system; Aienux is the kernel. The tool is not part of AienOS itself.
Shared rules: read https://www.aienos.com/agents/RULES.md first. This file adds what is specific to this repository; where the two differ, this file wins here.

## Scope

- Applies to the whole repository: `src/main.rs` (the whole program, unit tests included), `register.toml`, `Cargo.toml`, `Cargo.lock`.
- Keep `register.toml` generic. Real inventories belong in ignored `register.local.toml` or `/etc/pi-register/register.toml`, never tracked files.
- `.crumb` is compiled crumb metadata. Do not edit it by hand. `.crumb.local` is local-only and ignored by git.
- There is no CI workflow in this repository. Local build and test results are the only checks.

## Installation

```bash
git clone https://github.com/aien-dev/pi-register.git
cd pi-register
cargo build --release
```

Needs a stable Rust toolchain. Dependencies are `clap`, `serde` and `toml`. The binary is `target/release/pi-register`.

## Configuration

- Register location: `--register <path>`, else `/etc/pi-register/register.toml`, else `./register.toml`.
- The ignored `register.local.toml` requires an explicit `--register register.local.toml`; it is not an automatic fallback. Preserve an existing deployment's full register privately before updating, then migrate checkout-path callers or install it at the existing `/etc` default. See README for the migration procedure.
- Each service is a `[[service]]` table. Required fields: `name`, `purpose`, `unit`, `category`, `owner`, `boot`. Optional: `port`, `depends_on`, `files`, `secrets`, `health`, `notes`.
- A `unit` without a suffix means `<unit>.service`; write `.timer`, `.socket` and other suffixes in full.
- `secrets` holds a path only, never a secret value.
- `health` is a shell command that exits 0 when the service is healthy. It runs through `sh -c` with the caller's privileges.
- `depends_on` and `files` are recorded and shown, not enforced. A name that appears twice is a load error.

## Usage

```bash
# Unit tests (no systemd access needed)
cargo test

# Help for every command
./target/release/pi-register --help

# Markdown page describing the machine; reads the register only
./target/release/pi-register --register register.toml doc
```

Commands that touch the live machine. Run them only on the server the register describes, never on a development host:

- `pi-register status`: every service with its active state, boot state and health. Calls `systemctl` and runs every `health` command.
- `pi-register show <name>`: everything known about one service, plus its live state and health.
- `pi-register check`: every registered unit must exist; enabled service units missing from the register are listed as UNREGISTERED. Exits non-zero on any problem.
- `pi-register start|stop|enable|disable <name>`: wraps `systemctl`; names not in the register are refused. These change the machine and usually need root.

## Conventions

- One source file, `src/main.rs`. Tests live in its `#[cfg(test)]` module; add one when a parsing or validation rule can regress silently.
- `pi-register` itself makes no network calls. Keep it that way; network access belongs in a service's `health` command.
- Base-system units that `check` ignores come from a built-in list in `src/main.rs`. Extend that list rather than registering base-system units.
- Keep `README.md` in step with the commands and fields the code accepts.

## Safety and approvals

- Never run `status`, `show`, `check`, `start`, `stop`, `enable` or `disable` against the example register on a machine it does not describe: `status` and `show` execute the register's `health` commands, and the control commands change systemd state.
- Ask before running `start`, `stop`, `enable` or `disable` on the real server, and before changing any `boot` value. Some entries are deliberately left stopped or disabled; their `notes` say why.
- Never put secret values, tokens, keys, private network addresses, internal hosts, personal home paths or deployment inventories in tracked files. Keep names and attribution intact.
- Work on a branch and merge through a pull request with the `cargo test` result in its body.

## Further reading

- `README.md`: field reference and command list.
- `register.toml`: a generic worked example, never the live inventory.
- https://github.com/aien-dev/aienix: the AienOS system tree this tool supports.
