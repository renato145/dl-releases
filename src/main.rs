use anyhow::Context;
use clap::Parser;
use console::style;
use dialoguer::Confirm;
use dl_releases::{
    config::{RepoConfig, get_config_path, get_configuration, get_data_path},
    domain::Repository,
    github_client::GithubClient,
    utils::get_version,
};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use std::path::PathBuf;
use tokio::fs::write;

// TODO: add option to show release changelog

/// Personal utility to download and install binaries from git releases
#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    /// Repository name in format user/repo_name
    #[arg(short, long)]
    repo: Option<Repository>,
    /// Pattern to look in into assets to pick the one to download
    #[arg(short, long)]
    pat: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = get_config_path().await?;
    let Args { repo, pat } = Args::parse();

    match (repo, pat) {
        (None, None) => execute_from_config(config_path).await?,
        (Some(repo), Some(pat)) => {
            execute_from_args(config_path, repo, pat).await?;
        }
        _ => {
            anyhow::bail!("`repo` and `pat` should be defined together.");
        }
    };
    Ok(())
}

async fn execute_from_config(config_path: PathBuf) -> anyhow::Result<()> {
    let config = get_configuration(&config_path).await?.read_repositories()?;
    let client = GithubClient::new()?;
    for (repo, pat) in config {
        if let Err(e) = handle_repo(&client, &repo, &pat).await {
            println!(
                "Failed to handle repo \"{repo}\" with pat=\"{pat}\": {e}\nError details: {e:?}"
            );
        }
    }
    Ok(())
}

async fn execute_from_args(
    config_path: PathBuf,
    repo: Repository,
    pat: String,
) -> anyhow::Result<()> {
    let client = GithubClient::new()?;
    handle_repo(&client, &repo, &pat)
        .await
        .context("Failed to handle repo")?;
    let mut config = get_configuration(&config_path).await?;
    let repo = repo.to_string();
    if config.repos.iter().map(|o| &o.repo).contains(&repo) {
        return Ok(());
    }
    let add_to_config = Confirm::new()
        .with_prompt(format!(
            "Do you want to add this repository/pattern to your config file ({config_path:?})?"
        ))
        .interact()
        .unwrap();
    if add_to_config {
        config.repos.push(RepoConfig {
            repo: repo.clone(),
            pat,
        });
        let s = toml::to_string_pretty(&config).context("Failed to serialize config.")?;
        write(&config_path, s)
            .await
            .context("Failed to write to file.")?;
        println!("Added {repo} to {config_path:?}");
    }
    Ok(())
}

/// Downloads the last release and installs it if required
async fn handle_repo(client: &GithubClient, repo: &Repository, pat: &str) -> anyhow::Result<()> {
    let current_version = get_version(&repo.repository).await?;
    let release = client
        .get_latest_release(repo)
        .await
        .context("Failed to get latest release.")?;
    let release_version = release.version()?;
    if release_version > current_version {
        let asset = release.find_asset(pat)?;
        let output_path = get_data_path().await?;
        let pb = ProgressBar::new(asset.size);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner} {msg} [{elapsed_precise}] [{wide_bar}] {bytes}/{total_bytes} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        let _path = client.download_asset(repo, asset, &output_path, pb).await?;
        // TODO: unpack download
        // TODO: install bin
    } else {
        let s = style(format!("✓ {} is up to date", repo.repository)).green();
        println!("{s}");
    }
    Ok(())
}
