#!/bin/bash
set -e

# Go to project root
cd "$(dirname "$0")/.."

# Demos output directory
OUT_DIR="website/public/demos"
mkdir -p "$OUT_DIR"

# English
echo "=== Rendering English Demos ==="
LANG=en_US.UTF-8 cargo run -p media-sort-gui --features demo -- --export --demo-spec resources/demo_flows/sorting_workflow.json --demo-export "$OUT_DIR/sorting_workflow_en.mp4"
LANG=en_US.UTF-8 cargo run -p media-sort-gui --features demo -- --export --demo-spec resources/demo_flows/change_keybinding.json --demo-export "$OUT_DIR/change_keybinding_en.mp4"

# German
echo "=== Rendering German Demos ==="
LANG=de_DE.UTF-8 cargo run -p media-sort-gui --features demo -- --export --demo-spec resources/demo_flows/sorting_workflow.json --demo-export "$OUT_DIR/sorting_workflow_de.mp4"
LANG=de_DE.UTF-8 cargo run -p media-sort-gui --features demo -- --export --demo-spec resources/demo_flows/change_keybinding.json --demo-export "$OUT_DIR/change_keybinding_de.mp4"

# Japanese
echo "=== Rendering Japanese Demos ==="
LANG=ja_JP.UTF-8 cargo run -p media-sort-gui --features demo -- --export --demo-spec resources/demo_flows/sorting_workflow.json --demo-export "$OUT_DIR/sorting_workflow_ja.mp4"
LANG=ja_JP.UTF-8 cargo run -p media-sort-gui --features demo -- --export --demo-spec resources/demo_flows/change_keybinding.json --demo-export "$OUT_DIR/change_keybinding_ja.mp4"

echo "=== All Demos Rendered ==="
