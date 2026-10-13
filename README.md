# pi-register

A register of the services on a Linux server (Debian/Ubuntu with systemd), so one command tells you what the machine runs and why.

Services are listed in `register.toml`. The one in this folder is the Pi hub's own register and doubles as the example. Fields: name, purpose, unit, category, owner, port, depends_on, files, secrets (a path only, never the secret), health (a shell command that exits 0 when healthy), boot, notes. Required: name, purpose, unit, category, owner, boot. The rest are optional. A `unit` without a suffix means `<unit>.service`; write `.timer`, `.socket` and so on in full. `depends_on` and `files` are recorded and shown, not enforced.

## Commands
- `pi-register status`: table of every service, whether its unit is active, enabled at boot, and healthy.
- `pi-register show <name>`: everything known about one service.
- `pi-register start|stop|enable|disable <name>`: wraps systemctl; unknown names are refused.
- `pi-register check`: every registered unit must exist; enabled service units not in the register are listed as UNREGISTERED (base-system units are ignored via a built-in list; timers and other unit types are not scanned for unregistered ones). Exits with an error if any problem is found.
- `pi-register doc`: prints a Markdown page describing the machine.

Register location: `--register <path>`, else `/etc/pi-register/register.toml`, else `./register.toml`.

Build: `cargo build --release`. `pi-register` itself makes no network calls; the `health` commands in the register run in a shell and may. License: AGPL-3.0-or-later (see LICENSE).
