//! This crate is for handling game resources.
pub mod avatar;
pub mod level;
pub mod manifest;
mod utils;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    FetchOptions, ProxyOptions, RemoteCallbacks,
};
use manifest::Manifest;
use tracing::{info, warn};

// https://docs.github.com/en/rest/repos/contents
const RESOURCE_ROOT_URL: &str =
    "https://api.github.com/repos/AzurIce/azur-arknights-helper/contents/resources";

async fn fetch_manifest() -> Result<Manifest, anyhow::Error> {
    let client = reqwest::Client::builder()
        .user_agent("azur-arknights-helper")
        .build()?;
    let resp = client
        .get(format!("{RESOURCE_ROOT_URL}/manifest.toml"))
        .header("Accept", "application/vnd.github.raw+json")
        .send()
        .await?;
    let version: Manifest = toml::from_str(&resp.text().await?)?;

    Ok(version)
}

const RESOURCE_REPO: &str = "https://github.com/AzurIce/azur-arknights-helper.git";

fn fetch_options() -> FetchOptions<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|stats| {
        info!(
            "Transfer progress: objects: {}/{}, deltas: {}/{}",
            stats.received_objects(),
            stats.total_objects(),
            stats.indexed_deltas(),
            stats.total_deltas()
        );
        true
    });

    let mut proxy_options = ProxyOptions::new();
    proxy_options.auto();

    let mut fetch_options = FetchOptions::new();
    fetch_options
        .remote_callbacks(callbacks)
        .proxy_options(proxy_options)
        .depth(1);
    fetch_options
}

/// A struct for local Maa Resource Dir
///
/// default should be at `./.aah/MaaResource`
#[derive(Debug)]
pub struct Resource {
    pub repo_root: PathBuf,
    pub root: PathBuf,
    pub manifest: Manifest,
}

impl Resource {
    /// Try to initialize resource from a path
    ///
    /// open the repo and checkout to the latest main
    /// if cannot open, clone the repo to the path, then checkout to the latest main
    pub async fn try_init(root: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        info!("Opening resource repo...");
        let repo = match git2::Repository::open(&root) {
            Ok(repo) => repo,
            Err(e) => {
                info!("Failed to open resource root: {e}, cloning...");
                let mut builder = RepoBuilder::new();
                builder.fetch_options(fetch_options());
                builder
                    .clone(RESOURCE_REPO, root.as_ref())
                    .context("clone resource repo")?
            }
        };

        info!("Checking out head...");
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .context("checkout head")?;

        let mut resource = Self::load(root.as_ref())?;
        resource.update().await?;
        Ok(resource)
    }

    /// Load resource from a repo root of [MaaResource](https://github.com/MaaAssistantArknights/MaaResource)
    pub fn load(repo_root: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let repo_root = repo_root.as_ref().to_owned();
        let root = repo_root.join("resources");

        if !root.exists() {
            return Err(anyhow::anyhow!("Resource root not found"));
        }

        let manifest = fs::read_to_string(root.join("manifest.toml"))?;
        let manifest = toml::from_str(&manifest)?;

        Ok(Self {
            repo_root,
            root,
            manifest,
        })
    }

    /// Update resource to the latest version
    ///
    /// this will not do anything if the version.json is unchanged
    /// when the version is updated, it'll fetch the origin and checkout to latest main
    pub async fn update(&mut self) -> Result<(), anyhow::Error> {
        let manifest = fetch_manifest().await?;
        if self.manifest.last_updated == manifest.last_updated {
            info!("Resource is up to date");
            return Ok(());
        }

        let repo = git2::Repository::open(&self.repo_root)?;

        info!("Fetching origin...");
        let mut fetch_options = fetch_options();
        repo.find_remote("origin")?
            .fetch(&["main"], Some(&mut fetch_options), Some("update"))?;

        info!("Checking out main...");
        repo.set_head("refs/remotes/origin/main")?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))?;

        *self = Self::load(&self.repo_root)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    use super::*;

    fn init_logger() {
        // let indicatif_layer = IndicatifLayer::new();

        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(
                tracing_subscriber::fmt::layer(), // .with_level(false)
                                                  // .with_target(false)
                                                  // .without_time()
                                                  // .with_writer(indicatif_layer.get_stderr_writer()),
            )
            // .with(indicatif_layer)
            .init();
    }

    #[tokio::test]
    async fn test_try_initialize_resource() {
        init_logger();

        let resource = Resource::try_init("./test/.aah/resources").await.unwrap();
        println!("{:?}", resource.manifest);
    }

    #[tokio::test]
    async fn test_update_resource() {
        init_logger();

        let mut resource = Resource::load("./test/.aah/resources").unwrap();
        resource.update().await.unwrap();
    }

    #[tokio::test]
    async fn test_fetch_version() {
        let version = fetch_manifest().await.unwrap();
        println!("{:?}", version);
    }
}
