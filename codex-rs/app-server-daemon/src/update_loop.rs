use anyhow::Result;
use anyhow::bail;

pub(crate) async fn run() -> Result<()> {
    bail!("automatic updates are disabled; run `codex-akram update` explicitly")
}
