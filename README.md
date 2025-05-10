````markdown
# srt-bullet-summarizer

A fast and flexible CLI tool to summarize `.srt` (subtitle) and `.txt` files into clean, bullet-point summaries using a local LLM via an OpenAI-compatible endpoint (e.g., LLaMA 3.2 with [Ollama](https://ollama.com/)).

---

## ✨ Features

- ✅ Summarizes `.srt` and `.txt` files
- ✅ Strips timestamps and indices from `.srt` files
- ✅ Uses a Map-Reduce LLM prompt strategy for long content
- ✅ Generates clear, concise bullet points
- ✅ Automatically creates output filename if not specified
- ✅ Unicode-safe file handling
- ✅ **Live `.srt` monitoring script with GUI feedback** (optional)

---

## 🧠 How It Works

1. `.srt` files are cleaned of timestamps and sequence numbers.
2. The text is split into overlapping word chunks.
3. Each chunk is summarized using a **Map** prompt.
4. All chunk summaries are combined using a **Reduce** prompt.
5. The final bullet-point summary is saved to a `.txt` file.

---

## 🔧 Requirements

- Rust (1.70+ recommended)
- Running local LLM API (e.g., `ollama serve`)
- Optional: `yad` (for GUI monitoring)

---

## 🚀 Usage

### 🔨 Build the CLI tool

```sh
cargo build --release
````

### 🏃 Run manually

```sh
# Summarize an SRT file
./srt-bullet-summarizer path/to/video_subtitles.srt

# Summarize a plain text file
./srt-bullet-summarizer path/to/notes.txt

# Specify custom output path
./srt-bullet-summarizer input.srt output/summary.txt
```

> 💡 If no output path is provided, a file named like `input_summary.txt` will be created next to the input.

---

## 👀 Automatic SRT Monitor (Optional)

The `srt_monitor.sh` script watches your `~/Downloads` folder for new `.srt` files, processes them using the summarizer, and moves both the `.srt` and `_summary.txt` files into a `~/Downloads/srt/` folder. It also displays live feedback using a GUI window (requires `yad`).

### 🧪 Example

```sh
./srt_monitor.sh
```

> This launches a live monitor window that updates whenever a new `.srt` file is added.

---

## 📂 File Output Convention

* Input: `lecture.srt` → Output: `lecture_summary.txt`
* Input: `meeting_notes.txt` → Output: `meeting_notes_summary.txt`

---

## 🔌 Configuration

The code uses the following default API configuration:

```rust
let config = OpenAIConfig::default()
    .with_api_base("http://localhost:11434/v1");
```

Update it if you're using a different LLM server.

---

## 📦 Dependencies

* `langchain_rust`
* `serde_json`
* `regex`
* `tokio`
* `yad` (for the monitoring script GUI, optional)