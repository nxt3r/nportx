# NportX

> Find and kill processes hogging your ports. Instantly. Cross-platform. Single binary.

## Table of Contents

- [Why Not `lsof` or `netstat`?](#why-not-lsof-or-netstat)
- [Install](#install)
  - [Windows (Recommended: Scoop)](#windows-recommended-scoop)
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
| No runtime dependencies | ✅ | ✅ | ✅ |
| Instant startup | ✅ | ✅ | ✅ |

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
scoop update nportx
```

### macOS / Linux (One-Line Script)

```bash
curl -sSL https://raw.githubusercontent.com/nxt3r/nportx/main/install.sh | bash
```

The script detects your operating system and architecture (Intel/ARM), downloads the latest pre-built binary, and installs it to `/usr/local/bin` or `~/.local/bin`.

### Manual Windows Install

If you don't want to use Scoop:

1. Download `nportx-x86_64-pc-windows-msvc.zip` from the [Releases](https://github.com/nxt3r/nportx/releases) page.
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
nportx watch       # Open the interactive TUI
nportx list        # Show all listening TCP ports
nportx who 3000    # Find the process using port 3000
nportx kill 3000   # Kill the process using port 3000 (with confirmation)
nportx check 3000  # Exit 0 if free, 1 if taken — for scripts
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

PORT   PROCESS                   PID    MEM
----------------------------------------------------
135    svchost.exe               1072   0 MB
903    vmware-authd.exe          3832   0 MB
28451  nportx.exe                3776   0 MB
49690  explorer.exe              6408   252 MB
65092  services.exe              912    0 MB
```

Find the process using a port:

```text
$ nportx who 49690

Port 49690 is held by PID 6408 (explorer.exe, 252 MB)
```

Terminate the process:

```text
$ nportx kill 49690

About to kill: explorer.exe (PID 6408) on port 49690
Are you sure? [y/N] y

✅ Killed explorer.exe (PID 6408) — port 49690 is now free.
```

### Use It in Scripts

**Shell scripts**

Free a port if something is already using it:

```bash
nportx check 3000 || nportx kill 3000
```

**Makefile**

```makefile
dev:
	@nportx check 3000 || nportx kill 3000
	npm run dev
```

The `check` command returns:

- `0` if the port is free.
- `1` if the port is occupied.

This makes it suitable for shell pipelines and automated development workflows.

## Features

- ⚡ **Instant startup** — Written in Rust with no runtime dependencies.
- 🌍 **Cross-platform** — Windows, macOS (Intel/ARM), and Linux (x86_64/ARM64).
- 🖥️ **Interactive TUI** — Live-updating table, keyboard navigation, and in-app process termination.
- 🎨 **Color-coded output** — System processes in red, user processes in green.
- 🔍 **Instant search** — Filter by port or process name in the TUI.
- 🛡️ **Safety guard** — Refuses to terminate critical processes such as `System`, `lsass.exe`, and `csrss.exe`.
- 🧪 **Scriptable** — Predictable exit codes for automated workflows.

## Roadmap

### Completed

- [x] `list`, `who`, `kill`, `check`, and `watch` commands
- [x] Scoop distribution for Windows
- [x] Pre-built binaries for macOS and Linux
- [x] One-line installation script for Unix-like systems
- [x] Interactive TUI (`nportx watch`)

### Planned

- [ ] `kill --force` — Skip confirmation
- [ ] Shell completions for Bash, Zsh, Fish, and PowerShell
- [ ] Homebrew tap for macOS
- [ ] Project-aware port detection (`.env`, `package.json`, `docker-compose.yml`)
- [ ] Docker container awareness (`docker-proxy` → container name)

## Contributing

Issues and pull requests are welcome!

If you encounter a bug, please open an issue and include:

- Your operating system and version
- The output of `nportx list`
- What you expected to happen versus what actually happened

## License

MIT © Nx Conglo