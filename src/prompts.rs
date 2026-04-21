use crate::obsidian::SceneRequest;
use std::io::{self, Write};

/// Displays the parsed scene payload and prompts the user for confirmation.
/// Returns true if the user enters 'y' or 'Y', false otherwise.
pub fn confirm_execution(scene_data: &SceneRequest) -> bool {
    println!("\n=== Pre-Flight Checklist ===");
    println!(
        "Character    : {}",
        scene_data.character.as_deref().unwrap_or("None")
    );
    println!(
        "Duration     : {}",
        scene_data.duration.as_deref().unwrap_or("None")
    );
    println!(
        "Aspect Ratio : {}",
        scene_data.aspect_ratio.as_deref().unwrap_or("None")
    );
    println!(
        "Constraints  : {}",
        if scene_data.constraints.is_empty() {
            "None".to_string()
        } else {
            scene_data.constraints.join(", ")
        }
    );
    println!("----------------------------");
    println!("Scenes (PENDING):");
    if scene_data.scenes.is_empty() {
        println!("No PENDING scenes found.");
    } else {
        for scene in &scene_data.scenes {
            println!("  [Scene {}] {}", scene.scene_number, scene.prompt);
        }
    }
    println!("============================\n");

    print!("Proceed with generation? (Y/N): ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read from stdin");

    let input = input.trim().to_lowercase();
    if input == "y" {
        true
    } else {
        println!("Execution aborted by user.");
        false
    }
}
