mod scan;
mod tui;
// Assuming you have a kill.rs for your guards. If not, see note below.

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nportx")]
#[command(version = "0.1.1")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all listening ports
    List,
    /// Find who owns a port
    Who { port: u16 },
    /// Kill a process on a port
    Kill { port: u16 },
    /// Check if a port is free (0 = free, 1 = taken)
    Check { port: u16 },
    /// Live-updating TUI view
    Watch {
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let ports = scan::scan_listening_ports()?;
            // (Your existing list formatting here, or just print debug for now)
            for p in ports {
                println!("Port: {} | PID: {} | Process: {}", p.port, p.pid, p.process);
            }
        }
        Commands::Who { port } => { /* your existing logic */ }
        Commands::Kill { port } => { /* your existing logic */ }
        Commands::Check { port } => { /* your existing logic */ }
        Commands::Watch { interval } => tui::run(interval)?,
    }

    Ok(())
}