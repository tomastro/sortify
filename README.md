# NeuroSort 🧠📂

**NeuroSort** (internally `llm_sorter`) is an intelligent, privacy-first CLI tool written in Rust that brings order to your filesystem chaos. Instead of relying on brittle file extensions or complex regex rules, NeuroSort uses the power of Local Large Language Models (LLMs) via **Ollama** to semantically understand what a file is based on its name and categorize it accordingly.

> "Stop organizing files yourself. Let the AI do it."

## 🚀 Features

*   **Semantic Classification:** Understands that `final_v2_real_draft.pdf` is a *Document* and `main.rs` is *Code*, without hardcoded lists.
*   **Privacy First:** Runs 100% locally using Ollama. No data leaves your machine.
*   **Blazingly Fast:** Built with Rust 🦀 and Tokio for asynchronous performance.
*   **Model Agnostic:** Use any model available in your Ollama library (`llama3`, `mistral`, `gpt-oss`, etc.).

## 🛠️ Prerequisites

1.  **Rust Toolchain:** [Install Rust](https://rustup.rs/)
2.  **Ollama:** [Install Ollama](https://ollama.com/)
3.  **An LLM Model:**
    ```bash
    ollama pull gpt-oss:cloud-20b
    # OR
    ollama pull llama3
    ```

## 📦 Installation

Clone the repository and build:

```bash
cd llm_sorter
cargo build --release
```

The binary will be located at `./target/release/llm_sorter`.

## 💻 Usage

### Basic Sorting
Sort files in the current directory using the default model (`gpt-oss:cloud-20b`):

```bash
cargo run
```

### Specify a Directory
Clean up a specific messy download folder:

```bash
cargo run -- --target-dir ~/Downloads
```

### Use a Different Model
If you prefer `llama3` or another model you have pulled:

```bash
cargo run -- --model llama3
```

### Custom API URL
If Ollama is running on a different port or machine:

```bash
cargo run -- --api-url http://192.168.1.50:11434/api/generate
```

## 📂 Categories

The AI classifies files into these default buckets:
*   **Documents** (PDFs, Word docs, Text files)
*   **Images** (JPG, PNG, GIF, SVG)
*   **Music** (MP3, WAV, FLAC)
*   **Video** (MP4, MKV, AVI)
*   **Code** (Source files, Scripts)
*   **Archives** (Zips, Tars, Gzips)
*   **Other** (Executables, Unknowns)

## 🤝 Contributing

Feel free to open issues or PRs. If you want to add more complex logic (like looking at file headers/magic bytes), go ahead!

## 📜 License

MIT
