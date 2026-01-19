# NeuroSort 🧠📂

**NeuroSort** is a high-performance, intelligent CLI tool written in Rust that organizes your filesystem using Local Large Language Models (LLMs). By leveraging **Ollama**, NeuroSort semantically analyzes filenames to categorize them into logical folders—all while keeping your data 100% private and local.

---

## ✨ Key Features

- **Semantic Intelligence:** Goes beyond extensions. Understands context to group files naturally.
- **Batch Processing:** Optimized for speed by processing multiple files in a single LLM request.
- **Multi-Language Support:** Robust handling of Unicode filenames (Japanese, Chinese, Arabic, etc.) without losing semantic meaning.
- **Dry Run Mode:** Preview your organizational changes safely before any files are moved.
- **Resilient Logic:** Automatic retries and JSON cleaning to handle LLM non-determinism.
- **Privacy First:** No cloud APIs. Your filenames never leave your machine.

---

## 🛠️ Prerequisites

1.  **Rust:** [Install via rustup.rs](https://rustup.rs/)
2.  **Ollama:** [Install via ollama.com](https://ollama.com/)
3.  **LLM Model:**
    ```bash
    ollama pull gpt-oss:20b-cloud  # Default model
    # OR
    ollama pull llama3
    ```

---

## 📦 Installation

```bash
git clone https://github.com/tomastro/sortify.git
cd sortify
cargo build --release
```
The binary will be available at `./target/release/llm_sorter`.

---

## 🚀 Usage

### 1. Basic Organizational Run
Sort files in the current directory using the default model:
```bash
cargo run
```

### 2. Preview Changes (Dry Run) 🛡️
See what NeuroSort *would* do without actually moving any files:
```bash
cargo run -- --dry-run
```

### 3. Target a Specific Directory
```bash
cargo run -- --target-dir "~/Downloads/MessyFolder"
```

### 4. Advanced Configuration
Tailor the sorting process with custom models and batch sizes:
```bash
cargo run -- \
  --model llama3 \
  --batch-size 20 \
  --api-url http://localhost:11434/api/generate
```

---

## ⚙️ Options

| Flag | Long Flag | Description | Default |
| :--- | :--- | :--- | :--- |
| `-t` | `--target-dir` | Directory to organize | `.` |
| `-m` | `--model` | Ollama model to use | `gpt-oss:20b-cloud` |
| | `--api-url` | Ollama API endpoint | `localhost:11434` |
| `-b` | `--batch-size`| Files per LLM request | `15` |
| `-d` | `--dry-run` | Preview mode (no moves) | `false` |

---

## 📂 How It Categorizes

NeuroSort uses intelligent rules to ensure your folders stay clean:
1.  **Type Grouping:** Automatically groups media (.mp3, .jpg) and docs (.pdf, .xlsx).
2.  **No-Translation Policy:** Foreign filenames (Japanese/Chinese/etc.) are categorized by type, not by their English translation.
3.  **Sanitized Naming:** Folder names are automatically sanitized for filesystem compatibility.

---

## 📜 License
Distributed under the MIT License. See `LICENSE` for more information.