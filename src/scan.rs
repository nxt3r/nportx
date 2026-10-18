use anyhow::Result;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use sysinfo::{Pid, System};

#[derive(Debug, Clone)]
pub struct PortEntry {
    pub port: u16,
    pub pid: u32,
    pub process: String,
    pub memory_bytes: u64,
}

pub fn scan_listening_ports() -> Result<Vec<PortEntry>> {
    let mut ports = Vec::new();
    
    // 1. Initialize sysinfo to get process names and memory
    let mut sys = System::new_all();
    sys.refresh_all();

    // 2. Get sockets (adjust flags based on your OS if needed)
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

                let process_name = sys.process(Pid::from_u32(pid))
                    .map(|p| p.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| "<unknown>".to_string());

                let memory = sys.process(Pid::from_u32(pid))
                    .map(|p| p.memory())
                    .unwrap_or(0);

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