# NportX

> Find and kill processes hogging your ports. Instantly. Cross-platform. Single binary.

"Port 3000 is already in use" — every developer, every day.

`lsof` is ugly. `netstat` is painful. Python tools are slow. **nportx** is a single Rust binary that shows you what's on every port, and kills it when you're done.

## Install

```bash
cargo install nportx
```

## Usage

```
nportx              # launch TUI (coming soon)
nportx list         # show all listening TCP ports
nportx who 3000     # what's on port 3000?
nportx kill 3000    # kill whatever is on port 3000 (with confirmation)
nportx check 3000   # exit 0 if free, 1 if taken (for scripts)
nportx watch        # live-updating view, refreshes every 2s
nportx watch -i x   # delay it to refresh every xs (x seconds)
```

### Example

```
$ nportx list
PORT     PROCESS                  PID      MEM
----------------------------------------------------
135      svchost.exe              1072     0 MB
903      vmware-authd.exe         3832     0 MB
28451    MonectServerService.exe  3776     0 MB
49690    explorer.exe             6408     252 MB
65092    services.exe             912      0 MB
```

```
$ nportx who 65092
Port 65092 is held by PID 912 (services.exe, 0 MB)
```

```
$ nportx kill 65092
About to kill: services.exe (PID 912) on port 65092
Are you sure? [y/N] y
✅ Killed services.exe (PID 912) — port 65092 is now free.
```

### Use it in scripts

```bash
# Free a port if something's holding it
nportx check 3000 || nportx kill 3000

# In a Makefile
dev:
	@nportx check 3000 || nportx kill 3000
	npm run dev
```

`check` returns **exit code 0** if the port is free and **1** if it's taken — designed for shell pipelines.

## Features

- ⚡ Instant startup — Rust, no runtime dependencies
- 🌍 Cross-platform — macOS, Linux, Windows (Windows verified, others in progress)
- 🎨 Color-coded output — system processes in red, user processes in green
- 🛡️ Safety guard — refuses to kill system-critical processes (System, lsass, csrss, etc.)
- 👀 `watch` mode — live view, refreshes every N seconds
- 📦 Single binary, ~855 KB

## Roadmap

- [x] `list`, `who`, `kill`, `check`, `watch`
- [ ] Interactive TUI (ratatui)
- [ ] `kill --force` — no confirmation prompt
- [ ] Shell completions (bash, zsh, fish, PowerShell)
- [ ] Project-aware port detection (`.env`, `package.json`, `docker-compose.yml`)
- [ ] macOS + Linux testing

## License

MIT © 2026 Nx Conglo