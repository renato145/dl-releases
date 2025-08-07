use anyhow::Context;
use std::{fmt::Display, str::FromStr};

#[derive(Clone, Debug)]
pub struct Repository {
    pub user: String,
    pub repository: String,
}

impl FromStr for Repository {
    type Err = anyhow::Error;

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.user, self.repository)
    }
}
