#!/usr/bin/env python3
"""Convert BUI 2.x JSON fixtures to BUI 3.0-IR format.

Usage:
    python3 convert_2x_to_ir.py <input.json> <output.ir.json>
    python3 convert_2x_to_ir.py --batch <directory>

The conversion is a lossless bijection (field path restructuring):
  - version: "2.0" → "3.0-ir"
  - type: "Node"/"Text"/... → kind: "node"/"text"/... (lowercase)
  - styles → layout.styles
  - visuals → style.visuals
  - text_config → content.text
  - image_config → content.image
  - custom_tags → markers
  - tab_group_name etc → semantics.tab_group_name etc
  - Adds empty imports, state_model, resources
"""

import json
import sys
import os
import glob

TYPE_MAP = {
    "Node": "node",
    "Text": "text",
    "TextInput": "text_input",
    "Toggle": "toggle",
    "Button": "button",
    "Image": "image",
}

def convert_node(node):
    """Convert a BuiNode (2.x) dict to BuiNode (3.0-IR) dict."""
    kind = TYPE_MAP.get(node.get("type", "Node"), "node")

    layout = {}
    if "styles" in node and node["styles"]:
        layout["styles"] = node["styles"]

    style = {}
    if "visuals" in node and node["visuals"]:
        style["visuals"] = node["visuals"]

    content = {}
    if "text_config" in node and node["text_config"]:
        content["text"] = node["text_config"]
    if "image_config" in node and node["image_config"]:
        content["image"] = node["image_config"]

    semantics = {}
    for field in ["tab_group_name", "tab_binding_source", "tab_value",
                  "progress_binding_source", "progress_fill", "list_binding_source"]:
        if field in node and node[field] is not None:
            semantics[field] = node[field]

    result = {
        "id": node.get("id", ""),
        "kind": kind,
    }

    if "custom_tags" in node and node["custom_tags"]:
        result["markers"] = node["custom_tags"]
    if "actions" in node and node["actions"]:
        result["actions"] = node["actions"]
    if "bindings" in node and node["bindings"]:
        result["bindings"] = node["bindings"]

    if layout:
        result["layout"] = layout
    if style:
        result["style"] = style
    if content:
        result["content"] = content
    if semantics:
        result["semantics"] = semantics

    if "state_visuals" in node and node["state_visuals"]:
        result["state_visuals"] = node["state_visuals"]

    if "children" in node and node["children"]:
        result["children"] = [convert_node(child) for child in node["children"]]

    return result

def convert_document(doc):
    """Convert a BuiDocument (2.x) dict to BuiDocument (3.0-IR) dict."""
    return {
        "version": "3.0-ir",
        "scene_name": doc.get("scene_name", ""),
        "imports": [],
        "state_model": {},
        "resources": {},
        "root": convert_node(doc["root"]),
    }

def convert_file(input_path, output_path):
    """Convert a single JSON file."""
    with open(input_path, "r", encoding="utf-8") as f:
        doc = json.load(f)

    ir_doc = convert_document(doc)

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(ir_doc, f, indent=2, ensure_ascii=False)
        f.write("\n")

    print(f"  Converted: {input_path} → {output_path}")

def batch_convert(directory):
    """Convert all .json files in a directory (not .ir.json)."""
    for json_path in glob.glob(os.path.join(directory, "**", "*.json"), recursive=True):
        if json_path.endswith(".ir.json"):
            continue
        ir_path = json_path.replace(".json", ".ir.json")
        convert_file(json_path, ir_path)

def main():
    if len(sys.argv) < 2:
        print("Usage: convert_2x_to_ir.py <input.json> <output.ir.json>")
        print("       convert_2x_to_ir.py --batch <directory>")
        sys.exit(1)

    if sys.argv[1] == "--batch":
        batch_convert(sys.argv[2])
    else:
        convert_file(sys.argv[1], sys.argv[2])

if __name__ == "__main__":
    main()