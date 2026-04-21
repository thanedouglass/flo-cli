use crate::obsidian::SceneRequest;
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Reqwest(ReqwestError),
    EnvVar(env::VarError),
    ApiError { status: u16, message: String },
    Serialization(serde_json::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Reqwest(err) => write!(f, "Network error: {}", err),
            CliError::EnvVar(err) => write!(f, "Environment error: {}", err),
            CliError::ApiError { status, message } => {
                write!(f, "API Error ({}): {}", status, message)
            }
            CliError::Serialization(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl From<ReqwestError> for CliError {
    fn from(err: ReqwestError) -> Self {
        CliError::Reqwest(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Serialization(err)
    }
}

impl From<env::VarError> for CliError {
    fn from(err: env::VarError) -> Self {
        CliError::EnvVar(err)
    }
}

pub struct VeoClient {
    client: Client,
    base_url: String,
    bearer_token: String,
}

#[derive(Serialize)]
struct CreateProjectPayload {
    name: String,
}

#[derive(Deserialize)]
struct CreateProjectResponse {
    id: String,
}

#[derive(Serialize)]
struct CreateScenePayload {
    project_id: String,
    scene_number: String,
    prompt: String,
}

#[derive(Deserialize)]
struct CreateSceneResponse {
    id: String,
}

#[derive(Serialize)]
struct GenerateImageRequest {
    #[serde(rename = "type")]
    request_type: String,
    project_id: String,
    character_id: String,
    scene_number: String,
    scene_id: String,
    prompt: String,
}

#[derive(Serialize)]
struct BatchRequestPayload {
    requests: Vec<GenerateImageRequest>,
}

#[derive(Deserialize)]
struct BatchRequestResponse {
    id: String,
}

impl VeoClient {
    pub fn new() -> Result<Self, CliError> {
        dotenv::dotenv().ok();

        let base_url =
            env::var("FLOW_API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8100".to_string());
        let bearer_token = env::var("BEARER_TOKEN").unwrap_or_default();

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url,
            bearer_token,
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn bearer_token(&self) -> &str {
        &self.bearer_token
    }

    pub async fn create_project(&self, name: &str) -> Result<String, CliError> {
        let url = format!("{}/api/projects", self.base_url);
        let payload = CreateProjectPayload {
            name: name.to_string(),
        };

        let mut req = self.client.post(&url).json(&payload);
        if !self.bearer_token.is_empty() {
            req = req.bearer_auth(&self.bearer_token);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            let res_body: CreateProjectResponse = response.json().await?;
            Ok(res_body.id)
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(CliError::ApiError { status, message })
        }
    }

    pub async fn create_scene(
        &self,
        project_id: &str,
        scene_number: &str,
        prompt: &str,
    ) -> Result<String, CliError> {
        let url = format!("{}/api/scenes", self.base_url);
        let payload = CreateScenePayload {
            project_id: project_id.to_string(),
            scene_number: scene_number.to_string(),
            prompt: prompt.to_string(),
        };

        let mut req = self.client.post(&url).json(&payload);
        if !self.bearer_token.is_empty() {
            req = req.bearer_auth(&self.bearer_token);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            let res_body: CreateSceneResponse = response.json().await?;
            Ok(res_body.id)
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(CliError::ApiError { status, message })
        }
    }

    /// Submits a batch generation request. project_id and character_id are sourced
    /// from the parsed manifest's FlowKit IDs block.
    pub async fn submit_generation(
        &self,
        scene_data: &SceneRequest,
    ) -> Result<String, CliError> {
        let url = format!("{}/api/requests/batch", self.base_url);

        let project_id = scene_data.project_id.as_deref().unwrap_or("");
        let character_id = scene_data.character_id.as_deref().unwrap_or("");

        let requests: Vec<GenerateImageRequest> = scene_data
            .scenes
            .iter()
            .map(|scene| GenerateImageRequest {
                request_type: "GENERATE_IMAGE".to_string(),
                project_id: project_id.to_string(),
                character_id: character_id.to_string(),
                scene_number: scene.scene_number.clone(),
                scene_id: scene.id.clone().unwrap_or_default(),
                prompt: scene.prompt.clone(),
            })
            .collect();

        let payload = BatchRequestPayload { requests };

        let mut req = self.client.post(&url).json(&payload);
        if !self.bearer_token.is_empty() {
            req = req.bearer_auth(&self.bearer_token);
        }

        let response = req.send().await?;

        if response.status().is_success() {
            if let Ok(res_body) = response.json::<BatchRequestResponse>().await {
                Ok(res_body.id)
            } else {
                Ok(project_id.to_string())
            }
        } else {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(CliError::ApiError { status, message })
        }
    }
}
