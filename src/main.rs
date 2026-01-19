use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to sort
    #[arg(short, long, default_value = ".")]
    target_dir: String,

    /// The LLM model to use
    #[arg(short, long, default_value = "gpt-oss:20b-cloud")]
    model: String,

    /// The Ollama API URL
    #[arg(long, default_value = "http://localhost:11434/api/generate")]
    api_url: String,

    /// Number of files to process in a single LLM batch
    #[arg(short, long, default_value = "15")]
    batch_size: usize,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new();
    let target_path = Path::new(&args.target_dir);

    if !target_path.exists() || !target_path.is_dir() {
        anyhow::bail!("Target directory does not exist or is not a directory: {:?}", target_path);
    }

    println!("Sorting files in {:?} using model '{}' (Batch size: {})...", target_path, args.model, args.batch_size);

    let entries = fs::read_dir(target_path).context("Failed to read directory")?;
    let mut files_to_process = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() { continue; }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') { continue; }
            files_to_process.push(path);
        }
    }

    if files_to_process.is_empty() {
        println!("No files found to sort.");
        return Ok(());
    }

    // Process in batches
    for chunk in files_to_process.chunks(args.batch_size) {
        process_batch(&client, &args, chunk).await?;
    }

    println!("Done!");
    Ok(())
}

async fn process_batch(client: &Client, args: &Args, paths: &[PathBuf]) -> Result<()> {
    let filenames: Vec<String> = paths.iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    let filenames_json = serde_json::to_string(&filenames).unwrap_or_else(|_| "[]".to_string());

    let prompt = format!(
        "Analyze this list of filenames and assign a concise directory name for each.
        Rules:
        1. Group files primarily by file extension and type (e.g., all .mp3/.wav files should go to 'Music' or 'Audio', .jpg/.png to 'Images').
        2. Do NOT translate Japanese or foreign filenames to English for the category name. Classify them by their file type (e.g. 'Music').
        3. Use specific categories only if semantically distinct (e.g., 'Invoices' vs 'Documents').
        Return ONLY a JSON object mapping filenames to directory names.
        Filenames: {}
        Example output: {{ \"song.mp3\": \"Music\", \"photo.jpg\": \"Images\", \"invoice.pdf\": \"Documents\" }}",
        filenames_json
    );

    let request = OllamaRequest {
        model: args.model.clone(),
        prompt: prompt.clone(),
        stream: false,
        format: "json".to_string(), // Tell Ollama to enforce JSON output
    };

    let max_retries = 3;
    let mut mapping: Option<HashMap<String, String>> = None;

    for attempt in 1..=max_retries {
        let res = client.post(&args.api_url)
            .json(&request)
            .send()
            .await;

        match res {
            Ok(response) => {
                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    eprintln!("API Error (Attempt {}/{}): {} - {}", attempt, max_retries, status, error_text);
                } else {
                    match response.json::<OllamaResponse>().await {
                        Ok(ollama_res) => {
                            // Clean markdown if present
                            let clean_json = ollama_res.response.trim();
                            let clean_json = clean_json.strip_prefix("```json").unwrap_or(clean_json);
                            let clean_json = clean_json.strip_prefix("```").unwrap_or(clean_json);
                            let clean_json = clean_json.strip_suffix("```").unwrap_or(clean_json);
                            
                            match serde_json::from_str::<HashMap<String, String>>(clean_json) {
                                Ok(map) => {
                                    mapping = Some(map);
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("JSON Parse Error (Attempt {}/{}): {}. Response was: {}", attempt, max_retries, e, ollama_res.response);
                                }
                            }
                        }
                        Err(e) => eprintln!("Failed to parse response body (Attempt {}/{}): {}", attempt, max_retries, e),
                    }
                }
            }
            Err(e) => eprintln!("Network Error (Attempt {}/{}): {}", attempt, max_retries, e),
        }

        if attempt < max_retries {
            eprintln!("Retrying in 2 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    let mapping = match mapping {
        Some(m) => m,
        None => {
            eprintln!("Failed to process batch after {} attempts. Skipping batch.", max_retries);
            return Ok(());
        }
    };

    for path in paths {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        if let Some(category) = mapping.get(&filename) {
            let sanitized_category = category.chars().filter(|c| c.is_alphanumeric()).collect::<String>();
            let sanitized_category = if sanitized_category.is_empty() { "Other".to_string() } else { sanitized_category };

            let target_dir = Path::new(&args.target_dir).join(&sanitized_category);
            if !target_dir.exists() {
                fs::create_dir_all(&target_dir).context("Failed to create category directory")?;
            }

            let new_path = target_dir.join(path.file_name().unwrap());
            println!("Moving '{}' -> '{}'", filename, sanitized_category);
            fs::rename(path, new_path).ok(); // Use ok() to avoid stopping the whole batch on one failure
        }
    }

    Ok(())
}