// use std::{fs, path::Path};

// use serde::Deserialize;


// #[derive(Deserialize, Debug)]
// struct GitHubContent {
//     name: String,
//     path: String,
//     sha: String,
//     size: u64,
//     url: String,
//     download_url: Option<String>,
//     #[serde(rename = "type")]
//     content_type: String,
// }

// async fn download_file(url: &str, file_path: &Path) -> Result<(), anyhow::Error> {
//     let response = reqwest::get(url).await?.bytes().await?;
//     fs::write(file_path, &response).expect("Unable to write file");
//     Ok(())
// }

// async fn download_directory(owner: &str, repo: &str, path: &str, local_path: &Path) -> Result<(), anyhow::Error> {
//     let url = format!(
//         "https://api.github.com/repos/{}/{}/contents/{}",
//         owner, repo, path
//     );

//     let client = reqwest::Client::new();
//     let response = client
//         .get(&url)
//         .header("User-Agent", "azur-arknights-helper")
//         .send()
//         .await?;

//     let contents: Vec<GitHubContent> = response.json().await?;

//     for content in contents {
//         let local_file_path = local_path.join(&content.name);
//         if content.content_type == "dir" {
//             fs::create_dir_all(&local_file_path).expect("Unable to create directory");
//             download_directory(owner, repo, &content.path, &local_file_path).await?;
//         } else if let Some(download_url) = &content.download_url {
//             download_file(download_url, &local_file_path).await?;
//             println!("Downloaded: {:?}", local_file_path);
//         }
//     }

//     Ok(())
// }