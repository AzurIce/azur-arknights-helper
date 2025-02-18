use std::{
    fmt::Debug,
    fs::{self, File},
    io::Cursor,
    ops::Deref,
    path::{Path, PathBuf},
};

pub mod manifest;

use anyhow::Context;
use bytes::Bytes;
use image::DynamicImage;
use log::info;
use manifest::{task::TaskConfig, Manifest};
use serde::de::DeserializeOwned;

use crate::task::Task;

pub trait Load {
    fn load(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub trait GetTask<ActionSet: Debug + Clone> {
    /// 获取 `resources-root/tasks/<name>.toml` 的任务配置
    fn get_task(&self, name: impl AsRef<str>) -> Option<&Task<ActionSet>>;
}

pub trait ResRoot {
    fn res_root(&self) -> &Path;
}

// pub trait GetTemplate {
//     /// 获取 `resources-root/templates/<path>` 的图片
//     fn get_template(&self, path: impl AsRef<Path>) -> anyhow::Result<DynamicImage>;
// }

/// 一个通用的基础 resources 目录应当具备以下目录结构：
/// ```
/// /resource-repo
/// ├── manifest.toml
/// ├── tasks
/// │   ├── task1.toml
/// │   ├── task2.toml
/// │   └── ...
/// ├── templates
/// │   ├── template1.png
/// │   ├── template2.png
/// │   └── ...
/// └── ...
#[derive(Debug)]
pub struct GeneralAahResource<ActionSet: Debug + Clone> {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub task_config: TaskConfig<ActionSet>,
}

impl<ActionSet: Debug + Clone + DeserializeOwned> Load for GeneralAahResource<ActionSet> {
    fn load(root: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let root = root.as_ref().to_path_buf();
        if !root.exists() {
            anyhow::bail!("Resource root not exists: {:?}", root);
        }

        let manifest = fs::read_to_string(root.join("manifest.toml"))?;
        let manifest = toml::from_str(&manifest)?;

        let task_config =
            TaskConfig::load(root.join("tasks")).context("failed to load task config")?;
        // let copilot_config =
        //     CopilotConfig::load(root.join("copilot")).context("failed to load copilot config")?;
        // let navigate_config = NavigateConfig::load(root.join("navigates.toml"))
        //     .context("failed to load navigate config")?;

        Ok(Self {
            root,
            manifest,
            task_config,
            // copilot_config,
            // navigate_config,
        })
    }
}

impl<ActionSet: Debug + Clone> GetTask<ActionSet> for GeneralAahResource<ActionSet> {
    fn get_task(&self, name: impl AsRef<str>) -> Option<&Task<ActionSet>> {
        let name = name.as_ref().to_string();
        self.task_config.0.get(&name)
    }
}

impl<ActionSet: Debug + Clone> ResRoot for GeneralAahResource<ActionSet> {
    fn res_root(&self) -> &Path {
        &self.root
    }
}

// impl<ActionSet: Debug + Clone> GetTemplate for GeneralAahResource<ActionSet> {
//     fn get_template(&self, path: impl AsRef<Path>) -> anyhow::Result<DynamicImage> {
//         let path = path.as_ref();
//         let img = image::open(self.root.join("templates").join(path))?;
//         Ok(img)
//     }
// }

// MARK: GitRepoResource

/// 一个实现了 [`Load`] 的结构的 Wrapper，用于便捷地从 Git 仓库初始化/更新资源。
pub struct GitRepoResource<T: Load> {
    repo_url: String,
    root: PathBuf,
    pub inner: T,
    manifest: Manifest,
}

impl<T: Load> Deref for GitRepoResource<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Load> GitRepoResource<T> {
    pub async fn try_init(
        target_dir: impl AsRef<Path>,
        repo_url: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let repo_url = repo_url.as_ref().to_string();
        let target_dir = target_dir.as_ref();

        if target_dir.exists() {
            fs::remove_dir_all(target_dir).context("failed to clean resource dir")?;
        }
        fs::create_dir_all(target_dir).context("failed to create resource dir")?;

        let parent_dir = target_dir.parent().unwrap();
        download_repo_zip(&repo_url, parent_dir).await?;

        let file =
            File::open(parent_dir.join("resources.zip")).context("failed to open resources.zip")?;
        let mut archive = zip::ZipArchive::new(file).context("failed to read zip archive")?;

        info!("extracting resources...");
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            };
            let outpath = outpath.components().skip(1).collect::<PathBuf>();
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
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }
        Ok(())
    }

