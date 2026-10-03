//! pi-register: a register of the services on a Linux server (systemd).

use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const DEFAULT_REGISTER: &str = "/etc/pi-register/register.toml";

#[derive(Debug, Deserialize, Clone)]
struct Service {
    name: String,
    purpose: String,
    unit: String,
    category: String,
    owner: String,
    port: Option<u16>,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    secrets: Option<String>,
    health: Option<String>,
    boot: bool,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Register {
    #[serde(default, rename = "service")]
    services: Vec<Service>,
}

#[derive(Parser)]
#[command(name = "pi-register", about = "What this machine runs, and why")]
struct Cli {
    /// Path to the register file (default: /etc/pi-register/register.toml, then ./register.toml)
    #[arg(long, global = true)]
    register: Option<PathBuf>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Table of every registered service and its live state
    Status,
    /// Everything the register knows about one service
    Show { name: String },
    /// Start a service
    Start { name: String },
    /// Stop a service
    Stop { name: String },
    /// Turn on start-at-boot for a service
    Enable { name: String },
    /// Turn off start-at-boot for a service
    Disable { name: String },
    /// Compare the register with the machine
    Check,
    /// Print a Markdown page describing the machine
    Doc,
}

/// Base-system units that are expected on any Debian/Ubuntu server and are
/// not worth registering. Entries ending in '*' match by prefix.
const IGNORE: &[&str] = &[
    "systemd-*", "getty@*", "serial-getty@*", "dbus*", "ssh.service", "sshd.service",
    "cron.service", "rsyslog.service", "networkd-dispatcher.service", "polkit.service",
    "apparmor.service", "console-setup.service", "keyboard-setup.service", "e2scrub*",
    "fstrim.*", "logrotate.*", "apt-*", "apt-daily*", "unattended-upgrades.service",
    "ufw.service", "snapd*", "multipathd*", "irqbalance.service", "lvm2*", "blk-availability.service",
    "cloud-*", "open-iscsi.service", "iscsid*", "rsync.service", "ModemManager.service",
    "NetworkManager*", "wpa_supplicant.service", "user@*", "user-runtime-dir@*", "udev*",
    "setvtrgb.service", "secureboot-db.service", "ubuntu-advantage.service", "ua-*",
    "pollinate.service", "plymouth*", "grub-*", "gpu-manager.service", "hwclock.service",
    "motd-news.*", "man-db.*", "dpkg-db-backup.*", "e2scrub_*", "finalrd.service",
    "lxd-*", "lxcfs.service", "sysstat.service", "rpcbind.*", "nfs-*", "avahi-daemon.*",
    "bluetooth.service", "cups*", "accounts-daemon.service", "udisks2.service",
    "thermald.service", "upower.service", "power-profiles-daemon.service",
    "switcheroo-control.service", "rtkit-daemon.service", "pi-register*",
    "apport.service", "chrony.service", "dmesg.service", "netplan-*", "open-vm-tools.service",
    "piboot-*", "rpi-eeprom-update.service", "sshd-keygen.service", "vgauth.service",
];

fn pattern_matches(pat: &str, unit: &str) -> bool {
    if let Some(prefix) = pat.strip_suffix('*') {
        unit.starts_with(prefix)
    } else if let Some(suffix) = pat.strip_prefix('*') {
        unit.ends_with(suffix)
    } else if let Some((a, b)) = pat.split_once('*') {
        unit.starts_with(a) && unit.ends_with(b) && unit.len() >= a.len() + b.len()
    } else {
        pat == unit
    }
}

fn is_ignored(unit: &str) -> bool {
    IGNORE.iter().any(|p| pattern_matches(p, unit))
}

/// "foo" becomes "foo.service"; names that already have a unit suffix stay.
fn full_unit(unit: &str) -> String {
    const SUFFIXES: &[&str] = &[".service", ".socket", ".timer", ".target", ".mount", ".path"];
    if SUFFIXES.iter().any(|s| unit.ends_with(s)) {
        unit.to_string()
    } else {
        format!("{unit}.service")
    }
}

fn parse_register(text: &str) -> Result<Register, String> {
    toml::from_str(text).map_err(|e| format!("the register file is not valid: {e}"))
}

fn find_register(flag: &Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = flag {
        return Ok(p.clone());
    }
    for c in [DEFAULT_REGISTER, "./register.toml"] {
        if Path::new(c).is_file() {
            return Ok(PathBuf::from(c));
        }
    }
    Err(format!(
        "no register found. Looked for {DEFAULT_REGISTER} and ./register.toml. Use --register <path>."
    ))
}

fn load(flag: &Option<PathBuf>) -> Result<Register, String> {
    let path = find_register(flag)?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let reg = parse_register(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut seen = std::collections::HashSet::new();
    for s in &reg.services {
        if !seen.insert(&s.name) {
            return Err(format!("{}: the name '{}' appears twice", path.display(), s.name));
        }
    }
    Ok(reg)
}

fn systemctl(args: &[&str]) -> Result<(bool, String), String> {
    let out = Command::new("systemctl")
        .args(args)
        .output()
        .map_err(|e| format!("could not run systemctl (is this a systemd machine?): {e}"))?;
    Ok((
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

fn unit_exists(unit: &str) -> bool {
    matches!(systemctl(&["show", "-p", "LoadState", "--value", unit]),
        Ok((_, s)) if s != "not-found" && !s.is_empty())
}

fn unit_state(unit: &str) -> (String, String) {
    if !unit_exists(unit) {
        return ("missing".into(), "missing".into());
    }
    let active = systemctl(&["is-active", unit]).map(|r| r.1).unwrap_or_else(|_| "unknown".into());
    let enabled = systemctl(&["is-enabled", unit]).map(|r| r.1).unwrap_or_else(|_| "unknown".into());
    (active, enabled)
}

fn health(s: &Service) -> &'static str {
    match &s.health {
        None => "-",
        Some(cmd) => match Command::new("sh").arg("-c").arg(cmd).output() {
            Ok(o) if o.status.success() => "OK",
            _ => "FAIL",
        },
    }
}

fn find<'a>(reg: &'a Register, name: &str) -> Result<&'a Service, String> {
    reg.services.iter().find(|s| s.name == name).ok_or_else(|| {
        let known: Vec<&str> = reg.services.iter().map(|s| s.name.as_str()).collect();
        format!("'{name}' is not in the register. Known services: {}", known.join(", "))
    })
}

fn cmd_status(reg: &Register) -> Result<(), String> {
    println!("{:<28} {:<10} {:<10} {:<10} {:<7} WANT-BOOT", "NAME", "CATEGORY", "ACTIVE", "BOOT", "HEALTH");
    for s in &reg.services {
        let (a, e) = unit_state(&full_unit(&s.unit));
        println!("{:<28} {:<10} {:<10} {:<10} {:<7} {}", s.name, s.category, a, e, health(s),
            if s.boot { "yes" } else { "no" });
    }
    Ok(())
}

fn cmd_show(reg: &Register, name: &str) -> Result<(), String> {
    let s = find(reg, name)?;
    println!("Name:       {}", s.name);
    println!("Purpose:    {}", s.purpose);
    println!("Unit:       {}", s.unit);
    println!("Category:   {}", s.category);
    println!("Owner:      {}", s.owner);
    if let Some(p) = s.port { println!("Port:       {p}"); }
    if !s.depends_on.is_empty() { println!("Depends on: {}", s.depends_on.join(", ")); }
    if !s.files.is_empty() { println!("Files:      {}", s.files.join(", ")); }
    if let Some(x) = &s.secrets { println!("Secrets:    {x}"); }
    if let Some(h) = &s.health { println!("Health cmd: {h}"); }
    println!("Boot:       {}", if s.boot { "yes" } else { "no" });
    if let Some(n) = &s.notes { println!("Notes:      {n}"); }
    let (a, e) = unit_state(&full_unit(&s.unit));
    println!("Right now:  {a} (boot: {e}, health: {})", health(s));
    Ok(())
}

fn cmd_control(reg: &Register, verb: &str, name: &str) -> Result<(), String> {
    let s = find(reg, name)?;
    let unit = full_unit(&s.unit);
    let (ok, _) = systemctl(&[verb, &unit])?;
    if ok {
        println!("{verb} {unit}: done");
        Ok(())
    } else {
        Err(format!("systemctl {verb} {unit} failed (you may need sudo, or the unit may not exist)"))
    }
}

fn cmd_check(reg: &Register) -> Result<(), String> {
    let mut problems = 0;
    for s in &reg.services {
        if !unit_exists(&full_unit(&s.unit)) {
            println!("MISSING       {} (unit {} not found on this machine)", s.name, s.unit);
            problems += 1;
        }
    }
    let (ok, out) = systemctl(&["list-unit-files", "--type=service", "--state=enabled", "--no-legend", "--plain"])?;
    if !ok {
        return Err("could not list the enabled units on this machine".into());
    }
    let known: std::collections::HashSet<String> =
        reg.services.iter().map(|s| full_unit(&s.unit)).collect();
    for unit in out.lines().filter_map(|l| l.split_whitespace().next()) {
        if !known.contains(unit) && !is_ignored(unit) {
            println!("UNREGISTERED  {unit} is enabled but not in the register");
            problems += 1;
        }
    }
    if problems == 0 {
        println!("All good: register and machine agree.");
        Ok(())
    } else {
        Err(format!("{problems} problem(s) found"))
    }
}

fn cmd_doc(reg: &Register) -> Result<(), String> {
    println!("# Services on this machine\n");
    println!("Generated from the register by pi-register. {} services.\n", reg.services.len());
    let mut cats: Vec<&str> = reg.services.iter().map(|s| s.category.as_str()).collect();
    cats.sort();
    cats.dedup();
    for c in cats {
        println!("## {c}\n");
        for s in reg.services.iter().filter(|s| s.category == c) {
            println!("### {}\n", s.name);
            println!("{}\n", s.purpose);
            println!("- Unit: `{}`", s.unit);
            println!("- Owner: {}", s.owner);
            if let Some(p) = s.port { println!("- Port: {p}"); }
            if !s.depends_on.is_empty() { println!("- Depends on: {}", s.depends_on.join(", ")); }
            if !s.files.is_empty() {
                println!("- Files: {}", s.files.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", "));
            }
            if let Some(x) = &s.secrets { println!("- Secrets live in: `{x}`"); }
            if let Some(h) = &s.health { println!("- Health check: `{h}`"); }
            println!("- Starts at boot: {}", if s.boot { "yes" } else { "no" });
            if let Some(n) = &s.notes { println!("- Notes: {n}"); }
            println!();
        }
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let reg = load(&cli.register)?;
    match cli.command {
        Cmd::Status => cmd_status(&reg),
        Cmd::Show { name } => cmd_show(&reg, &name),
        Cmd::Start { name } => cmd_control(&reg, "start", &name),
        Cmd::Stop { name } => cmd_control(&reg, "stop", &name),
        Cmd::Enable { name } => cmd_control(&reg, "enable", &name),
        Cmd::Disable { name } => cmd_control(&reg, "disable", &name),
        Cmd::Check => cmd_check(&reg),
        Cmd::Doc => cmd_doc(&reg),
    }
}

#[cfg(unix)]
extern "C" {
    fn signal(signum: i32, handler: usize) -> usize;
}

/// Restore the default SIGPIPE action so `pi-register status | head -1` exits quietly
/// instead of panicking when the reader closes the pipe.
fn restore_sigpipe() {
    #[cfg(unix)]
    // SAFETY: signal(SIGPIPE=13, SIG_DFL=0) is a plain libc call with no Rust-side state.
    unsafe {
        signal(13, 0);
    }
}

fn main() -> ExitCode {
    restore_sigpipe();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("pi-register: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[[service]]
name = "demo"
purpose = "Does a thing."
unit = "demo"
category = "network"
owner = "drake"
port = 8080
depends_on = ["other"]
files = ["/etc/demo.conf"]
secrets = "/etc/demo/secret"
health = "true"
boot = true
notes = "hi"

[[service]]
name = "min"
purpose = "Minimal."
unit = "min.service"
category = "bot"
owner = "drake"
boot = false
"#;

    #[test]
    fn parses_full_and_minimal_entries() {
        let r = parse_register(SAMPLE).unwrap();
        assert_eq!(r.services.len(), 2);
        assert_eq!(r.services[0].port, Some(8080));
        assert_eq!(r.services[0].depends_on, vec!["other"]);
        assert!(r.services[1].port.is_none() && r.services[1].files.is_empty());
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(parse_register("[[service]]\nname = \"x\"\n").is_err());
    }

    #[test]
    fn ignore_list_filters_base_units_only() {
        assert!(is_ignored("systemd-journald.service"));
        assert!(is_ignored("getty@tty1.service"));
        assert!(is_ignored("ssh.service"));
        assert!(is_ignored("chrony.service"));
        assert!(is_ignored("piboot-try-reboot.service"));
        assert!(!is_ignored("headscale.service"));
        assert!(!is_ignored("fail2ban.service"));
    }

    #[test]
    fn unit_names_get_suffix() {
        assert_eq!(full_unit("vault"), "vault.service");
        assert_eq!(full_unit("x.timer"), "x.timer");
    }

    #[test]
    fn unknown_name_is_refused() {
        let r = parse_register(SAMPLE).unwrap();
        assert!(find(&r, "nope").is_err());
        assert!(find(&r, "demo").is_ok());
    }
}
