use std::time::{Duration, Instant};
use crate::scan::{scan_listening_ports, PortEntry};
use crate::kill::{is_system_critical, kill_pid};

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    ConfirmKill { port: u16, pid: u32, process: String },
}

pub struct App {
    pub ports: Vec<PortEntry>,
    pub selected: usize,
    pub should_quit: bool,
    pub refresh_interval: Duration,
    pub last_refresh: Instant,
    pub status: String,
    pub mode: Mode,
    pub search: String,
    pub searching: bool,
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
            mode: Mode::Normal,
            search: String::new(),
            searching: false,
        }
    }

    pub fn refresh_ports(&mut self) {
        match scan_listening_ports() {
            Ok(ports) => {
                self.ports = ports;
                self.clamp_selection();
                self.status = format!("{} listening ports", self.ports.len());
            }
            Err(e) => {
                self.status = format!("Scan error: {e}");
            }
        }
        self.last_refresh = Instant::now();
    }

    pub fn visible(&self) -> Vec<&PortEntry> {
        if !self.search.is_empty() {
            let query = self.search.to_lowercase();
            self.ports.iter().filter(|p| {
                p.process.to_lowercase().contains(&query) || p.port.to_string().contains(&query)
            }).collect()
        } else {
            self.ports.iter().collect()
        }
    }

    pub fn clamp_selection(&mut self) {
        let len = self.visible().len();
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }

    pub fn next(&mut self) {
        let len = self.visible().len();
        if len > 0 {
            self.selected = (self.selected + 1) % len;
        }
    }

    pub fn prev(&mut self) {
        let len = self.visible().len();
        if len > 0 {
            self.selected = if self.selected == 0 { len - 1 } else { self.selected - 1 };
        }
    }

    pub fn begin_kill(&mut self) {
        let visible = self.visible();
        if let Some(p) = visible.get(self.selected) {
            if is_system_critical(&p.process) {
                self.status = format!("⛔ Refusing to kill critical process: {}", p.process);
                return;
            }
            self.mode = Mode::ConfirmKill {
                port: p.port,
                pid: p.pid,
                process: p.process.clone(),
            };
        }
    }

    pub fn confirm_kill(&mut self, port: u16, pid: u32, process: &str) {
        match kill_pid(pid) {
            Ok(_) => self.status = format!("✅ Killed {process} (PID {pid}) on port {port}"),
            Err(e) => self.status = format!("❌ Kill failed: {e}"),
        }
        self.mode = Mode::Normal;
        self.refresh_ports();
    }
}