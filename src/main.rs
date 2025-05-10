// Author: Mohammed H Alsaeygh
// Project: srt-bullet-summarizer
//
// Description:
// This Rust CLI tool processes `.srt` (subtitle) or `.txt` files and generates a concise,
// bullet-point summary using a locally hosted LLM like LLaMA 3.2 via an OpenAI-compatible API.
// For `.srt` files, it strips timestamps and sequence numbers before processing. Text is split
// into overlapping chunks, summarized individually (Map), and then combined (Reduce) into a final summary.
//
// Dependencies:
// - langchain_rust: For LLM chaining and prompt handling.
// - serde_json: For dynamic input/output with the LLM.
// - regex: For timestamp/sequence number removal from `.srt` files.
//
// How to Use:
// 1. Compile the code using Cargo: `cargo build --release`.
// 2. Run the tool with the input file path as the first argument. Optionally, specify an output path.
//
// Example Usage:
// $ ./srt-bullet-summarizer ./example.srt
// $ ./srt-bullet-summarizer ./notes.txt ./output/summary.txt
//
// The summary will be saved in the same directory as the input file by default, using the
// filename format: `<original_name>_summary.txt` if no output path is given.

use regex::Regex;
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    path::Path,
    time::Instant,
};
use langchain_rust::{
    chain::{Chain, LLMChainBuilder},
    llm::openai::{OpenAI, OpenAIConfig},
    prompt::{HumanMessagePromptTemplate, PromptTemplate, TemplateFormat},
};

const MAP_TEMPLATE: &str = r#"Write a detailed summary of this text section in bullet points.
Use '-' for bullet points and answer only the bullet points.
Text:
{text}

SUMMARY:"#;

const COMBINE_TEMPLATE: &str = r#"Combine these summaries into a final summary in bullet points.
Use '-' for bullet points and answer only the bullet points.
Text:
{text}

FINAL SUMMARY:"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // === 1. Get input file path ===
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }
    let input_path = Path::new(&args[1]);
    if !input_path.exists() {
        return Err(format!("File not found: {:?}", input_path).into());
    }

    println!("Processing file: {:?}", input_path);
    let start_time = Instant::now();

    // === 2. Read and clean if SRT ===
    let raw_text = fs::read_to_string(input_path)?;
    let cleaned_text = match input_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
        "srt" => clean_srt(&raw_text),
        _ => raw_text,
    };

    // === 3. Configure LLM ===
    let config = OpenAIConfig::default()
        .with_api_base("http://localhost:11434/v1");
    let llm = OpenAI::new(config).with_model("llama3.2".to_string());

    // === 4. Prompt templates ===
    let map_prompt = PromptTemplate::new(
        MAP_TEMPLATE.to_string(),
        vec!["text".to_string()],
        TemplateFormat::FString,
    );
    let combine_prompt = PromptTemplate::new(
        COMBINE_TEMPLATE.to_string(),
        vec!["text".to_string()],
        TemplateFormat::FString,
    );

    // === 5. Chains ===
    let map_chain = LLMChainBuilder::new()
        .prompt(HumanMessagePromptTemplate::new(map_prompt))
        .llm(llm.clone())
        .build()?;
    let combine_chain = LLMChainBuilder::new()
        .prompt(HumanMessagePromptTemplate::new(combine_prompt))
        .llm(llm)
        .build()?;

    // === 6. Split text ===
    let chunks = split_text(&cleaned_text, 2000, 200);
    println!("Split into {} chunks", chunks.len());

    // === 7. Map step ===
    let map_start = Instant::now();
    let mut summaries = Vec::new();
    for chunk in chunks {
        let mut args = HashMap::new();
        args.insert("text".to_string(), Value::String(chunk));
        let gen = map_chain.call(args).await?;
        summaries.push(gen.generation);
    }
    println!("Map step completed in {:?}", map_start.elapsed());

    // === 8. Combine step ===
    let combined_input = summaries.join("\n\n");
    let mut combine_args = HashMap::new();
    combine_args.insert("text".to_string(), Value::String(combined_input));
    let combine_gen = combine_chain.call(combine_args).await?;
    let final_summary = combine_gen.generation;

    // === 9. Save summary ===
    let output_path = {
        let parent = input_path.parent().unwrap_or_else(|| Path::new("."));
        let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
        parent.join(format!("{}_summary.txt", stem))
    };
    fs::write(&output_path, &final_summary)?;
    println!("Summary saved to {:?}", output_path);
    println!("Total processing time: {:?}", start_time.elapsed());

    Ok(())
}

/// Remove SRT indices/timestamps and collapse to one long paragraph
fn clean_srt(text: &str) -> String {
    let timestamp_re = Regex::new(r"\d{2}:\d{2}:\d{2},\d{3} --> \d{2}:\d{2}:\d{2},\d{3}").unwrap();
    let seq_re = Regex::new(r"^\d+$").unwrap();

    text.lines()
        .filter(|line| {
            let t = line.trim();
            !t.is_empty() && !seq_re.is_match(t) && !timestamp_re.is_match(t)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Simple word-based splitter with overlap
fn split_text(text: &str, chunk_size: usize, chunk_overlap: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < words.len() {
        let end = usize::min(start + chunk_size, words.len());
        chunks.push(words[start..end].join(" "));
        if end == words.len() {
            break;
        }
        start += chunk_size.saturating_sub(chunk_overlap);
    }

    chunks
}
