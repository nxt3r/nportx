use std::time::{Duration, Instant};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PortEntry {
    pub port: u16,
    pub pid: u32,
    pub process: String,
    pub memory_bytes: u64,
}

pub struct App {
    pub ports: Vec<PortEntry>,
    pub selected: usize,
    pub should_quit: bool,
    pub refresh_interval: Duration,
    pub last_refresh: Instant,
    pub status: String,
}

impl App {
    pub fn new(refresh_interval: Duration) -> Self {
        Self {
            ports: Vec::new(),
            selected: 0,
            should_quit: false,
            refresh_interval,
            last_refresh: Instant::now(),
            status: String::new(),
        }
    }

    pub fn refresh_ports(&mut self) {
        // Stage 1: stub. Stage 2: wire in your real netstat2/sysinfo scan.
        self.last_refresh = Instant::now();
        self.status = format!("Loaded {} ports", self.ports.len());
    }
}