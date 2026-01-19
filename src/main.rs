use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to sort
    #[arg(short, long, default_value = ".")]
    target_dir: String,

    /// The LLM model to use
    #[arg(short, long, default_value = "gpt-oss:cloud-20b")]
    model: String,

    /// The Ollama API URL
    #[arg(long, default_value = "http://localhost:11434/api/generate")]
    api_url: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
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

    println!("Sorting files in {:?} using model '{}'...", target_path, args.model);

    let entries = fs::read_dir(target_path).context("Failed to read directory")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip directories and hidden files
        if path.is_dir() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }

        process_file(&client, &args, &path).await?;
    }

    println!("Done!");
    Ok(())
}

async fn process_file(client: &Client, args: &Args, file_path: &Path) -> Result<()> {
    let filename = file_path.file_name().unwrap().to_string_lossy();
    
    // Construct the prompt
    let prompt = format!(
        "Analyze the filename '{}'. Suggest a short, concise directory name to categorize this file. Return ONLY the directory name (e.g., 'Invoices', 'Scripts', 'HolidayPhotos'). Do not use spaces or special characters. Do not explain.",
        filename
    );

    let request = OllamaRequest {
        model: args.model.clone(),
        prompt,
        stream: false,
    };

    // Call Ollama API
    let res = client.post(&args.api_url)
        .json(&request)
        .send()
        .await
        .context("Failed to contact Ollama API")?;

    if !res.status().is_success() {
        eprintln!("API Error for {}: {}", filename, res.status());
        return Ok(());
    }

    let ollama_res: OllamaResponse = res.json().await.context("Failed to parse Ollama response")?;
    let category = ollama_res.response.trim().replace(".", ""); // Simple cleanup

    // Sanitize category to be safe for directory name
    let category = category.chars().filter(|c| c.is_alphanumeric()).collect::<String>();
    let category = if category.is_empty() { "Other".to_string() } else { category };

    let target_dir = Path::new(&args.target_dir).join(&category);
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).context("Failed to create category directory")?;
    }

    let new_path = target_dir.join(file_path.file_name().unwrap());
    
    println!("Moving '{}' -> '{}'", filename, category);
    fs::rename(file_path, new_path).context("Failed to move file")?;

    Ok(())
}
