#![feature(path_file_prefix)]
//! This crate is for handling game resources.
pub mod avatar;
pub mod level;
pub mod manifest;
mod utils;

use std::{
    env,
    fmt::Debug,
    fs::{self, File},
    io::Cursor,
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::Context;
use bytes::Bytes;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    FetchOptions, ProxyOptions, RemoteCallbacks,
};
use manifest::{
    copilot::{Copilot, CopilotConfig},
    navigate::{Navigate, NavigateConfig},
    task::{Task, TaskConfig},
    Manifest,
};
use tracing::{info, warn};

// https://docs.github.com/en/rest/repos/contents
const RESOURCE_ROOT_URL: &str =
    "https://api.github.com/repos/AzurIce/azur-arknights-helper/contents/resources";

async fn download_resource_zip(dir: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    let url = format!(
        "https://api.github.com/repos/AzurIce/azur-arknights-helper/contents/resources.zip"
    );

    let client = reqwest::Client::builder()
        .user_agent("azur-arknights-helper")
        .build()?;
    info!("sending request...");
    let request = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    info!("request status: {}", request.status());
    if request.status().is_success() {
        let json: serde_json::Value = request.json().await?;
        info!("resp json: {json}");
        let download_url = json["download_url"].as_str().unwrap();

        info!("sending download request...");
        let response = client.get(download_url).send().await?;

        if response.status().is_success() {
            info!("saving resources.zip...");
            let mut file = File::create(dir.as_ref().join("resources.zip"))?;
            let bytes = response.bytes().await?;
            std::io::copy(&mut Cursor::new(bytes), &mut file)?;
            info!("downloaded resource zip from LFS");
            return Ok(());
        } else {
            anyhow::bail!("download request status not success")
        }
    } else {
        anyhow::bail!("request status not success")
    }
}

async fn fetch_file_from_github(path: impl AsRef<str>) -> Result<Bytes, anyhow::Error> {
    let path = path.as_ref();
    let client = reqwest::Client::builder()
        .user_agent("azur-arknights-helper")
        .build()?;
    let resp = client
        .get(format!("{RESOURCE_ROOT_URL}/{path}"))
        .header("Accept", "application/vnd.github.raw+json")
        .send()
        .await?;
    Ok(resp.bytes().await?)
}

async fn fetch_manifest() -> Result<Manifest, anyhow::Error> {
    let manifest_bytes = fetch_file_from_github("manifest.toml").await?;
    let manifest = String::from_utf8_lossy(&manifest_bytes);
    let manifest: Manifest = toml::from_str(&manifest)?;

    Ok(manifest)
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
        .proxy_options(proxy_options);
    // .depth(1);
    fetch_options
}

/// A struct for local Maa Resource Dir
///
/// default should be at `./.aah/MaaResource`
#[derive(Debug)]
pub struct LocalResource {
    pub root: PathBuf,
    pub manifest: Manifest,
    /// 由 `tasks.toml` 和 `tasks` 目录加载的任务配置
    pub task_config: TaskConfig,
    /// 由 `copilots.toml` 和 `copilots` 目录加载的任务配置
    pub copilot_config: CopilotConfig,
    /// 由 `navigates.toml` 加载的导航配置
    pub navigate_config: NavigateConfig,
}

impl LocalResource {
    pub fn get_task(&self, name: impl AsRef<str>) -> Option<&Task> {
        let name = name.as_ref().to_string();
        self.task_config.0.get(&name)
    }

    pub fn get_copilot(&self, name: impl AsRef<str>) -> Option<&Copilot> {
        let name = name.as_ref().to_string();
        self.copilot_config.0.get(&name)
    }

    pub fn get_navigate(&self, name: impl AsRef<str>) -> Option<&Navigate> {
        let name = name.as_ref().to_string();
        self.navigate_config.0.get(&name)
    }

    /// 获取所有 Task 名称
    pub fn get_tasks(&self) -> Vec<String> {
        self.task_config.0.keys().map(|s| s.to_string()).collect()
    }

    /// 获取所有 Copilot 名称
    pub fn get_copilots(&self) -> Vec<String> {
        self.copilot_config
            .0
            .keys()
            .map(|s| s.to_string())
            .collect()
    }

