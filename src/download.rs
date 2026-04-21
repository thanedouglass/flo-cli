use crate::api::{CliError, VeoClient};
use std::fs;
use std::io::Write;
use std::path::Path;

pub async fn download_asset(
    client: &VeoClient,
    media_url: &str,
    project_id: &str,
    scene_id: &str,
) -> Result<(), CliError> {
    if media_url.is_empty() {
        return Err(CliError::ApiError {
            status: 0,
            message: "Media URL is empty or missing from status response".into(),
        });
    }

    let dir = Path::new("/Users/thanedouglass/Desktop/purple-pill-obsidian/assets/pilots/01/");
    fs::create_dir_all(dir).map_err(|e| CliError::ApiError {
        status: 0,
        message: format!("Failed to create directory: {}", e),
    })?;

    let file_name = format!("{}_{}.mp4", project_id, scene_id);
    let file_path = dir.join(file_name);

    println!("Downloading asset from {}...", media_url);

    let mut req = client.client().get(media_url);
    let token = client.bearer_token();
    if !token.is_empty() {
        req = req.bearer_auth(token);
    }

    let response = req.send().await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let mut file = fs::File::create(&file_path).map_err(|e| CliError::ApiError {
            status: 0,
            message: format!("Failed to create file: {}", e),
        })?;
        file.write_all(&bytes).map_err(|e| CliError::ApiError {
            status: 0,
            message: format!("Failed to write file: {}", e),
        })?;
        println!("Asset successfully downloaded to: {}", file_path.display());
        Ok(())
    } else {
        let status = response.status().as_u16();
        Err(CliError::ApiError {
            status,
            message: format!("Failed to download asset: HTTP {}", status),
        })
    }
}
