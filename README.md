# pi-register

A register of the services on a Linux server (Debian/Ubuntu with systemd), so one command tells you what the machine runs and why.

Supports [AienOS](https://github.com/aien-dev/aienix), the whole AIEN system. Aienux is its kernel. This is an operations tool, not part of the system itself.

The checked-in `register.toml` is a generic Debian/Ubuntu example, not an inventory of a deployed machine.

Fields: name, purpose, unit, category, owner, port, depends_on, files, secrets (a path only, never the secret), health (a shell command that exits 0 when healthy), boot, notes. Required: name, purpose, unit, category, owner, boot. The rest are optional. A `unit` without a suffix means `<unit>.service`; write `.timer`, `.socket` and so on in full. `depends_on` and `files` are recorded and shown, not enforced.

## Commands
- `pi-register status`: table of every service, whether its unit is active, enabled at boot, and healthy.
- `pi-register show <name>`: everything known about one service.
- `pi-register start|stop|enable|disable <name>`: wraps systemctl; unknown names are refused.
- `pi-register check`: every registered unit must exist; enabled service units not in the register are listed as UNREGISTERED (base-system units are ignored via a built-in list; timers and other unit types are not scanned for unregistered ones). Exits with an error if any problem is found.
- `pi-register doc`: prints a Markdown page describing the machine.

Register location: `--register <path>`, else `/etc/pi-register/register.toml`, else `./register.toml`.

## Private deployment configuration

For a new machine, copy the example to the ignored `register.local.toml`, restrict it to mode 600, and edit it to describe that machine. Invoke it explicitly:

```sh
install -m 600 register.toml register.local.toml
pi-register --register register.local.toml doc
```

`register.local.toml` is not an automatic lookup location. Use `--register register.local.toml` in local scripts and operator commands. For an installed deployment, keep the real register at `/etc/pi-register/register.toml` (mode 600, owned by the intended operator); the existing default lookup then needs no command changes.

**Existing deployments:** before updating a checkout, preserve the complete real register outside git with mode 600. Restore it as `register.local.toml` and migrate any caller that explicitly names the checkout's `register.toml` to `--register register.local.toml`, or install the preserved register at `/etc/pi-register/register.toml` and remove that explicit flag. Do not replace a deployed register with the public example. Preserve all health commands, boot values, paths and targets in the private copy; inspect `doc` output privately to confirm it loads. `doc` reads configuration only and does not execute health commands or call systemd.

Never commit local service inventories, home paths, internal hosts, network targets or secrets. The public example intentionally has no health command; adapt it before operating services. This migration changes the example only, not CLI lookup semantics.

Build: `cargo build --release`. `pi-register` itself makes no network calls; the `health` commands in the register run in a shell and may. License: AGPL-3.0-or-later (see LICENSE).
