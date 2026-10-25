# NportX

> Find and kill processes hogging your ports. Instantly. Cross-platform. Single binary.

## Table of Contents

- [Why Not `lsof` or `netstat`?](#why-not-lsof-or-netstat)
- [Install](#install)
  - [Windows (Recommended: Scoop)](#windows-recommended-scoop)
  - [macOS (Homebrew)](#macos-homebrew)
  - [macOS / Linux (One-Line Script)](#macos--linux-one-line-script)
  - [Manual Windows Install](#manual-windows-install)
  - [Via Cargo](#via-cargo)
- [Usage](#usage)
  - [The Interactive TUI](#the-interactive-tui)
  - [Example](#example)
  - [Use It in Scripts](#use-it-in-scripts)
- [Features](#features)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

"Port 3000 is already in use" — every developer, every day.

`lsof` is ugly. `netstat` is painful. Python tools are slow.

**NportX** is a single ~1.5 MB Rust binary that shows you what's listening on your ports and lets you terminate the process when you're done. It also includes a fully interactive terminal UI (TUI).

## Why Not `lsof` or `netstat`?

| Feature | `lsof` | `netstat` | **NportX** |
|---|:---:|:---:|:---:|
| Cross-platform | ❌ | ❌ | ✅ |
| Human-readable output | ❌ | ❌ | ✅ |
| Kill by port | ❌ | ❌ | ✅ |
| Scripting exit codes | ❌ | ❌ | ✅ |
| Interactive TUI | ❌ | ❌ | ✅ |
| Project-aware detection | ❌ | ❌ | ✅ |
| UDP support | ❌ | ✅ | ✅ |
| Export to JSON/CSV | ❌ | ❌ | ✅ |
| No runtime dependencies | ✅ | ✅ | ✅ |
| Instant Startup | ✅ | ✅ | ✅ |

## Install

### Windows (Recommended: Scoop)

If you don't have Scoop installed, run the following in PowerShell:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
```

Install Git if you haven't already:

```powershell
scoop install git
```

Add the NportX bucket and install NportX:

```powershell
scoop bucket add nportx https://github.com/nxt3r/scoop-nportx
scoop install nportx
```

Scoop handles PATH configuration, updates, and uninstallation automatically.

To upgrade later:

```powershell
scoop update
scoop update nportx
```

### macOS (Homebrew)

If you don't have Homebrew installed, run the following in your terminal:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Add the tap and install NportX:

```bash
brew tap nxt3r/nportx
brew install nportx
```

### macOS / Linux (One-Line Script)

```bash
curl -sSL https://raw.githubusercontent.com/nxt3r/nportx/main/install.sh | bash
```

The script detects your operating system and architecture (Intel/ARM), downloads the latest pre-built binary, and installs it to `/usr/local/bin` or `~/.local/bin`.

### Manual Windows Install

If you don't want to use Scoop:

1. Download `nportx-x86_64-pc-windows-msvc.zip` from the [Releases page](https://github.com/nxt3r/nportx/releases).
2. Extract `nportx.exe` to `%USERPROFILE%\bin\`. Create the folder if it doesn't exist.
3. Add `%USERPROFILE%\bin` to your PATH through Windows Environment Variables.
4. Restart PowerShell.

### Via Cargo

If you have Rust installed, install NportX from crates.io:

```bash
cargo install nportx
```

## Usage

```bash
nportx watch                 # Open the interactive TUI
nportx list                  # Show all listening TCP ports (with Docker awareness)
nportx list --udp            # Show TCP and UDP ports (with PROTO column)
nportx who 3000              # Find the process using port 3000
nportx kill 3000             # Kill the process using port 3000 (with confirmation)
nportx kill 3000 -f          # Skips prompt (for scripts)
nportx kill --name 'x'       # Kill all processes named "x" (asks for confirmation)
nportx doctor                # Detect expected project ports and cross-check them
nportx check 3000            # Exit 0 if free, 1 if taken — for scripts
nportx export json           # Export listening ports to JSON
nportx export csv            # Export listening ports to CSV
nportx free 3000             # Find the next available free port starting from 3000
nportx completions bash      # Generate shell completions (bash, zsh, fish, powershell)
```

**Note:** The commands accept a port number, not a process ID (PID).

### The Interactive TUI

Run `nportx watch` for a live, color-coded view of your ports.

| Key | Action |
|---|---|
| `↑` / `↓` | Navigate the list |
| `k` | Kill the selected process (confirmation required) |
| `/` | Search by port number or process name |
| `r` | Manually refresh the list |
| `q` | Quit |

The TUI automatically refreshes in the background every two seconds. Change the refresh interval with:

```bash
nportx watch -i 5
```

### Example

List listening ports:

```text
$ nportx list

 PORT       PID  PROCESS                            MEMORY
────────────────────────────────────────────────────────────
  135      1072  svchost.exe                          0 B
  903      3836  fixtional.exe                        0 B
28451      3724  nportx.exe                           0 B
26796      9792  Code.exe                         362.5 MB
42050     12292  OneDrive.Sync.Service.exe         20.7 MB
49690      6408  explorer.exe                     312.5 MB
 5432      3204  [docker] my-postgres-db           89.2 MB
```

Find the process using a port:

```text
$ nportx who 3000

Port 3000 is owned by node.exe (PID 8460)
```

Terminate the process:

```text
$ nportx kill 49690

Kill process explorer.exe (PID 6408) on port 49690? [y/N] y
Killed explorer.exe (PID 6408) on port 49690.
```

Kill by process name:

```text
$ nportx kill --name node

Found 2 process(es) matching 'node':
  - node.exe (PID 8460) on port 3000
  - node.exe (PID 11204) on port 5173
Kill all 2 process(es)? [y/N] y
Killed node.exe (PID 8460) on port 3000.
Killed node.exe (PID 11204) on port 5173.

Done. Successfully killed 2 process(es).
```

Export to JSON:

```bash
nportx export json > ports.json
nportx export csv > ports.csv
```

Find the next free port:

```text
$ nportx free 3000

Port 3000 is taken. Next free port is 3001.
```

Project-aware detection:

```text
$ nportx doctor

🔍 3 expected port(s) in C:\Projects\my-vite-app

PORT    STATUS      EXPECTED BY                              PROCESS
------------------------------------------------------------------------------------------
3000    ✅ listening package.json (scripts.dev)              node.exe (PID 8460)
5173    ✅ listening Vite default                             node.exe (PID 11204)
5432    ❌ free      docker-compose.yml (service: db)         —
```

### Use It in Scripts

**Shell scripts**

Free a port if something is already using it:

```bash
nportx check 3000 || nportx kill 3000 -f
```

**Makefile**

```makefile
dev:
	@nportx check 3000 || nportx kill 3000 -f
	npm run dev
```

Find an open port dynamically:

```bash
PORT=$(nportx free 3000 | grep -oE '[0-9]+$')
npm run dev -- --port $PORT
```

The `check` command returns:

- `0` if the port is free.
- `1` if the port is occupied.

## Features

- ⚡ **Instant startup** — Written in Rust with no runtime dependencies.
- 🌍 **Cross-platform** — Windows, macOS (Intel/ARM), and Linux (x86_64/ARM64).
- 🖥️ **Interactive TUI** — Live-updating table, keyboard navigation, and in-app process termination.
- 🩺 **Project-Aware** — `nportx doctor` reads your `.env`, `package.json`, and `docker-compose.yml` to detect port conflicts.
- 🐳 **Docker Aware** — Replaces docker-proxy with the actual container name.
- 🎨 **Color-coded output** — System processes in red, user processes in green, Docker containers in magenta.
- 🔍 **Instant search** — Filter by port or process name in the TUI.
- 📦 **Exportable** — Dump the port list to JSON or CSV for dashboards and logging.
- 🔌 **UDP Support** — Scan both TCP and UDP sockets with a single flag.
- 🎯 **Next Free Port** — Instantly find the next available port starting from a given number.
- 🛡️ **Safety guard** — Refuses to terminate critical processes such as System, lsass.exe, and csrss.exe.
- 🧪 **Scriptable** — Predictable exit codes, `--force` flags, and JSON/CSV output for automated workflows.

## Roadmap

### Completed

- ☑ list, who, kill, check, and watch commands
- ☑ Interactive TUI (`nportx watch`)
- ☑ Scoop distribution for Windows
- ☑ Homebrew tap for macOS
- ☑ Pre-built binaries for macOS and Linux
- ☑ One-line installation script for Unix-like systems
- ☑ `kill --force` — Skip confirmation
- ☑ Shell completions for Bash, Zsh, Fish, and PowerShell
- ☑ Project-aware port detection (`.env`, `package.json`, framework defaults)
- ☑ Docker container awareness
- ☑ `docker-compose.yml` Detection (host ports for multi-container apps)
- ☑ `kill --name` — Kill all listening ports owned by a specific process name
- ☑ `export` — Output the port list as JSON or CSV
- ☑ `free` — Find the next available free port starting from a given number
- ☑ UDP Support — `--udp` flag to scan TCP + UDP sockets

### Planned

- □ `nportx wait <port>`: Block execution until a port is free. Perfect for scripts that need to wait for a previous process to fully release the port before starting.
- □ `nportx stats`: A live dashboard showing top memory hogs, total listening ports, and historical port usage (like htop for ports).
- □ Linux apt/dnf repos: Native package manager distribution for Debian/Ubuntu and Fedora.
- □ Windows winget support: Submit the release to the official Windows Package Manager repository.
- □ VSCode Extension: A tiny extension that runs nportx doctor in the background and warns you if a port conflict is detected before you run your code.
- □ `nportx kill --interactive`: A guided wizard to pick and choose which processes to kill from a list (like git add -p but for processes).

## Contributing

Issues and pull requests are welcome!

If you encounter a bug, please open an issue and include:

- Your operating system and version
- The output of `nportx list`
- What you expected to happen versus what actually happened

## License

MIT © Nx Conglo