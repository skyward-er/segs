# /// script
# requires-python = ">=3.11"
# ///
from __future__ import annotations

import re
import os
from pathlib import Path

if (sources_path := os.environ.get("SOURCES_PATH")) is None:
    raise SystemExit("SOURCES_PATH environment variable is not set")

if (assets_dir := os.environ.get("ASSETS_DIR")) is None:
    raise SystemExit("ASSETS_DIR environment variable is not set")

sources_path = Path(sources_path)
assets_dir = Path(assets_dir)

start_marker = "// ~~~\u00a0START AUTOMATICALLY GENERATED CODE ~~~"
end_marker = "// ~~~\u00a0END AUTOMATICALLY GENERATED CODE ~~~"


def snake_case(value: str) -> str:
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", value)
    value = re.sub(r"[^0-9a-zA-Z]+", "_", value)
    return value.strip("_")


def const_name(stem: str) -> str:
    name = snake_case(stem).upper()
    if name and name[0].isdigit():
        name = f"_{name}"
    return name


def mod_name(segment: str) -> str:
    name = snake_case(segment).lower()
    if not name:
        name = "assets"
    if name[0].isdigit():
        name = f"mod_{name}"
    return name


tree: dict[str, dict] = {}
for path in sorted(assets_dir.rglob("*")):
    if not path.is_file():
        continue
    if path.name == ".DS_Store":
        continue
    rel = path.relative_to(assets_dir)
    node = tree
    for part in rel.parts[:-1]:
        node = node.setdefault(part, {})
    node.setdefault("__files__", []).append(rel)


def emit(node: dict, indent: int = 0) -> list[str]:
    lines: list[str] = []
    dirs = sorted(k for k in node.keys() if k != "__files__")
    for dir_name in dirs:
        module = mod_name(dir_name)
        lines.append("#[rustfmt::skip]")
        lines.append(" " * indent + f"pub mod {module} {{")
        lines.extend(emit(node[dir_name], indent + 4))
        lines.append(" " * indent + "}")
        if indent == 0:
            lines.append("")
    for rel in sorted(node.get("__files__", [])):
        name = const_name(rel.stem)
        segments = ", ".join(f'"{segment}"' for segment in rel.parts)
        if rel.parts and rel.parts[0] == "icons":
            lines.append(
                " " * indent + f"pub static {name}: &[u8] = include_asset!({segments});"
            )
        elif rel.suffix.lower() == ".svg":
            lines.append(
                " " * indent + f"pub static {name}: egui::ImageSource = include_svg!({segments});"
            )
        elif rel.suffix.lower() in {".ttf", ".otf"}:
            lines.append(
                " " * indent + f"pub static {name}: &[u8] = include_asset!({segments});"
            )
    if indent == 0 and lines and lines[-1] == "":
        lines.pop()
    return lines


generated_lines = emit(tree)
replacement = start_marker + "\n" + "\n".join(generated_lines) + "\n" + end_marker

content = sources_path.read_text()
pattern = re.compile(re.escape(start_marker) + r".*?" + re.escape(end_marker), re.S)
if not pattern.search(content):
    raise SystemExit("Could not find generated code markers in sources.rs")

sources_path.write_text(pattern.sub(replacement, content))
print(f"Updated {sources_path}")
