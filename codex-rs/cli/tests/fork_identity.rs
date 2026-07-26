use std::fs;

use anyhow::Result;
use pretty_assertions::assert_eq;

#[test]
fn version_reports_fork_identity() -> Result<()> {
    let output = std::process::Command::new(codex_utils_cargo_bin::cargo_bin("codex-akram")?)
        .arg("--version")
        .output()?;

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout)?,
        "codex-akram 0.145.0-ak.0.1\n"
    );
    Ok(())
}

#[test]
fn fork_home_ignores_official_state_environment_variables() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let official_home = temp.path().join("official");
    let fork_home = temp.path().join("fork");
    let official_sqlite_home = temp.path().join("official-sqlite");
    let fork_sqlite_home = temp.path().join("fork-sqlite");
    fs::create_dir_all(&official_home)?;
    fs::create_dir_all(&fork_home)?;
    fs::create_dir_all(&official_sqlite_home)?;
    fs::create_dir_all(&fork_sqlite_home)?;
    fs::write(official_home.join("auth.json"), "official sentinel")?;
    fs::write(
        official_sqlite_home.join("state_5.sqlite"),
        "official sentinel",
    )?;

    let output = std::process::Command::new(codex_utils_cargo_bin::cargo_bin("codex-akram")?)
        .args(["login", "status"])
        .env("CODEX_HOME", &official_home)
        .env("CODEX_SQLITE_HOME", &official_sqlite_home)
        .env("CODEX_AKRAM_HOME", &fork_home)
        .env("CODEX_AKRAM_SQLITE_HOME", &fork_sqlite_home)
        .output()?;

    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(official_home.join("auth.json"))?,
        "official sentinel"
    );
    assert_eq!(
        fs::read_to_string(official_sqlite_home.join("state_5.sqlite"))?,
        "official sentinel"
    );
    assert!(!fork_home.join("auth.json").exists());
    Ok(())
}
