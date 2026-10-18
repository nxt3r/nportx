mod scan;
mod tui;
mod kill;

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
    List,
    Who { port: u16 },
    Kill { port: u16 },
    Check { port: u16 },
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
            for p in ports {
                println!("Port: {} | PID: {} | Process: {}", p.port, p.pid, p.process);
            }
        }
        Commands::Watch { interval } => tui::run(interval)?,
        // ... your existing Who, Kill, Check logic ...
        _ => {}
    }

    Ok(())
}