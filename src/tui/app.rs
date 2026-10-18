use std::time::{Duration, Instant};
use crate::scan::{scan_listening_ports, PortEntry};

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
        }
    }

    pub fn refresh_ports(&mut self) {
        match scan_listening_ports() {
            Ok(ports) => {
                self.ports = ports;
                // Clamp selection if the list shrinks
                if self.selected >= self.ports.len() {
                    self.selected = self.ports.len().saturating_sub(1);
                }
                self.status = format!("{} listening ports", self.ports.len());
            }
            Err(e) => {
                self.status = format!("Scan error: {e}");
            }
        }
        self.last_refresh = Instant::now();
    }

    pub fn next(&mut self) {
        if !self.ports.is_empty() {
            self.selected = (self.selected + 1) % self.ports.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.ports.is_empty() {
            self.selected = if self.selected == 0 {
                self.ports.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}