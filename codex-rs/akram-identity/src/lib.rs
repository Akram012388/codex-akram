//! Runtime identity for the independent codex-akram fork.

pub const CLI_NAME: &str = "codex-akram";
pub const DISPLAY_NAME: &str = "Codex Akram";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const HOME_ENV_VAR: &str = "CODEX_AKRAM_HOME";
pub const SQLITE_HOME_ENV_VAR: &str = "CODEX_AKRAM_SQLITE_HOME";
pub const DEFAULT_HOME_DIR: &str = ".codex-akram";
pub const PROJECT_OVERLAY_DIR: &str = ".codex-akram";
pub const RUNTIME_CACHE_DIR: &str = "codex-akram-runtimes";

pub const AUTH_KEYRING_SERVICE: &str = "Codex Akram Auth";
pub const SECRETS_KEYRING_SERVICE: &str = "codex-akram";

pub const GITHUB_REPOSITORY: &str = "Akram012388/codex-akram";
pub const GITHUB_LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/Akram012388/codex-akram/releases/latest";
pub const GITHUB_LATEST_RELEASE_URL: &str =
    "https://github.com/Akram012388/codex-akram/releases/latest";
pub const INSTALLER_URL: &str = concat!(
    "https://raw.githubusercontent.com/Akram012388/codex-akram/",
    "main/scripts/install/install-akram.sh"
);
pub const INSTALLER_COMMAND: &str = concat!(
    "curl -fsSL https://raw.githubusercontent.com/Akram012388/codex-akram/",
    "main/scripts/install/install-akram.sh | CODEX_AKRAM_NON_INTERACTIVE=1 sh"
);
