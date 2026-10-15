# nportx

> Find and kill processes hogging your ports. Instantly. Cross-platform. Single binary.

"Port 3000 is already in use" — every developer, every day. `lsof` is ugly. `netstat` is painful. Python tools are slow.

**nportx** is a single Rust binary that shows you what's on every port, and kills it when you're done.

## Install

```bash
cargo install nportx
```

## Usage

```bash
nportx list          # show all listening TCP ports
nportx who 3000      # what's on port 3000?
nportx kill 3000     # kill whatever is on port 3000 (with confirmation)
```

### Example

```
$ nportx list
PORT     PROCESS                  PID      MEM
----------------------------------------------------
135      svchost.exe              1072     0 MB
3000     node.exe                 48213    142 MB
5432     postgres.exe             1204     54 MB
65092    services.exe             912      0 MB
```

## Features

- ⚡ Instant startup — Rust, no runtime dependencies
- 🌍 macOS, Linux, Windows
- 🎨 Color-coded output — system processes in red, yours in green
- 🛡️ Safety guard against killing critical system processes
- 📦 Single binary (~2 MB)

## Roadmap

- [x] `list`, `who`, `kill`
- [ ] `check <port>` — exit 0 if free, 1 if taken
- [ ] `watch` — live-updating TUI
- [ ] Interactive TUI mode
- [ ] Project-aware port detection
- [ ] Shell completions

## License

MIT