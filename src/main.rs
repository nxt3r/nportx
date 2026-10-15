use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState,
};
use std::io::{self, Write};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "nportx")]
#[command(about = "Find and kill processes hogging your ports", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all listening TCP ports
    List,
    /// Show what process is on a specific port
    Who {
        /// Port number
        port: u16,
    },
    /// Kill the process listening on a specific port
    Kill {
        /// Port number
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_listening()?,
        Commands::Who { port } => who(port)?,
        Commands::Kill { port } => kill(port)?,
    }

    Ok(())
}

struct PortEntry {
    port: u16,
    pid: u32,
    process_name: String,
    memory_mb: u64,
}

fn listening_ports() -> Result<Vec<(u16, u32)>> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;

    let sockets = get_sockets_info(af_flags, proto_flags)?;

    let mut out: Vec<(u16, u32)> = Vec::new();

    for socket in sockets {
        if let ProtocolSocketInfo::Tcp(tcp) = socket.protocol_socket_info {
            if tcp.state != TcpState::Listen {
                continue;
            }

            let Some(&pid) = socket.associated_pids.first() else {
                continue;
            };

            if pid == 0 {
                continue;
            }

            if out.iter().any(|(p, _)| *p == tcp.local_port) {
                continue;
            }

            out.push((tcp.local_port, pid));
        }
    }

    out.sort_by_key(|(p, _)| *p);
    Ok(out)
}

fn scan() -> Result<Vec<PortEntry>> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );

    let ports = listening_ports()?;

    let entries = ports
        .into_iter()
        .map(|(port, pid)| {
            let (process_name, memory_mb) = match sys.process(sysinfo::Pid::from_u32(pid)) {
                Some(p) => (
                    p.name().to_string_lossy().to_string(),
                    p.memory() / 1_048_576,
                ),
                None => ("<unknown>".to_string(), 0),
            };

            PortEntry {
                port,
                pid,
                process_name,
                memory_mb,
            }
        })
        .collect();

    Ok(entries)
}

fn list_listening() -> Result<()> {
    let entries = scan()?;

    println!(
        "{:<8} {:<24} {:<8} {:<10}",
        "PORT".bold(),
        "PROCESS".bold(),
        "PID".bold(),
        "MEM".bold()
    );
    println!("{}", "-".repeat(52).dimmed());

    for e in entries {
        let color_fn: fn(&str) -> colored::ColoredString = if is_system_critical(&e.process_name)
        {
            |s| s.red()
        } else if e.process_name == "<unknown>" {
            |s| s.yellow()
        } else {
            |s| s.green()
        };

        println!(
            "{:<8} {:<24} {:<8} {:<10}",
            color_fn(&e.port.to_string()),
            color_fn(&truncate(&e.process_name, 24)),
            e.pid,
            format!("{} MB", e.memory_mb).dimmed()
        );
    }

    Ok(())
}

fn who(port: u16) -> Result<()> {
    let entries = scan()?;

    match entries.iter().find(|e| e.port == port) {
        Some(e) => {
            let name = if is_system_critical(&e.process_name) {
                e.process_name.red().bold()
            } else {
                e.process_name.green().bold()
            };
            println!(
                "Port {} is held by PID {} ({}, {} MB)",
                port.to_string().cyan().bold(),
                e.pid.to_string().yellow(),
                name,
                e.memory_mb
            );
            Ok(())
        }
        None => {
            bail!(
                "Port {} is free — nothing is listening on it.",
                port
            );
        }
    }
}

fn kill(port: u16) -> Result<()> {
    // 1. Find the process on this port
    let entries = scan()?;
    let entry = match entries.iter().find(|e| e.port == port) {
        Some(e) => e,
        None => {
            bail!("Port {} is free — nothing to kill.", port);
        }
    };

    // 2. Safety check: don't let anyone kill critical Windows processes
    if is_system_critical(&entry.process_name) {
        bail!(
            "Refusing to kill '{}' (PID {}) — it's a system-critical process.",
            entry.process_name,
            entry.pid
        );
    }

    // 3. Show what we're about to do, ask for confirmation
println!(
    "About to kill: {} {} on port {}",
    entry.process_name.red().bold(),
    format!("(PID {})", entry.pid).dimmed(),
    port.to_string().cyan().bold()
);
    print!("Are you sure? [y/N] ");
    io::stdout().flush().context("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read user input")?;

    if input.trim().to_lowercase() != "y" {
        println!("Aborted.");
        return Ok(());
    }

    // 4. Do the kill
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );

    let Some(process) = sys.process(sysinfo::Pid::from_u32(entry.pid)) else {
        bail!("Process with PID {} no longer exists.", entry.pid);
    };

    if process.kill() {
        println!(
            "✅ Killed {} (PID {}) — port {} is now free.",
            entry.process_name, entry.pid, port
        );
        Ok(())
    } else {
        bail!(
            "❌ Failed to kill {} (PID {}). Try running as administrator.",
            entry.process_name,
            entry.pid
        );
    }
}

/// Refuse to kill the Windows processes that keep the OS alive.
fn is_system_critical(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "system"
            | "system idle process"
            | "csrss.exe"
            | "wininit.exe"
            | "winlogon.exe"
            | "services.exe"
            | "lsass.exe"
            | "smss.exe"
            | "registry"
            | "memory compression"
            | "svchost.exe" // risky but a lot of critical services use it
    )
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}