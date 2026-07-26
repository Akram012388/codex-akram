# codex-app-server-daemon

> `codex-app-server-daemon` is experimental and its lifecycle contract may
> change while the remote-management flow is still being developed.

`codex-app-server-daemon` backs the machine-readable `codex-akram app-server`
lifecycle commands used by remote clients such as the desktop and mobile apps.
It is intended for Codex instances launched over SSH, including fresh developer
machines that should expose app-server with `remote_control` enabled.

## Platform support

The current daemon implementation is Unix-only. It uses pidfile-backed
daemonization plus Unix process and file-locking primitives, and does not yet
support Windows lifecycle management.

## Commands

```sh
codex-akram app-server daemon start
codex-akram app-server daemon restart
codex-akram app-server daemon enable-remote-control
codex-akram app-server daemon disable-remote-control
codex-akram app-server daemon stop
codex-akram app-server daemon version
codex-akram app-server daemon bootstrap --remote-control
```

On success, every command writes exactly one JSON object to stdout. Consumers
should parse that JSON rather than relying on human-readable text. Lifecycle
responses report the resolved backend, socket path, local CLI version, and
running app-server version when applicable.

## Bootstrap flow

For a new remote machine:

```sh
curl -fsSL https://raw.githubusercontent.com/Akram012388/codex-akram/main/scripts/install/install-akram.sh | sh
$HOME/.codex-akram/packages/standalone/current/bin/codex-akram app-server daemon bootstrap --remote-control
```

`bootstrap` requires the standalone managed install. It records the daemon
settings under `CODEX_AKRAM_HOME/app-server-daemon/` and starts app-server as a
pidfile-backed detached process.

## Installation and update cases

The daemon assumes Codex Akram is installed through `install-akram.sh` and
always launches the standalone managed binary under `CODEX_AKRAM_HOME`.

### Standalone installs

For installs created by `install.sh`:

- lifecycle commands always use the standalone managed binary path
- `bootstrap` is supported
- the daemon never downloads or installs updates
- `codex-akram update` is the only supported update entrypoint
- a running app-server adopts an installed update after an explicit restart

### Out-of-band updates

This daemon does not watch arbitrary executable files for replacement. If an
explicit update replaces the managed binary, a currently running app-server
remains on the old executable image until an explicit `restart`.

## Lifecycle semantics

`start` is idempotent and returns after app-server is ready to answer the normal
JSON-RPC initialize handshake on the Unix control socket.

`restart` stops any managed daemon and starts it again.

`enable-remote-control` and `disable-remote-control` persist the launch setting
for future starts. If a managed app-server is already running, they restart it
so the new setting takes effect immediately.

Top-level `codex-akram remote-control` bootstraps with `--remote-control` when
daemon settings do not exist. Otherwise it enables remote control and starts
the daemon normally.

`stop` sends a graceful termination request first, then sends a second
termination signal after the grace window if the process is still alive.

All mutating lifecycle commands are serialized per `CODEX_AKRAM_HOME`, so a concurrent
`start`, `restart`, `enable-remote-control`, `disable-remote-control`, `stop`,
or `bootstrap` does not race another in-flight lifecycle operation.

## State

The daemon stores its local state under `CODEX_AKRAM_HOME/app-server-daemon/`:

- `settings.json` for persisted launch settings
- `app-server.pid` for the app-server process record
- `daemon.lock` for daemon-wide lifecycle serialization