    pub async fn try_load_or_init(
        target_dir: impl AsRef<Path>,
        repo_url: impl AsRef<str>,
    ) -> anyhow::Result<Self> {
        let target_dir = target_dir.as_ref();
        let repo_url = repo_url.as_ref().to_string();
        if let Ok(resource) = T::load(&target_dir) {
            let manifest = fs::read_to_string(target_dir.join("manifest.toml"))?;
            let manifest = toml::from_str(&manifest)?;
            return Ok(Self {
                repo_url,
                inner: resource,
                manifest,
                root: target_dir.to_path_buf(),
            });
        } else {
            Self::try_init(&target_dir, &repo_url).await?;
            let manifest = fs::read_to_string(target_dir.join("manifest.toml"))?;
            let manifest = toml::from_str(&manifest)?;
            T::load(target_dir).map(|resource| Self {
                repo_url,
                inner: resource,
                manifest,
                root: target_dir.to_path_buf(),
            })
        }
    }
    /// Update resource to the latest version
    ///
    /// this will not do anything if the `last_updated` field in `manifest.toml` is unchanged
    /// when the version is updated, it'll fetch the origin and checkout to latest main
    pub async fn update(self) -> Result<Self, anyhow::Error> {
        let manifest = fetch_manifest(&self.repo_url).await?;
        if self.manifest.last_updated == manifest.last_updated {
            info!("Resource is up to date");
            return Ok(self);
        }

        fs::remove_dir_all(&self.root)?;
        Self::try_load_or_init(&self.root, self.repo_url).await
    }
}

/// MARK: Functions

async fn download_repo_zip(
    repo_url: impl AsRef<str>,
    dir: impl AsRef<Path>,
) -> Result<(), anyhow::Error> {
    let mut repo_url = repo_url.as_ref().to_string();
    if !repo_url.ends_with("/") {
        repo_url.push('/');
    }

    let url = format!("{repo_url}archive/main.zip");

    let client = reqwest::Client::builder()
        .user_agent("azur-arknights-helper")
        .build()?;

    let response = client.get(url).send().await?;

    if response.status().is_success() {
        info!("saving resources.zip...");
        let mut file = File::create(dir.as_ref().join("resources.zip"))?;
        let bytes = response.bytes().await?;
        std::io::copy(&mut Cursor::new(bytes), &mut file)?;
        info!("downloaded resource zip");
        return Ok(());
    } else {
        anyhow::bail!("download request status not success")
    }
}

// api: "https://api.github.com/repos/AzurIce/azur-arknights-helper/contents/resources";
async fn fetch_file_from_github(
    repo_url: impl AsRef<str>,
    path: impl AsRef<str>,
) -> Result<Bytes, anyhow::Error> {
    let path = path.as_ref();
    let mut repo_url = repo_url.as_ref().to_string();
    if !repo_url.ends_with("/") {
        repo_url.push('/');
    }
    let mut url = repo_url
        .split(".com")
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
    url[0] = "https://api.github.com/repos".to_string();
    let url = url.join("");
    let url = format!("{url}contents/{path}");

    let client = reqwest::Client::builder()
        .user_agent("azur-arknights-helper")
        .build()?;
    let resp = client
        .get(url)
        .header("Accept", "application/vnd.github.raw+json")
        .send()
        .await?;
    Ok(resp.bytes().await?)
}

async fn fetch_manifest(repo_url: impl AsRef<str>) -> Result<Manifest, anyhow::Error> {
    let manifest_bytes = fetch_file_from_github(repo_url, "manifest.toml").await?;
    let manifest = String::from_utf8_lossy(&manifest_bytes);
    let manifest: Manifest = toml::from_str(&manifest)?;

    Ok(manifest)
}

// MARK: Test

#[cfg(test)]
mod test {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    use crate::android;

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
    async fn test_try_load_or_init_resource() {
        init_logger();

        let resource =
            GitRepoResource::<GeneralAahResource<android::actions::ActionSet>>::try_load_or_init(
                "./test/.aah/resources",
                "https://github.com/AzurIce/aah-resources",
            )
            .await
            .unwrap();
        println!("{:?}", resource.manifest);
    }

    #[tokio::test]
    async fn test_update_resource() {
        init_logger();

        let resource =
            GitRepoResource::<GeneralAahResource<android::actions::ActionSet>>::try_load_or_init(
                "./test/.aah/resources",
                "https://github.com/AzurIce/aah-resources",
            )
            .await
            .unwrap();
        let resource = resource.update().await.unwrap();
        println!("{:?}", resource.manifest);
    }

    #[tokio::test]
    async fn test_fetch_manifest() {
        let version = fetch_manifest("https://github.com/AzurIce/aah-resources")
            .await
            .unwrap();
        println!("{:?}", version);
    }
}
