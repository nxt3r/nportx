use anyhow::Result;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use std::collections::HashMap;
use std::fmt;
use sysinfo::{Pid, System};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PortEntry {
    pub port: u16,
    pub protocol: Protocol,
    pub pid: u32,
    pub process: String,
    pub memory_bytes: u64,
}

/// Returns a map of (port -> container_name) for running Docker containers.
fn get_docker_containers() -> HashMap<u16, String> {
    let mut map = HashMap::new();

    let mut cmd = std::process::Command::new("docker");
    cmd.args(["ps", "--format", "{{.Names}} {{.Ports}}"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if let Ok(output) = cmd.output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0];
                    if let Some(host_port_str) = parts[1].split("->").next() {
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

/// Scan TCP ports only. Backwards-compatible wrapper used by kill/who/check/doctor/tui.
pub fn scan_listening_ports() -> Result<Vec<PortEntry>> {
    scan_ports(false)
}

/// Scan listening ports. If `include_udp` is true, UDP sockets are included too.
pub fn scan_ports(include_udp: bool) -> Result<Vec<PortEntry>> {
    let mut ports: Vec<PortEntry> = Vec::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    let docker_map = get_docker_containers();

    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let mut proto_flags = ProtocolFlags::TCP;
    if include_udp {
        proto_flags |= ProtocolFlags::UDP;
    }

    let sockets = get_sockets_info(af_flags, proto_flags)?;

    for socket in sockets {
        match socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_info) => {
                if !tcp_info.state.eq(&netstat2::TcpState::Listen) {
                    continue;
                }
                let port = tcp_info.local_port;
                for pid in socket.associated_pids {
                    if pid == 0 {
                        continue;
                    }
                    push_entry(
                        &mut ports,
                        port,
                        Protocol::Tcp,
                        pid,
                        &docker_map,
                        &sys,
                    );
                }
            }
            ProtocolSocketInfo::Udp(udp_info) => {
                let port = udp_info.local_port;
                for pid in socket.associated_pids {
                    if pid == 0 {
                        continue;
                    }
                    push_entry(
                        &mut ports,
                        port,
                        Protocol::Udp,
                        pid,
                        &docker_map,
                        &sys,
                    );
                }
            }
        }
    }

    // Sort by port, then protocol (TCP before UDP) for stable ordering
    ports.sort_by(|a, b| {
        a.port
            .cmp(&b.port)
            .then_with(|| (a.protocol as u8).cmp(&(b.protocol as u8)))
    });
    Ok(ports)
}

fn push_entry(
    out: &mut Vec<PortEntry>,
    port: u16,
    protocol: Protocol,
    pid: u32,
    docker_map: &HashMap<u16, String>,
    sys: &System,
) {
    let mut process_name = sys
        .process(Pid::from_u32(pid))
        .map(|p| p.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| "<unknown>".to_string());

    if process_name.to_lowercase().contains("docker") {
        if let Some(container_name) = docker_map.get(&port) {
            process_name = container_name.clone();
        }
    }

    let memory = sys.process(Pid::from_u32(pid)).map(|p| p.memory()).unwrap_or(0);

    // Deduplicate by (port, protocol)
    if !out
        .iter()
        .any(|p| p.port == port && p.protocol == protocol)
    {
        out.push(PortEntry {
            port,
            protocol,
            pid,
            process: process_name,
            memory_bytes: memory,
        });
    }
}

/// Find the process listening on a specific TCP port.
/// Returns `Ok(None)` if the port is free.
pub fn find_port(port: u16) -> Result<Option<PortEntry>> {
    let all = scan_listening_ports()?;
    Ok(all.into_iter().find(|p| p.port == port))
}