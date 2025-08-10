use anyhow::Context;
use clap::Parser;
use dialoguer::Confirm;
use dl_releases::{
    config::{RepoConfig, get_binaries_path, get_config_path, get_configuration, get_data_path},
    domain::Repository,
    github_client::GithubClient,
    utils::{extract_file_async, get_version},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
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
    /// Output path to extract binaries
    #[arg(short, long)]
    outpath: Option<PathBuf>,
    /// Final binaries location (eg: ~/.local/bin/)
    #[arg(short, long)]
    binaries_location: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args {
        repo,
        pat,
        outpath,
        binaries_location,
    } = Args::parse();
    let config_path = get_config_path().await?;
    let outpath = match outpath {
        Some(x) => x,
        None => get_data_path().await?,
    };
    let binaries_location = match binaries_location {
        Some(x) => x,
        None => get_binaries_path()?,
    };
    match (repo, pat) {
        (None, None) => execute_from_config(config_path, outpath, binaries_location).await?,
        (Some(repo), Some(pat)) => {
            execute_from_args(config_path, outpath, binaries_location, repo, pat).await?;
        }
        _ => {
            anyhow::bail!("`repo` and `pat` should be defined together.");
        }
    };
    Ok(())
}

async fn execute_from_config(
    config_path: PathBuf,
    outpath: PathBuf,
    binaries_location: PathBuf,
) -> anyhow::Result<()> {
    let config = get_configuration(&config_path)?.read_repositories()?;
    let client = GithubClient::new()?;
    let m = MultiProgress::new();
    for (repo, pat) in config {
        if let Err(e) = handle_repo(&m, &client, &repo, &pat, &outpath, &binaries_location).await {
            println!(
                "Failed to handle repo \"{repo}\" with pat=\"{pat}\": {e}\nError details: {e:?}"
            );
        }
    }
    Ok(())
}

async fn execute_from_args(
    config_path: PathBuf,
    outpath: PathBuf,
    binaries_location: PathBuf,
    repo: Repository,
    pat: String,
) -> anyhow::Result<()> {
    let client = GithubClient::new()?;
    let m = MultiProgress::new();
    handle_repo(&m, &client, &repo, &pat, &outpath, &binaries_location)
        .await
        .context("Failed to handle repo")?;
    let mut config = get_configuration(&config_path)?;
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
async fn handle_repo(
    m: &MultiProgress,
    client: &GithubClient,
    repo: &Repository,
    pat: &str,
    outpath: &Path,
    binaries_location: &Path,
) -> anyhow::Result<()> {
    let pb1 = m.add(
        ProgressBar::no_length()
            .with_style(
                ProgressStyle::with_template("{spinner} {prefix} {msg} [{elapsed_precise}] [{wide_bar}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            )
            .with_prefix("[1/3]"),
    );
    let pb2 = m
        .add(
            ProgressBar::new_spinner()
                .with_style(ProgressStyle::with_template("{spinner} {prefix} {wide_msg}").unwrap())
                .with_prefix("[2/3]"),
        )
        .with_message("Waiting to extract file...");
    let pb3 = m
        .add(
            ProgressBar::new_spinner()
                .with_style(ProgressStyle::with_template("{spinner} {prefix} {wide_msg}").unwrap())
                .with_prefix("[3/3]"),
        )
        .with_message("Waiting to check new version...");
    pb2.enable_steady_tick(Duration::from_millis(100));
    pb3.enable_steady_tick(Duration::from_millis(100));
    let current_version = get_version(&repo.repository).await?;
    let release = client
        .get_latest_release(repo)
        .await
        .context("Failed to get latest release.")?;
    let release_version = release.version()?;
    if release_version > current_version {
        let asset = release.find_asset(pat)?;
        pb1.set_length(asset.size);
        let path = client.download_asset(repo, asset, outpath, &pb1).await?;
        pb1.with_style(ProgressStyle::with_template("{msg:.green} {bytes}").unwrap())
            .finish_with_message(format!(
                "✓ [{}] Downloaded to {outpath:?}.",
                repo.repository
            ));
        let extracted_path =
            extract_file_async(path, &repo.repository, binaries_location, &pb2).await?;
        pb2.with_style(ProgressStyle::with_template("{msg:.green}").unwrap())
            .finish_with_message(format!(
                "✓ [{}] Extracted to {extracted_path:?}.",
                repo.repository
            ));
        let extracted_version = get_version(extracted_path).await?;
        if extracted_version != release_version {
            anyhow::bail!(
                "extracted_version ({release_version}) doesn't match the downloaded one ({extracted_version})."
            )
        }
        pb3.with_style(ProgressStyle::with_template("{msg:.green}").unwrap())
            .finish_with_message(format!(
                "✓ [{}] Updated to version {extracted_version}.",
                repo.repository
            ));
        Ok(())
    } else {
        m.remove(&pb2);
        m.remove(&pb3);
        pb1.with_style(ProgressStyle::with_template("{msg:.green}").unwrap())
            .finish_with_message(format!(
                "✓ [{}] is up to date: {current_version}",
                repo.repository
            ));
        Ok(())
    }
}
