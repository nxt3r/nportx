mod completions;
mod detect;
mod kill;
mod scan;
mod tui;

use anyhow::Result;
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

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[derive(Subcommand)]
enum Commands {
    /// List all listening ports
    List {
        /// Include UDP sockets (shows a PROTO column)
        #[arg(long)]
        udp: bool,
    },
    /// Show the process owning a specific port
    Who {
        /// Port number to inspect
        port: u16,
    },
    /// Kill the process listening on a port, or by process name
    #[command(after_help = "\
 Examples:
  nportx kill 3000          # prompts [y/N] before killing port 3000
  nportx kill 3000 -f       # skips prompt (for scripts/CI)
  nportx kill --name node   # kills all ports owned by 'node'
  nportx kill --name node -f")]
    Kill {
        /// Port number to kill
        #[arg(conflicts_with = "name")]
        port: Option<u16>,

        /// Process name to kill (e.g., 'node', 'python')
        #[arg(short, long)]
        name: Option<String>,

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
    /// Detect expected ports for the current project and cross-check them
    #[command(after_help = "\
Reads .env, .env.local, .env.development, package.json, and docker-compose.yml.

Examples:
  cd my-vite-app
  nportx doctor             # shows expected ports vs. what's actually listening

Supported sources:
  - .env / .env.local / .env.development  (any VAR ending in PORT)
  - package.json scripts.start / scripts.dev  (-p, --port, --port=)
  - docker-compose.yml / .yaml  (host ports)
  - Framework defaults: Vite=5173, Next.js=3000, CRA=3000, Nuxt=3000, Gatsby=8000")]
    Doctor,
    /// Export listening ports to JSON or CSV
    Export {
        /// Output format
        #[arg(value_enum)]
        format: ExportFormat,

        /// Include UDP sockets
        #[arg(long)]
        udp: bool,
    },
    /// Find the next available free port starting from a given number
    Free {
        /// Port number to start searching from
        start_port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { udp } => {
            use colored::Colorize;
            use colored::control;

            control::set_override(true);

            let ports = scan::scan_ports(udp)?;
            if ports.is_empty() {
                println!("No listening ports found.");
                return Ok(());
            }

            if udp {
                // With PROTO column
                println!(
                    "{:>6}  {:<5}  {:>8}  {:<30}  {:>10}",
                    "PORT".bold(),
                    "PROTO".bold(),
                    "PID".bold(),
                    "PROCESS".bold(),
                    "MEMORY".bold()
                );
                println!("{}", "─".repeat(67).bright_black());

                for p in ports {
                    let process_colored = color_process(&p.process);
                    let mem_colored = color_memory(p.memory_bytes);
                    let proto_colored = match p.protocol {
                        scan::Protocol::Tcp => "TCP".cyan(),
                        scan::Protocol::Udp => "UDP".magenta(),
                    };

                    println!(
                        "{:>6}  {:<5}  {:>8}  {:<30}  {:>10}",
                        p.port.to_string().cyan(),
                        proto_colored,
                        p.pid.to_string().yellow(),
                        process_colored,
                        mem_colored
                    );
                }
            } else {
                // Classic layout (TCP only, no PROTO column)
                println!(
                    "{:>6}  {:>8}  {:<30}  {:>10}",
                    "PORT".bold(),
                    "PID".bold(),
                    "PROCESS".bold(),
                    "MEMORY".bold()
                );
                println!("{}", "─".repeat(60).bright_black());

                for p in ports {
                    println!(
                        "{:>6}  {:>8}  {:<30}  {:>10}",
                        p.port.to_string().cyan(),
                        p.pid.to_string().yellow(),
                        color_process(&p.process),
                        color_memory(p.memory_bytes)
                    );
                }
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
        Commands::Kill { port, name, force } => {
            kill::kill_process(port, name, force)?;
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
        Commands::Doctor => {
            run_doctor()?;
        }
        Commands::Completions { shell } => {
            completions::generate_completions(shell)?;
        }
        Commands::Export { format, udp } => {
            let ports = scan::scan_ports(udp)?;
            if ports.is_empty() {
                return Ok(());
            }

            match format {
                ExportFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&ports)?);
                }
                ExportFormat::Csv => {
                    let mut wtr = csv::Writer::from_writer(std::io::stdout());
                    wtr.write_record(&[
                        "port",
                        "protocol",
                        "pid",
                        "process",
                        "memory_bytes",
                    ])?;
                    for p in &ports {
                        wtr.write_record(&[
                            p.port.to_string(),
                            p.protocol.to_string(),
                            p.pid.to_string(),
                            p.process.clone(),
                            p.memory_bytes.to_string(),
                        ])?;
                    }
                    wtr.flush()?;
                }
            }
        }
        Commands::Free { start_port } => {
            use colored::Colorize;

            let ports = scan::scan_listening_ports()?;

            let start_taken = ports.iter().any(|p| p.port == start_port);
            if !start_taken {
                println!(
                    "Port {} is free.",
                    start_port.to_string().bright_green().bold()
                );
                return Ok(());
            }

            let mut found_port = None;
            for candidate in (start_port + 1)..=65535 {
                if !ports.iter().any(|p| p.port == candidate) {
                    found_port = Some(candidate);
                    break;
                }
            }

            match found_port {
                Some(free_port) => {
                    println!(
                        "Port {} is taken. Next free port is {}.",
                        start_port.to_string().red(),
                        free_port.to_string().bright_green().bold()
                    );
                }
                None => {
                    println!("No free ports found above {}.", start_port);
                }
            }
        }
    }

    Ok(())
}

/// Shared color logic for process names in `list`
fn color_process(name: &str) -> colored::ColoredString {
    use colored::Colorize;
    if name.contains("[docker]") {
        name.magenta()
    } else if kill::is_system_critical(name) {
        name.red()
    } else {
        name.green()
    }
}

/// Shared color logic for memory column in `list`
fn color_memory(bytes: u64) -> colored::ColoredString {
    use colored::Colorize;
    const FIFTY_MB: u64 = 50 * 1024 * 1024;
    if bytes == 0 {
        "0 B".bright_black()
    } else if bytes > FIFTY_MB {
        format_memory(bytes).bright_green().bold()
    } else {
        format_memory(bytes).white()
    }
}

fn run_doctor() -> Result<()> {
    use colored::Colorize;

    let cwd = std::env::current_dir()?;
    let expected = detect::detect_expected_ports(&cwd)?;

    if expected.is_empty() {
        println!(
            "{} No project port configuration detected.",
            "ℹ️ ".bright_blue()
        );
        println!(
            "  Looked for .env, .env.local, .env.development, and package.json in {}",
            cwd.display()
        );
        println!("  Run `nportx list` to see all listening ports.");
        return Ok(());
    }

    let listening = scan::scan_listening_ports()?;

    println!(
        "{} {} expected port(s) in {}\n",
        "🔍".to_string(),
        expected.len(),
        cwd.display()
    );

    println!(
        "{:<6}  {:<10}  {:<40}  {}",
        "PORT".bold(),
        "STATUS".bold(),
        "EXPECTED BY".bold(),
        "PROCESS".bold()
    );
    println!("{}", "─".repeat(90).bright_black());

    for exp in &expected {
        let hit = listening.iter().find(|p| p.port == exp.port);
        match hit {
            Some(p) => {
                println!(
                    "{:<6}  {:<10}  {:<40}  {}",
                    exp.port,
                    "✅ listening".bright_green(),
                    exp.source,
                    format!("{} (PID {})", p.process, p.pid).bright_green()
                );
            }
            None => {
                println!(
                    "{:<6}  {:<10}  {:<40}  {}",
                    exp.port,
                    "❌ free".bright_red(),
                    exp.source,
                    "—".bright_black()
                );
            }
        }
    }

    Ok(())
}

fn format_memory(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}