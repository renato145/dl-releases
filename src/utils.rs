use anyhow::Context;
use regex::Regex;
use semver::Version;
use std::sync::LazyLock;
use tokio::process::Command;

pub async fn get_version(bin_name: &str) -> anyhow::Result<Version> {
    let output = Command::new(bin_name)
        .arg("--version")
        .output()
        .await
        .context("Failed to execute command.")?;
    if !output.status.success() {
        anyhow::bail!("Failed to execute command.");
    }
    let s = String::from_utf8(output.stdout).context("Failed to read output.")?;
    extract_version(&s)
}

pub fn extract_version(s: &str) -> anyhow::Result<Version> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+\.\d+\.\d+)").unwrap());
    let version = RE
        .captures(s)
        .and_then(|o| o.get(1))
        .with_context(|| format!("No version found on: {s:?}."))?
        .as_str();
    Version::parse(version).context("Failed to parse version")
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use std::fs::read_to_string;

    #[gtest]
    fn parse_version_works() {
        for (o, expected) in [
            ("lazydocker", "0.24.1"),
            ("lazygit", "0.50.0"),
            ("rust-analyzer", "0.3.2555"),
        ] {
            let s = read_to_string(format!("src/test_files/{o}_example.txt")).unwrap();
            let version = extract_version(&s);
            let expected = Version::parse(expected).unwrap();
            expect_that!(version, ok(eq(&expected)), "Failed for {o}");
        }
    }
}