    /// Load resource from resources dir (where the manifest.toml sits)
    pub fn load(root: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let root = root.as_ref().to_path_buf();
        if !root.exists() {
            anyhow::bail!("Resource root not exists: {:?}", root);
        }

        let manifest = fs::read_to_string(root.join("manifest.toml"))?;
        let manifest = toml::from_str(&manifest)?;

        let task_config =
            TaskConfig::load(root.join("tasks")).context("failed to load task config")?;
        let copilot_config =
            CopilotConfig::load(root.join("copilot")).context("failed to load copilot config")?;
        let navigate_config = NavigateConfig::load(root.join("navigates.toml"))
            .context("failed to load navigate config")?;

        Ok(Self {
            root,
            manifest,
            task_config,
            copilot_config,
            navigate_config,
        })
    }
}

#[derive(Debug)]
pub enum Resource {
    LocalResource(LocalResource),
    ArchiveFileResource(ArchiveFileResource),
    GitResource(GitResource),
}

impl From<LocalResource> for Resource {
    fn from(res: LocalResource) -> Self {
        Self::LocalResource(res)
    }
}

impl From<ArchiveFileResource> for Resource {
    fn from(res: ArchiveFileResource) -> Self {
        Self::ArchiveFileResource(res)
    }
}

impl From<GitResource> for Resource {
    fn from(res: GitResource) -> Self {
        Self::GitResource(res)
    }
}

impl Resource {
    pub fn root(&self) -> &Path {
        match self {
            Resource::LocalResource(res) => &res.root,
            Resource::ArchiveFileResource(res) => &res.root,
            Resource::GitResource(res) => &res.root,
        }
    }

    pub async fn try_init(target_dir: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let res = if cfg!(debug_assertions) {
            info!("debug mod, loading with LocalResource...");
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            LocalResource::load(Path::new(&manifest_dir).join("resources"))?.into()
        } else {
            info!("release mod, loading with ArchiveFileResource...");
            ArchiveFileResource::try_init(target_dir).await?.into()
        };
        Ok(res)
    }

    pub async fn updatable(&self) -> bool {
        !matches!(self, Resource::LocalResource(_))
    }

    pub async fn update(self) -> Result<Self, anyhow::Error> {
        match self {
            Resource::ArchiveFileResource(res) => {
                res.update().await.map(Resource::ArchiveFileResource)
            }
            Resource::GitResource(res) => res.update().await.map(Resource::GitResource),
            _ => unimplemented!("not implemented"),
        }
    }
}

impl Deref for Resource {
    type Target = LocalResource;
    fn deref(&self) -> &Self::Target {
        match self {
            Resource::LocalResource(res) => res,
            Resource::ArchiveFileResource(res) => res.deref(),
            Resource::GitResource(res) => res.deref(),
        }
    }
}

