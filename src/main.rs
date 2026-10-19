mod kill;
mod scan;
mod tui;
mod completions;

use anyhow::{Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nportx")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Find and kill processes hogging your ports. Instantly.")]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// List all listening TCP ports
    List,
    /// Show the process owning a specific port
    Who {
        /// Port number to inspect
        port: u16,
    },
    /// Kill the process listening on a port
    Kill {
        /// Port number to kill
        port: u16,

        /// Skip the confirmation prompt (useful for scripts)
        #[arg(short, long)]
        force: bool,
    },
    /// Exit 0 if port is free, 1 if taken
    Check {
        /// Port number to check
        port: u16,
    },
    /// Interactive TUI
    Watch {
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },
        /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let ports = scan::scan_listening_ports()?;
            for p in ports {
                println!(
                    "Port: {} | PID: {} | Process: {}",
                    p.port, p.pid, p.process
                );
            }
        }
        Commands::Who { port } => match scan::find_port(port)? {
            Some(info) => {
                println!(
                    "Port {} is owned by {} (PID {})",
                    port, info.process, info.pid
                );
            }
            None => {
                println!("Port {} is free (no process listening).", port);
            }
        },
        Commands::Kill { port, force } => {
            kill::kill_port(port, force)?;
        }
        Commands::Check { port } => {
            let taken = scan::find_port(port)?.is_some();
            if taken {
                std::process::exit(1);
            } else {
                std::process::exit(0);
            }
        }
        Commands::Watch { interval } => tui::run(interval)?,
        Commands::Completions { shell } => {
            completions::generate_completions(shell)?;
        }
    }

    Ok(())
}