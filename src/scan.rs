use anyhow::Result;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use std::collections::HashMap;
use sysinfo::{Pid, System};

#[derive(Debug, Clone)]
pub struct PortEntry {
    pub port: u16,
    pub pid: u32,
    pub process: String,
    pub memory_bytes: u64,
}

/// Returns a map of (port -> container_name) for running Docker containers.
fn get_docker_containers() -> HashMap<u16, String> {
    let mut map = HashMap::new();

    // We use std::process::Command to ask Docker for its running containers.
    let mut cmd = std::process::Command::new("docker");
    cmd.args(["ps", "--format", "{{.Names}} {{.Ports}}"]);

    // On Windows, prevent a console window from flashing up
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    // If docker isn't installed or the daemon isn't running, this quietly fails and returns empty.
    if let Ok(output) = cmd.output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                // Format: "my-postgres 0.0.0.0:5432->5432/tcp"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0];
                    // Find the host port (the number before the ->)
                    if let Some(host_port_str) = parts[1].split("->").next() {
                        // Split by ':' to get the port at the end
                        if let Some(port_str) = host_port_str.split(':').last() {
                            if let Ok(port) = port_str.parse::<u16>() {
                                map.insert(port, format!("[docker] {}", name));
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

pub fn scan_listening_ports() -> Result<Vec<PortEntry>> {
    let mut ports = Vec::new();

    // 1. Initialize sysinfo to get process names and memory
    let mut sys = System::new_all();
    sys.refresh_all();

    // 2. Get Docker container mappings (fails silently if no Docker)
    let docker_map = get_docker_containers();

    // 3. Get sockets (adjust flags based on your OS if needed)
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;
    let sockets = get_sockets_info(af_flags, proto_flags)?;

    for socket in sockets {
        // Only care about sockets that are listening
        if let ProtocolSocketInfo::Tcp(tcp_info) = socket.protocol_socket_info {
            if !tcp_info.state.eq(&netstat2::TcpState::Listen) {
                continue;
            }

            // Get PIDs associated with this socket
            for pid in socket.associated_pids {
                if pid == 0 {
                    continue; // Skip PID 0
                }

                let mut process_name = sys
                    .process(Pid::from_u32(pid))
                    .map(|p| p.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| "<unknown>".to_string());

                // Override with Docker container name if applicable
                if process_name.to_lowercase().contains("docker") {
                    if let Some(container_name) = docker_map.get(&tcp_info.local_port) {
                        process_name = container_name.clone();
                    }
                }

                let memory = sys.process(Pid::from_u32(pid)).map(|p| p.memory()).unwrap_or(0);

                // Deduplicate ports
                if !ports.iter().any(|p: &PortEntry| p.port == tcp_info.local_port) {
                    ports.push(PortEntry {
                        port: tcp_info.local_port,
                        pid,
                        process: process_name,
                        memory_bytes: memory,
                    });
                }
            }
        }
    }

    // Sort by port number for consistency
    ports.sort_by_key(|p| p.port);
    Ok(ports)
}

/// Find the process listening on a specific port.
/// Returns `Ok(None)` if the port is free.
pub fn find_port(port: u16) -> Result<Option<PortEntry>> {
    let all = scan_listening_ports()?;
    Ok(all.into_iter().find(|p| p.port == port))
}