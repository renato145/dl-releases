use anyhow::Context;
use flate2::read::GzDecoder;
use indicatif::ProgressBar;
use regex::Regex;
use semver::Version;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::BufWriter,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::LazyLock,
};
use tar::Archive;
use tokio::process::Command;

pub async fn get_version(path: impl AsRef<OsStr>) -> anyhow::Result<Version> {
    let output = Command::new(path)
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

#[derive(Clone, Copy, Debug)]
enum SupportedExtension {
    /// .gz
    Gz,
    /// .tar.gz
    TarGz,
}

impl SupportedExtension {
    fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let extension = path
            .as_ref()
            .file_name()
            .and_then(|x| x.to_str())
            .context("Failed to get file_name.")?;
        if extension.ends_with(".tar.gz") {
            Ok(Self::TarGz)
        } else if extension.ends_with(".gz") {
            Ok(Self::Gz)
        } else {
            anyhow::bail!("File extension not supported.")
        }
    }
}

pub fn extract_file(
    path: impl AsRef<Path>,
    fname: impl AsRef<Path>,
    outpath: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();
    let fname = fname.as_ref();
    let outpath = outpath.as_ref().join(fname);
    let extension = SupportedExtension::from_path(path)?;
    let file = File::open(path).context("Failed to open file.")?;
    match extension {
        SupportedExtension::Gz => {
            let mut decoder = GzDecoder::new(file);
            let output_file = File::create(&outpath).context("Failed to create output file.")?;
            let mut writer = BufWriter::new(output_file);
            std::io::copy(&mut decoder, &mut writer)?;
            set_execute_permission(&outpath)?;
            Ok(outpath)
        }
        SupportedExtension::TarGz => {
            let mut archive = Archive::new(GzDecoder::new(file));
            for entry in archive.entries().context("Failed to read entries.")? {
                let mut entry = entry.context("Failed to read entry.")?;
                let path = entry.path()?;
                let Some(fname_) = path.file_name() else {
                    continue;
                };
                if fname_ == fname {
                    entry.unpack(&outpath)?;
                    set_execute_permission(&outpath)?;
                    return Ok(outpath);
                }
            }
            anyhow::bail!("{fname:?} not found in {path:?}.");
        }
    }
}

fn set_execute_permission(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    fs::set_permissions(&path, perms)?;
    Ok(())
}

pub async fn extract_file_async(
    path: impl AsRef<Path>,
    fname: &str,
    outpath: impl AsRef<Path>,
    pb: &ProgressBar,
) -> anyhow::Result<PathBuf> {
    let path = path.as_ref().to_owned();
    let fname = fname.to_owned();
    let outpath = outpath.as_ref().to_owned();
    pb.set_message(format!("Extracting {path:?} into {outpath:?}..."));
    let outpath = tokio::task::spawn_blocking(move || extract_file(path, fname, outpath))
        .await
        .context("Failed to execute tokio task.")??;
    Ok(outpath)
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use std::fs::read_to_string;
    use tempfile::tempdir;

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

    #[gtest]
    fn extract_file_works() {
        for fname in ["test_file.tar.gz", "test_file.gz"] {
            let outpath = tempdir().unwrap();
            extract_file(format!("src/test_files/{fname}"), "test_file.txt", &outpath).unwrap();
            let content = read_to_string(outpath.as_ref().join("test_file.txt"));
            expect_that!(content, ok(eq("hello\n")));
        }
    }
}
