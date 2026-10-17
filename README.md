# NportX

> Find and kill processes hogging your ports. Instantly. Cross-platform. Single binary.

## Table of Contents

- [Why not `lsof` or `netstat`?](#why-not-lsof-or-netstat)
- [Install](#install)
  - [Windows (recommended: Scoop)](#windows-recommended-scoop)
  - [Manual Windows install](#manual-windows-install)
  - [macOS / Linux (via Cargo)](#macos--linux-via-cargo)
- [Usage](#usage)
  - [Example](#example)
  - [Use it in scripts](#use-it-in-scripts)
- [Features](#features)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

"Port 3000 is already in use" — every developer, every day.

`lsof` is ugly. `netstat` is painful. Python tools are slow. **NportX** is a single ~896 KB Rust binary that shows you what's on every port and kills it when you're done.

## Why not `lsof` or `netstat`?

| | lsof | netstat | **NportX** |
|---|:---:|:---:|:---:|
| Cross-platform | ❌ | ❌ | ✅ |
| Human-readable output | ❌ | ❌ | ✅ |
| Kill by port | ❌ | ❌ | ✅ |
| Scripting exit codes | ❌ | ❌ | ✅ |
| No runtime dependencies | ✅ | ✅ | ✅ |
| Instant startup | ✅ | ✅ | ✅ |

## Install

### Windows (recommended: Scoop)

```powershell
# First time using Scoop? Install it (one-time setup, requires PowerShell)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
scoop install git

# Then install NportX
scoop bucket add nportx https://github.com/nxt3r/scoop-nportx
scoop install nportx
```

Scoop handles PATH, updates, and uninstallation automatically. To upgrade later: `scoop update nportx`.

### Manual Windows install

If you don't want Scoop:

1. Download nportx.exe from Releases
2. Move it to `%USERPROFILE%\bin\` (create the folder if it doesn't exist)
3. Add `%USERPROFILE%\bin` to your PATH via Environment Variables in Windows settings
4. Restart PowerShell

### macOS / Linux (via Cargo)

```bash
cargo install nportx
```

If you don't have Rust: install it via rustup (one-time, ~5 minutes).

Pre-built binaries for macOS (Intel/ARM64) and Linux (x86_64/ARM64) are coming in a future release.

## Usage

```text
nportx list       # show all listening TCP ports
nportx who 3000   # what's on port 3000? (3000 = the PORT, not the PID)
nportx kill 3000  # kill whatever is on port 3000 (with confirmation)
nportx check 3000 # exit 0 if free, 1 if taken — for scripts
nportx watch      # live view, refreshes every 2s
nportx watch -i x # refresh every x seconds
```

### Example

```text
$ nportx list
PORT  PROCESS                   PID   MEM
----------------------------------------------------
135   svchost.exe               1072  0 MB
903   vmware-authd.exe          3832  0 MB
28451 MonectServerService.exe   3776  0 MB
49690 explorer.exe              6408  252 MB
65092 services.exe              912   0 MB
```

```text
$ nportx who 65092
Port 65092 is held by PID 912 (services.exe, 0 MB)
```

```text
$ nportx kill 65092
About to kill: services.exe (PID 912) on port 65092
Are you sure? [y/N] y
✅ Killed services.exe (PID 912) — port 65092 is now free.
```

```text
$ nportx check 3000
Port 3000 is free.
$ echo $?
0
```

### Use it in scripts

```bash
# Free a port if something's holding it
nportx check 3000 || nportx kill 3000
```

```makefile
# In a Makefile
dev:
	@nportx check 3000 || nportx kill 3000
	npm run dev
```

`check` returns exit code 0 if the port is free and 1 if it's taken — designed for shell pipelines.

## Features

- ⚡ Instant startup — Rust, no runtime dependencies
- 🌍 Cross-platform — Windows verified; macOS/Linux builds in progress
- 🎨 Color-coded output — system processes in red, user processes in green
- 🛡️ Safety guard — refuses to kill System, lsass.exe, csrss.exe, and other critical processes
- 👀 watch mode — live view, refreshes on an interval
- 🧪 Scriptable — check returns clean exit codes
- 📦 Single binary — ~896 KB, no install wizard, no dependencies

## Roadmap

- ☑ list, who, kill, check, watch
- ☑ Scoop distribution for Windows
- □ Interactive TUI mode (nportx with no args)
- □ kill --force — skip confirmation
- □ Shell completions (bash, zsh, fish, PowerShell)
- □ Pre-built binaries for macOS + Linux
- □ Homebrew tap for macOS
- □ Project-aware port detection (.env, package.json, docker-compose.yml)
- □ Docker container awareness (docker-proxy → container name)

## Contributing

Issues and PRs welcome. If you hit a bug on macOS or Linux, please open an issue with:

- Your OS and version
- The output of nportx list
- What you expected vs. what happened

## License

MIT © Nx Conglo