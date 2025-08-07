use crate::domain::{Asset, Release, Repository};
use anyhow::Context;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};

pub struct GithubClient {
    client: Client,
}

impl GithubClient {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .user_agent("dl-releases")
            .build()
            .context("Failed to build client.")?;
        Ok(Self { client })
    }

    pub async fn get_latest_release(&self, repo: &Repository) -> anyhow::Result<Release> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            repo.user, repo.repository
        );
        let raw_response = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        #[cfg(debug_assertions)]
        {
            use tokio::fs::{create_dir, write};

            let s = serde_json::to_string_pretty(&raw_response)?;
            let path = Path::new("raw_outputs");
            if !path.exists() {
                create_dir(path).await?;
            }
            let filename = repo.to_string().replace('/', "_");
            write(path.join(format!("{filename}.json")), s).await?;
        }
        let release = serde_json::from_value(raw_response)?;
        Ok(release)
    }

    pub async fn download_asset(
        &self,
        repo: &Repository,
        asset: &Asset,
        output_path: &Path,
        pb: ProgressBar,
    ) -> anyhow::Result<PathBuf> {
        pb.set_message(asset.name.clone());
        let path = output_path.join(&asset.name);
        if path.exists() {
            pb.with_style(ProgressStyle::with_template("{msg:.green}").unwrap())
                .finish_with_message(format!("✓ File already exists for {}.", repo.repository));
            return Ok(path);
        }
        pb.set_message(format!("Downloading {}", repo.repository));
        let mut file = File::create(&path)
            .await
            .with_context(|| format!("Failed to create file: {path:?}."))?;
        // TODO: use bufwriter
        let response = self.client.get(&asset.browser_download_url).send().await?;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        while let Some(Ok(chunk)) = stream.next().await {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }
        pb.with_style(ProgressStyle::with_template("{msg:.green} {bytes}").unwrap())
            .finish_with_message(format!("✓ Downloaded {}", repo.repository));
        file.flush().await?;
        Ok(path)
    }
}
