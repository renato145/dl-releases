use crate::domain::Repository;
use anyhow::Context;
use config::Config;
use directories::BaseDirs;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::fs::{create_dir, write};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub repos: Vec<RepoConfig>,
}

impl Configuration {
    pub fn read_repositories(self) -> anyhow::Result<Vec<(Repository, String)>> {
        self.repos
            .into_iter()
            .map(|o| Repository::from_str(&o.repo).map(|repo| (repo, o.pat)))
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn validate(self) -> anyhow::Result<Self> {
        let duplicated_repos = self
            .repos
            .iter()
            .counts_by(|o| &o.repo)
            .into_iter()
            .filter_map(|(o, n)| (n > 1).then_some(o))
            .collect::<Vec<_>>();
        if !duplicated_repos.is_empty() {
            let s = duplicated_repos.into_iter().join(", ");
            anyhow::bail!("Found duplicated repos on config: {s}.");
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepoConfig {
    /// Repository name in format user/repo_name
    pub repo: String,
    /// Pattern to look in into assets to pick the one to download
    pub pat: String,
}

pub async fn get_config_path() -> anyhow::Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("No valid home directory path found.")?;
    let parent = base_dirs.config_dir().join("dl-releases");
    if !parent.exists() {
        create_dir(&parent)
            .await
            .context("Failed to create directory.")?;
    }
    let path = parent.join("config.toml");
    if !path.exists() {
        let config = Configuration { repos: Vec::new() };
        let s = toml::to_string_pretty(&config).context("Failed to serialize config.")?;
        write(&path, s).await.context("Failed to write to file.")?;
    }
    Ok(path)
}

pub async fn get_data_path() -> anyhow::Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("No valid home directory path found.")?;
    let path = base_dirs.data_dir().join("dl-releases");
    if !path.exists() {
        create_dir(&path)
            .await
            .context("Failed to create directory.")?;
    }
    Ok(path)
}

pub fn get_binaries_path() -> anyhow::Result<PathBuf> {
    let base_dirs = BaseDirs::new().context("No valid home directory path found.")?;
    let path = base_dirs
        .executable_dir()
        .context("No executable dir found.")?
        .to_owned();
    Ok(path)
}

pub async fn get_configuration(path: &Path) -> anyhow::Result<Configuration> {
    Config::builder()
        .add_source(config::File::from(path))
        .build()?
        .try_deserialize::<Configuration>()
        .context("Failed to deserialize configuration.")?
        .validate()
}
