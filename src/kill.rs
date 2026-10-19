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

/// Look up the process on `port`, confirm (unless `force`), then kill it.
pub fn kill_port(port: u16, force: bool) -> Result<()> {
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