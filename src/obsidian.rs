use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct SceneData {
    pub id: Option<String>,
    pub scene_number: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct SceneRequest {
    pub character: Option<String>,
    pub duration: Option<String>,
    pub aspect_ratio: Option<String>,
    pub constraints: Vec<String>,
    pub project_id: Option<String>,
    pub character_id: Option<String>,
    pub scenes: Vec<SceneData>,
}

#[derive(Debug, Deserialize, Default)]
struct TechnicalGear {
    lens: Option<String>,
    focal_length: Option<String>,
    aperture: Option<String>,
    aspect_ratio: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct DirectorIntentInner {
    global_aesthetic: Option<String>,
    technical_gear: Option<TechnicalGear>,
}

#[derive(Debug, Deserialize, Default)]
struct DirectorIntentBlock {
    director_intent: Option<DirectorIntentInner>,
}

pub fn parse_markdown_file<P: AsRef<Path>, Q: AsRef<Path>>(
    manifest_path: P,
    character_path: Q,
) -> Result<SceneRequest, String> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let char_content = fs::read_to_string(character_path)
        .map_err(|e| format!("Failed to read character file: {}", e))?;
        
    let mut request = parse_markdown_content(&content)?;
    
    if let Some(char_props) = parse_character_frontmatter(&char_content) {
        if request.character.is_none() || request.character.as_ref().unwrap().is_empty() {
            request.character = Some(char_props);
        } else {
            let existing = request.character.take().unwrap();
            request.character = Some(format!("{} ({})", existing, char_props));
        }
    }
    
    Ok(request)
}

fn parse_character_frontmatter(content: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let mut props = Vec::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            } else {
                in_frontmatter = true;
                continue;
            }
        }
        
        if in_frontmatter {
            if let Some((key, value)) = trimmed.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
                props.push(format!("{}: {}", key, value));
            }
        }
    }
    
    if props.is_empty() {
        None
    } else {
        Some(props.join(", "))
    }
}

/// Scans for the first ```json block containing "director_intent" and extracts
/// global_aesthetic and a formatted technical_gear string.
fn parse_director_intent(content: &str) -> Option<(String, String)> {
    let mut in_code_block = false;
    let mut json_buf = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```json") {
            in_code_block = true;
            json_buf.clear();
            continue;
        }
        if in_code_block && trimmed == "```" {
            in_code_block = false;
            if json_buf.contains("director_intent") {
                if let Ok(block) = serde_json::from_str::<DirectorIntentBlock>(&json_buf) {
                    if let Some(intent) = block.director_intent {
                        let aesthetic = intent.global_aesthetic.unwrap_or_default();
                        let gear_str = intent.technical_gear.map(|g| {
                            format!(
                                "{}, {}, {}, {}",
                                g.lens.unwrap_or_default(),
                                g.focal_length.unwrap_or_default(),
                                g.aperture.unwrap_or_default(),
                                g.aspect_ratio.unwrap_or_default()
                            )
                        }).unwrap_or_default();
                        return Some((aesthetic, gear_str));
                    }
                }
            }
            json_buf.clear();
            continue;
        }
        if in_code_block {
            json_buf.push_str(line);
            json_buf.push('\n');
        }
    }
    None
}

fn extract_backtick_id(line: &str) -> Option<String> {
    let start = line.find('`')?;
    let rest = &line[start + 1..];
    let end = rest.find('`')?;
    let id = &rest[..end];
    if id.is_empty() { None } else { Some(id.to_string()) }
}

