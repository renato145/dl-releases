//! GitHub release domain models.
//!
//! This module provides structures for working with GitHub releases and assets,
//! including functionality to find assets by substring patterns and extract version information.

use crate::utils::extract_version;
use jiff::Timestamp;
use semver::Version;
use serde::Deserialize;

/// Represents a GitHub release with metadata and associated assets.
#[derive(Clone, Debug, Deserialize)]
pub struct Release {
    /// The git tag name for this release
    pub tag_name: String,
    /// Release description/notes
    pub body: String,
    /// When the release was created
    pub created_at: Timestamp,
    /// List of downloadable assets for this release
    pub assets: Vec<Asset>,
}

/// Errors that can occur when finding assets in a release.
#[derive(Debug, thiserror::Error)]
pub enum FindAssetError {
    #[error("No asset found for pattern: {0:?}.")]
    NoAsset(String),
    #[error("Found {} assets for the same pattern ({pat:?}): {assets:#?}.", .assets.len())]
    ManyAssets { pat: String, assets: Vec<String> },
}

impl Release {
    /// Find asset based on a substring pattern.
    ///
    /// Searches for assets whose name contains the given pattern (case-insensitive).
    pub fn find_asset(&self, pat: &str) -> Result<&Asset, FindAssetError> {
        let res = self
            .assets
            .iter()
            .filter(|o| o.name.to_lowercase().contains(pat))
            .collect::<Vec<_>>();
        if res.is_empty() {
            return Err(FindAssetError::NoAsset(pat.to_string()));
        }
        if res.len() > 1 {
            let assets = res.into_iter().map(|o| o.name.clone()).collect();
            return Err(FindAssetError::ManyAssets {
                assets,
                pat: pat.to_string(),
            });
        }
        Ok(res[0])
    }

    /// Extracts the semantic version from the release tag name or body.
    pub fn version(&self) -> anyhow::Result<Version> {
        match extract_version(&self.tag_name) {
            Ok(version) => Ok(version),
            _ => extract_version(&self.body),
        }
    }
}

/// Represents a downloadable asset from a GitHub release.
#[derive(Clone, Debug, Deserialize)]
pub struct Asset {
    /// The filename of the asset
    pub name: String,
    /// Direct download URL for the asset
    pub browser_download_url: String,
    /// File size in bytes
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use std::fs::read_to_string;

    #[gtest]
    fn find_linux_asset_works() {
        let cases = [
            (
                "jesseduffield_lazydocker",
                "linux_x86_64",
                "lazydocker_0.24.1_Linux_x86_64.tar.gz",
            ),
            (
                "jesseduffield_lazygit",
                "linux_x86_64",
                "lazygit_0.54.1_linux_x86_64.tar.gz",
            ),
            (
                "rust-lang_rust-analyzer",
                "x86_64-unknown-linux-gnu",
                "rust-analyzer-x86_64-unknown-linux-gnu.gz",
            ),
        ];
        for (name, pat, expected) in cases {
            let s = read_to_string(format!("src/domain/test_files/{name}.json")).unwrap();
            let release = serde_json::from_str::<Release>(&s).unwrap();
            let asset = release.find_asset(pat);
            expect_that!(asset, ok(field!(&Asset.name, eq(expected))));
        }
    }

    #[gtest]
    fn get_version_works() {
        let cases = [
            ("jesseduffield_lazydocker", "0.24.1"),
            ("jesseduffield_lazygit", "0.54.1"),
            ("rust-lang_rust-analyzer", "0.3.2563"),
        ];
        for (name, expected) in cases {
            let s = read_to_string(format!("src/domain/test_files/{name}.json")).unwrap();
            let release = serde_json::from_str::<Release>(&s).unwrap();
            let version = release.version();
            let expected = Version::parse(expected).unwrap();
            expect_that!(version, ok(eq(&expected)), "Failed for {name}");
        }
    }
}
