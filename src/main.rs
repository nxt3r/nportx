use anyhow::Result;
use clap::{Parser, Subcommand};
use sysinfo::System;

#[derive(Parser)]
#[command(name = "nportx")]
#[command(about = "Find and kill processes hogging your ports", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all processes (placeholder — not yet port-scoped)
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_processes()?,
    }

    Ok(())
}

fn list_processes() -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("{:<8} {:<30} {:<10}", "PID", "NAME", "MEM (MB)");
    println!("{}", "-".repeat(50));

    for (pid, process) in sys.processes() {
        let mem_mb = process.memory() / 1_048_576; // bytes → MB
        println!(
            "{:<8} {:<30} {:<10}",
            pid,
            process.name().to_string_lossy(),
            mem_mb
        );
    }

    Ok(())
}