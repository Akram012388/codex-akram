# codex-akram

codex-akram is an independent, opinionated fork of the open source OpenAI Codex CLI. It is maintained by Akram, is not affiliated with or endorsed by OpenAI, and does not use OpenAI logos or visual identity.

The initial release is based on upstream `rust-v0.145.0` at commit `25af12f7`. It keeps OpenAI service compatibility while providing a separate executable, runtime identity, state directory, authentication store, package channel, and update path.

## Install

The initial release supports Apple Silicon Macs. It is unsigned and every release archive is accompanied by SHA-256 checksums.

```shell
curl -fsSL https://raw.githubusercontent.com/Akram012388/codex-akram/main/scripts/install/install-akram.sh | sh
```

The installer adds `~/.local/bin/codex-akram` and stores managed releases under `~/.codex-akram`. It does not install a `codex` alias or import `~/.codex`.

Run:

```shell
codex-akram
```

Use `codex-akram update` to explicitly install a newer codex-akram release when one is available.

## Isolation and project configuration

User state defaults to `~/.codex-akram`. Override it with `CODEX_AKRAM_HOME`, and override SQLite storage with `CODEX_AKRAM_SQLITE_HOME`. The official `CODEX_HOME` and `CODEX_SQLITE_HOME` variables do not redirect this fork.

Trusted repositories load standard `.codex` project configuration first, then apply `.codex-akram` as the higher-priority overlay. Both directories remain behind the existing project trust gate.

## Build

```shell
cd codex-rs
cargo build --bin codex-akram
```

Repository development follows the checks documented in [AGENTS.md](AGENTS.md).

## Attribution and license

This fork retains the upstream [Apache-2.0 License](LICENSE) and [NOTICE](NOTICE). OpenAI, Codex, and related names are the property of their respective owners.