impl ArchiveFileResource {
    /// Try initialize resource into the target dir
    pub async fn try_init(target_dir: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let target_dir = target_dir.as_ref().to_path_buf();
        info!("loading local resource...");
        let res = match LocalResource::load(&target_dir) {
            Ok(res) => res,
            Err(err) => {
                info!("local resource load failed: {err}, downloading resource zip...");
                let parent_dir = target_dir.parent().unwrap();
                download_resource_zip(parent_dir)
                    .await
                    .context("failed to download_resource_zip")?;
                // let archived_resource = fetch_file_from_github("resources.zip")
                //     .await
                //     .context("fetching resources.zip form github")?;
                // let mut archive = zip::ZipArchive::new(std::io::Cursor::new(archived_resource))?;
                let file = File::open(parent_dir.join("resources.zip"))?;
                let mut archive = zip::ZipArchive::new(file)?;

                info!("extracting resources...");
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let outpath = match file.enclosed_name() {
                        Some(path) => path,
                        None => continue,
                    };
                    let outpath = target_dir.join(outpath);

                    // {
                    //     let comment = file.comment();
                    //     if !comment.is_empty() {
                    //         println!("File {i} comment: {comment}");
                    //     }
                    // }

                    if file.is_dir() {
                        // println!("File {} extracted to \"{}\"", i, outpath.display());
                        fs::create_dir_all(&outpath).unwrap();
                    } else {
                        // println!(
                        //     "File {} extracted to \"{}\" ({} bytes)",
                        //     i,
                        //     outpath.display(),
                        //     file.size()
                        // );
                        if let Some(p) = outpath.parent() {
                            if !p.exists() {
                                fs::create_dir_all(p).unwrap();
                            }
                        }
                        let mut outfile = fs::File::create(&outpath).unwrap();
                        std::io::copy(&mut file, &mut outfile).unwrap();
                    }

                    // Get and Set permissions
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;

                        if let Some(mode) = file.unix_mode() {
                            fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                                .unwrap();
                        }
                    }
                }
                LocalResource::load(&target_dir)?
            }
        };

        Ok(ArchiveFileResource {
            root: target_dir,
            inner: res,
        })
    }

    /// Update resource to the latest version
    ///
    /// this will not do anything if the version.json is unchanged
    /// when the version is updated, it'll fetch the origin and checkout to latest main
    pub async fn update(self) -> Result<Self, anyhow::Error> {
        let manifest = fetch_manifest().await?;
        if self.manifest.last_updated == manifest.last_updated {
            info!("Resource is up to date");
            return Ok(self);
        }

        fs::remove_dir_all(&self.root)?;

        Self::try_init(&self.root).await
    }
}

#[derive(Debug)]
pub struct ArchiveFileResource {
    pub root: PathBuf,
    inner: LocalResource,
}

impl Deref for ArchiveFileResource {
    type Target = LocalResource;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Resource from github repo
#[derive(Debug)]
pub struct GitResource {
    pub repo_root: PathBuf,
    inner: LocalResource,
}

impl Deref for GitResource {
    type Target = LocalResource;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl GitResource {
    /// Try to initialize resource into the target dir
    ///
    /// open the repo and checkout to the latest main
    /// if cannot open, clone the repo to the path, then checkout to the latest main
    pub async fn try_init(target_dir: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        info!("Opening resource repo...");
        let repo = match git2::Repository::open(&target_dir) {
            Ok(repo) => repo,
            Err(e) => {
                info!("Failed to open resource root: {e}, cloning...");
                let mut builder = RepoBuilder::new();
                builder.fetch_options(fetch_options());
                builder
                    .clone(RESOURCE_REPO, target_dir.as_ref())
                    .context("clone resource repo")?
            }
        };

        info!("Checking out head...");
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .context("checkout head")?;

        let resource = Self::load(target_dir)?;
        resource.update().await
    }

    /// Update resource to the latest version
    ///
    /// this will not do anything if the version.json is unchanged
    /// when the version is updated, it'll fetch the origin and checkout to latest main
    pub async fn update(self) -> Result<Self, anyhow::Error> {
        let manifest = fetch_manifest().await?;
        if self.manifest.last_updated == manifest.last_updated {
            info!("Resource is up to date");
            return Ok(self);
        }

        let repo = git2::Repository::open(&self.repo_root)?;

        info!("Fetching origin...");
        let mut fetch_options = fetch_options();
        repo.find_remote("origin")?
            .fetch(&["main"], Some(&mut fetch_options), None)
            .context("failed to fetch origin")?;

        info!("Checking out main...");
        repo.set_head("refs/remotes/origin/main")
            .context("faild to set head")?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .context("failed to checkout head")?;

        Self::load(&self.repo_root)
    }
}

impl GitResource {
    pub fn load(repo_root: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let repo_root = repo_root.as_ref().to_path_buf();
        let inner = LocalResource::load(&repo_root.join("resources"))?;
        Ok(Self { repo_root, inner })
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

        let resource = GitResource::try_init("./test/.aah/resources")
            .await
            .unwrap();
        println!("{:?}", resource.manifest);
    }

    #[tokio::test]
    async fn test_update_resource() {
        init_logger();

        let resource = GitResource::load("./test/.aah/resources").unwrap();
        resource.update().await.unwrap();
    }

    #[tokio::test]
    async fn test_fetch_version() {
        let version = fetch_manifest().await.unwrap();
        println!("{:?}", version);
    }
}
