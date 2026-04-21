use crate::api::{CliError, VeoClient};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct StatusResponse {
    status: String,
    media_url: Option<String>,
}

pub async fn poll_request_status(client: &VeoClient, request_id: &str) -> Result<String, CliError> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    pb.set_message("Waiting for generation to complete...");

    loop {
        let url = format!("{}/api/requests/batch-status/{}", client.base_url(), request_id);
        let mut req = client.client().get(&url);
        let token = client.bearer_token();
        if !token.is_empty() {
            req = req.bearer_auth(token);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            let status_res: StatusResponse = response.json().await?;
            
            if status_res.status == "COMPLETED" {
                pb.finish_with_message("Generation completed successfully.");
                return Ok(status_res.media_url.unwrap_or_default());
            } else if status_res.status == "FAILED" {
                pb.finish_with_message("Generation failed.");
                return Err(CliError::ApiError { status: 500, message: "Generation failed on the server".into() });
            }
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            pb.finish_with_message("Error during status poll.");
            return Err(CliError::ApiError { status, message });
        }

        pb.inc(1);
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
