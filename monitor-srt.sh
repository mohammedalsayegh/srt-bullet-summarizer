#!/bin/bash
#
# Author: Mohammed H Alsaeygh
# Project: srt-bullet-summarizer
#
# Description:
# This Bash script continuously monitors a target directory (e.g., Downloads) for new `.srt` files.
# When a new subtitle file is detected, it invokes the `srt-bullet-summarizer` Rust CLI tool to
# generate a bullet-point summary using a local LLM (e.g., LLaMA 3.2 via an OpenAI-compatible API).
# Once processed, both the original `.srt` and its resulting `_summary.txt` are moved to an archive folder.
#
# Features:
# - Live GUI display of processing status using YAD
# - Automatically detects and processes new `.srt` files
# - Handles success/failure logging for each file
# - Avoids redundant "Waiting" messages when idle
#
# Requirements:
# - `yad` for GUI status output
# - `srt-bullet-summarizer` binary in the same directory or system PATH
#
# How to Use:
# 1. Ensure `srt-bullet-summarizer` is built and accessible.
# 2. Run this script: `./srt_monitor.sh`
#
# Notes:
# - Summaries are saved alongside the `.srt` with `_summary.txt` suffix.
# - After processing, files are moved into `~/Downloads/srt/`.

WATCH_DIR="$HOME/Downloads"
TARGET_DIR="$WATCH_DIR/srt"
mkdir -p "$TARGET_DIR"

STATUS_FILE=$(mktemp)
echo "🟢 SRT Monitor Initialized..." > "$STATUS_FILE"

already_waiting=false

# Function to truncate status file if too large
truncate_status_file() {
    if [ "$(wc -l < "$STATUS_FILE")" -gt 100 ]; then
        tail -n 50 "$STATUS_FILE" > "${STATUS_FILE}.tmp"
        mv "${STATUS_FILE}.tmp" "$STATUS_FILE"
    fi
}

# Launch live-updating YAD window
tail -f "$STATUS_FILE" | yad --text-info \
    --title="📝 SRT Monitor (Close to stop)" \
    --window-icon=dialog-information \
    --width=500 --height=300 \
    --fontname="monospace" &

YAD_PID=$!

# Main monitor loop
while kill -0 "$YAD_PID" 2>/dev/null; do
    found_files=false

    while IFS= read -r srt_file; do
        found_files=true
        already_waiting=false

        base_name="$(basename "$srt_file" .srt)"
        summary_file="${WATCH_DIR}/${base_name}_summary.txt"

        echo "🔄 Processing: $base_name.srt" >> "$STATUS_FILE"

        if ./srt-bullet-summarizer "$srt_file"; then
            mv "$srt_file" "$TARGET_DIR/"

            if [ -f "$summary_file" ]; then
                mv "$summary_file" "$TARGET_DIR/"
                echo "✅ Success: $(basename "$summary_file") moved" >> "$STATUS_FILE"
            else
                echo "⚠️ Warning: _summary.txt not found for $base_name" >> "$STATUS_FILE"
            fi
        else
            echo "❌ Error: Failed to process $base_name.srt" >> "$STATUS_FILE"
        fi

        truncate_status_file
    done < <(find "$WATCH_DIR" -maxdepth 1 -type f -iname "*.srt")

    if ! $found_files && ! $already_waiting; then
        echo "⏳ Waiting for new .srt files..." >> "$STATUS_FILE"
        already_waiting=true
    fi

    truncate_status_file
    sleep 5
done

# Cleanup if YAD is closed
echo "🛑 Monitor stopped by user." >> "$STATUS_FILE"
rm -f "$STATUS_FILE"
