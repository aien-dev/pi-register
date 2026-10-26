# Audit scope

Written for an outside, independent security audit. Revision described:
`4935b8a03ca93b4dddaf83bb9a99784dfdffcb3b` on `main`, 2026-10-26. Everything below is taken from the repository's own files at
that revision; items marked NOT_RUN were not executed while writing this document.

## What this repository is

A small Rust command-line tool that reads a hand-kept list of the services a Debian or Ubuntu
systemd machine runs and why (`register.toml`), and compares it with what systemd reports. The
whole program and its unit tests are in `src/main.rs` (391 lines). The tracked `register.toml` is a
generic example with one `ssh` entry, not an inventory of a deployed machine. It is an operations
tool, not part of AienOS. Licence AGPL-3.0-or-later.

## What to review, in priority order

1. `health` execution (`fn health`, `src/main.rs` around line 170). Each service's `health` string
   runs as `sh -c <cmd>` with the caller's privileges, with no timeout, during `status` and
   `show`. Check that no other command runs health strings (README says `doc` does not), and that
   the operator is warned the register file must be as trusted as a script.
2. Register lookup (`fn find_register`). `--register <path>`, else
   `/etc/pi-register/register.toml`, else `./register.toml` in the current directory. Check the
   consequence of running `status` or `show` from an untrusted working directory with no `/etc`
   register present.
3. systemctl wrapper (`fn systemctl`, `fn cmd_control`, `fn cmd_check`). Unit names from the
   register are passed as separate arguments to `systemctl` (no shell). There is no `--` separator
   before the unit; check whether a register `unit` value beginning with `-` is parsed as an option.
   `start|stop|enable|disable` refuse names not in the register (`unknown_name_is_refused` test).
4. Parsing and validation (`fn parse_register`, `fn load`). Required fields `name`, `purpose`,
   `unit`, `category`, `owner`, `boot`; duplicate names are a load error. `secrets` holds a path
   only.
5. `fn full_unit` and the built-in ignore list (`IGNORE`, `fn is_ignored`, `fn pattern_matches`)
   used by `check` to skip base-system units. Check an attacker-chosen unit name cannot hide behind
   the ignore patterns.
6. `restore_sigpipe` uses an `extern` declaration of `signal`; check it for soundness.

There is no `.github/workflows/` directory. AGENTS.md: "There is no CI workflow in this repository.
Local build and test results are the only checks."

## Trust boundaries

- Trusted: the register file (it contains shell commands) and the operator who runs the tool;
  `systemctl` on the target machine.
- Untrusted: none at run time beyond the register, if its file permissions or location are not
  controlled. README recommends mode 600 for `register.local.toml` and
  `/etc/pi-register/register.toml`.
- Privilege: control commands usually need root (AGENTS.md), so the register's trust level is the
  caller's privilege level.

## Threat model summary

- Assets: the systemd state of the machine the register describes; the privacy of real
  inventories.
- Attackers considered: someone who can write a register file the tool will load; accidental
  publication of a real inventory.
- Defended: unknown names refused by control commands; real inventories kept in ignored
  `register.local.toml` or `/etc`; no network calls by the tool.
- Not defended / explicitly out of scope: a hostile register is equivalent to running its `health`
  commands; `depends_on` and `files` are recorded and shown, not enforced; timers and other unit
  types are not scanned for unregistered ones (README).

## Build from source at a pin

Needs a stable Rust toolchain (no `rust-toolchain.toml`; the exact version is not pinned).
Dependencies `clap` 4.6.7, `serde` 1.0.229, `toml` 1.1.6, locked by `Cargo.lock`.

```sh
cargo build --release
```

The binary is `target/release/pi-register`.

## Run every test

NOT_RUN while writing this document.

```sh
cargo test                                                  # unit tests in src/main.rs, no systemd access
./target/release/pi-register --help                         # help for every command
./target/release/pi-register --register register.toml doc   # reads the register only
```

The five unit tests are `parses_full_and_minimal_entries`, `rejects_missing_required_field`,
`ignore_list_filters_base_units_only`, `unit_names_get_suffix` and `unknown_name_is_refused`.
Expected: `test result: ok. 5 passed` (count taken from the `#[test]` attributes).

`status`, `show`, `check`, `start`, `stop`, `enable`, `disable`: NOT_RUN on a plain host. AGENTS.md:
run them only on the server the register describes, never on a development host, because `status`
and `show` execute `health` commands and the control commands change systemd state.

## Known limitations and NOT_RUN

- No CI; local results only.
- `health` commands have no timeout in the code.
- No tests cover `health` execution or the systemctl wrappers (they need a live systemd).
- Rust toolchain version is not pinned.

## Dependencies and provenance

- `Cargo.lock` present (34 packages including transitive `windows-sys` and `winnow`).
- Fetched from crates.io at build time.
- No vendored code, no GitHub Actions.
- At run time the tool calls `systemctl` and `sh`; it makes no network calls itself.

## Evidence

There is no evidence directory. AGENTS.md asks that pull requests carry the `cargo test` result in
their body. No receipts are kept in this repository.
