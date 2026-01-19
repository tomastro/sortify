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

    let prompt = format!(
        "Analyze this list of filenames and suggest a concise, single-word directory name for each. 
        Return ONLY a JSON object where keys are filenames and values are the suggested directory names.
        Filenames: {:?}
        Example output: {{ \"file1.jpg\": \"Photos\", \"script.py\": \"Coding\" }}",
        filenames
    );

    let request = OllamaRequest {
        model: args.model.clone(),
        prompt,
        stream: false,
        format: "json".to_string(), // Tell Ollama to enforce JSON output
    };

    let res = client.post(&args.api_url)
        .json(&request)
        .send()
        .await
        .context("Failed to contact Ollama API")?;

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        eprintln!("API Error: {} - {}", status, error_text);
        return Ok(());
    }

    let ollama_res: OllamaResponse = res.json().await.context("Failed to parse Ollama response")?;
    
    // Parse the JSON mapping from the LLM
    let mapping: HashMap<String, String> = serde_json::from_str(&ollama_res.response)
        .context("LLM returned invalid JSON mapping")?;

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