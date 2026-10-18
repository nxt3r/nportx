use anyhow::{bail, Result};

pub fn is_system_critical(process_name: &str) -> bool {
    let name = process_name.to_lowercase();
    matches!(
        name.as_str(),
        "system" | "lsass.exe" | "svchost.exe" | "wininit.exe" | "services.exe" | "smss.exe" | "csrss.exe" | "winlogon.exe"
    )
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