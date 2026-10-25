use anyhow::{bail, Result};
use std::io::{self, Write};

use crate::scan;

pub fn is_system_critical(process_name: &str) -> bool {
    let name = process_name.to_lowercase();
    matches!(
        name.as_str(),
        "system"
            | "lsass.exe"
            | "svchost.exe"
            | "wininit.exe"
            | "services.exe"
            | "smss.exe"
            | "csrss.exe"
            | "winlogon.exe"
    )
}

/// Unified entry point for `nportx kill`
pub fn kill_process(port: Option<u16>, name: Option<String>, force: bool) -> Result<()> {
    if let Some(port) = port {
        return kill_port(port, force);
    }

    if let Some(name) = name {
        return kill_by_name(&name, force);
    }

    // Clap usually prevents this, but just in case:
    bail!("Must specify either a port or a process name.");
}

/// Look up the process on `port`, confirm (unless `force`), then kill it.
fn kill_port(port: u16, force: bool) -> Result<()> {
    let info = match scan::find_port(port)? {
        Some(i) => i,
        None => {
            println!("Port {} is free — nothing to kill.", port);
            return Ok(());
        }
    };

    // Safety guard — runs no matter what, even with --force.
    if is_system_critical(&info.process) {
        bail!(
            "Refusing to kill system-critical process '{}' (PID {}) on port {}.",
            info.process,
            info.pid,
            port
        );
    }

    if force {
        println!(
            "Force-killing {} (PID {}) on port {}...",
            info.process, info.pid, port
        );
    } else {
        print!(
            "Kill process {} (PID {}) on port {}? [y/N] ",
            info.process, info.pid, port
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    kill_pid(info.pid)?;
    println!("Killed {} (PID {}) on port {}.", info.process, info.pid, port);
    Ok(())
}

/// Find all processes matching a name (partial match, case-insensitive)
fn kill_by_name(target_name: &str, force: bool) -> Result<()> {
    let ports = scan::scan_listening_ports()?;
    let target_lower = target_name.to_lowercase();

    // Filter ports where process name contains the target (e.g. "node" matches "node.exe")
    let matches: Vec<_> = ports
        .into_iter()
        .filter(|p| p.process.to_lowercase().contains(&target_lower))
        .collect();

    if matches.is_empty() {
        println!("No listening processes found matching '{}'.", target_name);
        return Ok(());
    }

    // Filter out system critical ones
    let safe_matches: Vec<_> = matches
        .into_iter()
        .filter(|p| {
            if is_system_critical(&p.process) {
                println!(
                    "Skipping system-critical process '{}' (PID {}) on port {}.",
                    p.process, p.pid, p.port
                );
                false
            } else {
                true
            }
        })
        .collect();

    if safe_matches.is_empty() {
        println!("All matching processes are system-critical. Nothing killed.");
        return Ok(());
    }

    println!("Found {} process(es) matching '{}':", safe_matches.len(), target_name);
    for p in &safe_matches {
        println!("  - {} (PID {}) on port {}", p.process, p.pid, p.port);
    }

    if !force {
        print!("Kill all {} process(es)? [y/N] ", safe_matches.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    } else {
        println!("Force-killing all {} process(es)...", safe_matches.len());
    }

    let mut killed_count = 0;
    for p in safe_matches {
        // We use a match here so if one fails (e.g. access denied), we don't abort the rest
        match kill_pid(p.pid) {
            Ok(_) => {
                println!("Killed {} (PID {}) on port {}.", p.process, p.pid, p.port);
                killed_count += 1;
            }
            Err(e) => {
                println!("Failed to kill {} (PID {}): {}", p.process, p.pid, e);
            }
        }
    }

    println!("\nDone. Successfully killed {} process(es).", killed_count);
    Ok(())
}

#[cfg(windows)]
pub fn kill_pid(pid: u32) -> Result<()> {
    let output = std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output()?;
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        bail!("taskkill failed: {}", err_msg.trim());
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn kill_pid(pid: u32) -> Result<()> {
    let output = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()?;
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        bail!("kill failed: {}", err_msg.trim());
    }
    Ok(())
}