pub fn parse_markdown_content(content: &str) -> Result<SceneRequest, String> {
    let mut request = SceneRequest::default();

    let director_prefix = parse_director_intent(content).map(|(aesthetic, gear)| {
        if gear.is_empty() {
            format!("[Aesthetic: {}] ", aesthetic)
        } else {
            format!("[Aesthetic: {}] [Gear: {}] ", aesthetic, gear)
        }
    });

    let mut in_frontmatter = false;
    let mut frontmatter_parsed = false;
    let mut in_production_status = false;
    let mut in_flowkit_ids = false;
    let mut lines_before_frontmatter = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Frontmatter boundary (only before it's been parsed)
        if !frontmatter_parsed && trimmed == "---" {
            if in_frontmatter {
                in_frontmatter = false;
                frontmatter_parsed = true;
            } else if lines_before_frontmatter == 0 {
                in_frontmatter = true;
            }
            continue;
        }

        if !in_frontmatter && !frontmatter_parsed && !trimmed.is_empty() {
            lines_before_frontmatter += 1;
        }

        if in_frontmatter {
            if let Some((key, value)) = trimmed.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
                match key {
                    "character" => request.character = Some(value.to_string()),
                    "duration" => request.duration = Some(value.to_string()),
                    "aspect_ratio" => request.aspect_ratio = Some(value.to_string()),
                    "constraints" => {
                        let clean_val = value.trim_start_matches('[').trim_end_matches(']');
                        request.constraints = clean_val
                            .split(',')
                            .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    _ => {}
                }
            }
            continue;
        }

        // Horizontal rules after frontmatter reset section state
        if frontmatter_parsed && trimmed == "---" {
            in_production_status = false;
            in_flowkit_ids = false;
            continue;
        }

        // Section header detection
        if trimmed.starts_with("## Production Status") {
            in_production_status = true;
            in_flowkit_ids = false;
            continue;
        } else if trimmed.starts_with("**FlowKit IDs**") {
            in_flowkit_ids = true;
            continue;
        } else if trimmed.starts_with("## ") {
            in_production_status = false;
            in_flowkit_ids = false;
        }

        // Parse FlowKit IDs
        if in_flowkit_ids {
            if trimmed.to_lowercase().starts_with("- project:") {
                request.project_id = extract_backtick_id(trimmed);
            } else if trimmed.to_lowercase().contains("entity:") {
                if request.character_id.is_none() {
                    request.character_id = extract_backtick_id(trimmed);
                }
            }
        }

        // Parse Production Status table
        // Columns: | Scene | Plate | FlowKit Prompt | Status |
        if in_production_status {
            println!("DEBUG parser line: {}", trimmed);
        }

        if in_production_status && trimmed.starts_with('|') {
            if trimmed.contains("---") || trimmed.to_lowercase().contains("flowkit prompt") {
                continue;
            }

            let parts: Vec<&str> = trimmed.split('|').collect();
            // parts[0]="", parts[1]=Scene, parts[2]=Plate, parts[3]=Prompt, parts[4]=Status
            if parts.len() >= 5 {
                let scene_number = parts[1].trim();
                let prompt = parts[3].trim();
                let status = parts[4].trim().to_lowercase();

                if status.contains("pending") || status.contains("⏳") {
                    let full_prompt = match &director_prefix {
                        Some(prefix) => format!("{}{}", prefix, prompt),
                        None => prompt.to_string(),
                    };
                    
                    let id = extract_backtick_id(parts[1]).or_else(|| extract_backtick_id(parts[2]));

                    request.scenes.push(SceneData {
                        id,
                        scene_number: scene_number.to_string(),
                        prompt: full_prompt,
                    });
                }
            }
        }
    }

    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_content() {
        let content = r#"---
character: "Cyberpunk Hacker"
duration: "10s"
aspect_ratio: "16:9"
constraints: [neon, dark, rain]
---
## Production Status

| Scene | Plate | FlowKit Prompt | Status |
| --- | --- | --- | --- |
| 1 | PLATE_01 | The hacker types furiously. | ✅ DONE |
| 2 | PLATE_02 | A neon sign flickers. | ⏳ PENDING |
| 3 | PLATE_03 | Rain falls on the pavement. | ⏳ PENDING |
"#;

        let result = parse_markdown_content(content).unwrap();
        assert_eq!(result.character.as_deref(), Some("Cyberpunk Hacker"));
        assert_eq!(result.duration.as_deref(), Some("10s"));
        assert_eq!(result.aspect_ratio.as_deref(), Some("16:9"));
        assert_eq!(result.constraints, vec!["neon", "dark", "rain"]);
        assert_eq!(result.scenes.len(), 2);
        assert_eq!(result.scenes[0].scene_number, "2");
        assert_eq!(result.scenes[0].prompt, "A neon sign flickers.");
        assert_eq!(result.scenes[1].scene_number, "3");
        assert_eq!(result.scenes[1].prompt, "Rain falls on the pavement.");
    }

    #[test]
    fn test_director_intent_prepend() {
        let content = r#"
## Directorial Intent

```json
{
  "director_intent": {
    "actor_id": "TEST",
    "global_aesthetic": "cinematic warm tone",
    "technical_gear": {
      "lens": "Anamorphic",
      "focal_length": "35mm",
      "aperture": "f/1.4",
      "aspect_ratio": "9:16"
    }
  }
}
```

---

## Production Status

| Scene | Plate | FlowKit Prompt | Status |
| --- | --- | --- | --- |
| 1 | PLATE_01 | A street scene. | ⏳ PENDING |
"#;
        let result = parse_markdown_content(content).unwrap();
        assert_eq!(result.scenes.len(), 1);
        assert!(result.scenes[0].prompt.starts_with("[Aesthetic: cinematic warm tone]"));
        assert!(result.scenes[0].prompt.contains("[Gear:"));
        assert!(result.scenes[0].prompt.contains("A street scene."));
    }

    #[test]
    fn test_flowkit_ids() {
        let content = r#"
## Production Status

| Scene | Plate | FlowKit Prompt | Status |
| --- | --- | --- | --- |
| 1 | PLATE_01 | A scene. | ⏳ PENDING |

**FlowKit IDs**
- Project: `proj-123`
- Mateo entity: `char-456` — ref `other-id`

---
"#;
        let result = parse_markdown_content(content).unwrap();
        assert_eq!(result.project_id.as_deref(), Some("proj-123"));
        assert_eq!(result.character_id.as_deref(), Some("char-456"));
    }
}
