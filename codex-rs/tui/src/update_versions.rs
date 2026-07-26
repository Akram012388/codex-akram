pub(crate) fn is_newer(latest: &str, current: &str) -> Option<bool> {
    let latest = semver::Version::parse(latest.trim()).ok()?;
    let current = semver::Version::parse(current.trim()).ok()?;
    Some(latest > current)
}

pub(crate) fn extract_version_from_latest_tag(latest_tag_name: &str) -> anyhow::Result<String> {
    latest_tag_name
        .strip_prefix('v')
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse latest tag name '{latest_tag_name}'"))
}

pub(crate) fn is_source_build_version(version: &str) -> bool {
    parse_version(version) == Some((0, 0, 0))
}

fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let version = semver::Version::parse(v.trim()).ok()?;
    Some((version.major, version.minor, version.patch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn extracts_version_from_latest_tag() {
        assert_eq!(
            extract_version_from_latest_tag("v0.145.0-ak.0.2").expect("failed to parse version"),
            "0.145.0-ak.0.2"
        );
    }

    #[test]
    fn latest_tag_without_prefix_is_invalid() {
        assert!(extract_version_from_latest_tag("rust-v0.145.0").is_err());
    }

    #[test]
    fn fork_prerelease_versions_follow_semver_ordering() {
        assert_eq!(is_newer("0.145.0-ak.0.2", "0.145.0-ak.0.1"), Some(true));
        assert_eq!(is_newer("0.145.0-ak.0.1", "0.145.0"), Some(false));
    }

    #[test]
    fn plain_semver_comparisons_work() {
        assert_eq!(is_newer("0.11.1", "0.11.0"), Some(true));
        assert_eq!(is_newer("0.11.0", "0.11.1"), Some(false));
        assert_eq!(is_newer("1.0.0", "0.9.9"), Some(true));
        assert_eq!(is_newer("0.9.9", "1.0.0"), Some(false));
    }

    #[test]
    fn source_build_version_is_not_checked() {
        assert!(is_source_build_version("0.0.0"));
        assert!(!is_source_build_version("0.1.0"));
    }

    #[test]
    fn whitespace_is_ignored() {
        assert_eq!(parse_version(" 1.2.3 \n"), Some((1, 2, 3)));
        assert_eq!(is_newer(" 1.2.3 ", "1.2.2"), Some(true));
    }
}
