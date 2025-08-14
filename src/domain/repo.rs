//! GitHub repository domain model.
//!
//! This module provides the [`Repository`] struct for parsing and representing
//! GitHub repositories in the format `user/repository_name`.

use anyhow::Context;
use std::{fmt::Display, str::FromStr};

/// Represents a GitHub repository with user and repository name components.
///
/// The repository follows the standard GitHub format: `user/repository_name`.
/// It provides parsing from string format and display formatting.
#[derive(Clone, Debug)]
pub struct Repository {
    /// The GitHub username or organization name
    pub user: String,
    /// The repository name
    pub repository: String,
}

impl FromStr for Repository {
    type Err = anyhow::Error;

    /// Parses a repository string in the format "user/repository".
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (user, repository) = s.split_once('/').context("No delimiter '/' on input.")?;
        if user.contains('/') || repository.contains('/') {
            anyhow::bail!("Invalid input.")
        }
        Ok(Repository {
            user: user.to_string(),
            repository: repository.to_string(),
        })
    }
}

impl Display for Repository {
    /// Formats the repository as "user/repository".
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.user, self.repository)
    }
}
