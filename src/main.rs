pub mod api;
pub mod download;
pub mod obsidian;
pub mod prompts;
pub mod status;

use clap::{Parser, Subcommand};
use std::path::Path;
use std::process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a new workspace
    Init,
    /// Renders a scene from an Obsidian Markdown manifest
    Render {
        /// The path to the Markdown manifest file
        manifest: String,
        /// The path to the Character file
        character: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            println!("Initializing workspace...");
        }
        Commands::Render { manifest, character } => {
            let mut scene_data = obsidian::parse_markdown_file(manifest, character).unwrap_or_else(|e| {
                eprintln!("Error parsing files: {}", e);
                process::exit(1);
            });

            if !prompts::confirm_execution(&scene_data) {
                process::exit(0);
            }

            println!("Proceeding with API generation...");

            let client = api::VeoClient::new().unwrap_or_else(|e| {
                eprintln!("Failed to initialize API client: {}", e);
                process::exit(1);
            });

            // Use the project_id from the manifest if present; otherwise create one.
            let project_id = if let Some(id) = &scene_data.project_id {
                id.clone()
            } else {
                match client.create_project("flo-cli-render").await {
                    Ok(id) => id,
                    Err(e) => {
                        eprintln!("Failed to create project: {}", e);
                        process::exit(1);
                    }
                }
            };

            // Auto-Initialization: Check if scenes have IDs, create them if they don't
            let mut new_ids_created = false;
            for scene in &mut scene_data.scenes {
                if scene.id.is_none() {
                    match client.create_scene(&project_id, &scene.scene_number, &scene.prompt).await {
                        Ok(new_id) => {
                            println!("Created new scene record for Scene {} -> ID: {}", scene.scene_number, new_id);
                            scene.id = Some(new_id);
                            new_ids_created = true;
                        }
                        Err(e) => {
                            eprintln!("Failed to create scene {}: {}", scene.scene_number, e);
                            process::exit(1);
                        }
                    }
                }
            }

            if new_ids_created {
                println!("\nUpdate Manifest: Please paste the following IDs into your Obsidian file:");
                for scene in &scene_data.scenes {
                    if let Some(id) = &scene.id {
                        println!("Scene {}: `{}`", scene.scene_number, id);
                    }
                }
                println!();
            }

            let request_id = match client.submit_generation(&scene_data).await {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to submit generation: {}", e);
                    process::exit(1);
                }
            };

            println!("Successfully submitted generation request! ID: {}", request_id);

            let media_url = match status::poll_request_status(&client, &request_id).await {
                Ok(url) => url,
                Err(e) => {
                    eprintln!("Failed to poll status: {}", e);
                    process::exit(1);
                }
            };

            let file_path = Path::new(manifest);
            let scene_id = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("scene");

            if let Err(e) =
                download::download_asset(&client, &media_url, &project_id, scene_id).await
            {
                eprintln!("Failed to download asset: {}", e);
                process::exit(1);
            }
        }
    }
}
