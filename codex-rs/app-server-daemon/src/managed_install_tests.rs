use pretty_assertions::assert_eq;

use super::managed_codex_bin;
use super::parse_codex_version;

#[test]
fn parses_codex_cli_version_output() {
    assert_eq!(
        parse_codex_version("codex-akram 0.145.0-ak.0.1\n").expect("version"),
        "0.145.0-ak.0.1"
    );
}

#[test]
fn rejects_malformed_codex_cli_version_output() {
    assert!(parse_codex_version("codex\n").is_err());
}

#[test]
fn managed_binary_uses_fork_package_entrypoint() {
    let expected = if cfg!(windows) {
        "state/packages/standalone/current/bin/codex-akram.exe"
    } else {
        "state/packages/standalone/current/bin/codex-akram"
    };

    assert_eq!(
        managed_codex_bin(std::path::Path::new("state")),
        std::path::PathBuf::from(expected)
    );
}
