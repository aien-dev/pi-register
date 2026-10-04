# pi-register

A register of the services on a Linux server (Debian/Ubuntu with systemd), so one command tells you what the machine runs and why.

Services are listed in `register.toml` (see the example in this folder). Fields: name, purpose, unit, category, owner, port, depends_on, files, secrets (a path only, never the secret), health (a shell command that exits 0 when healthy), boot, notes.

## Commands
- `pi-register status`: table of every service, whether its unit is active, enabled at boot, and healthy.
- `pi-register show <name>`: everything known about one service.
- `pi-register start|stop|enable|disable <name>`: wraps systemctl; unknown names are refused.
- `pi-register check`: every registered unit must exist; enabled units not in the register are listed as UNREGISTERED (base-system units are ignored via a built-in list).
- `pi-register doc`: prints a Markdown page describing the machine.

Register location: `--register <path>`, else `/etc/pi-register/register.toml`, else `./register.toml`.

Build: `cargo build --release`. No network calls. License: AGPL-3.0-or-later (see LICENSE).
