import html
import json
import os
import re
import difflib
from pathlib import Path
from typing import List, Dict, Set, Optional, Any
from functools import lru_cache

try:
    from jinja2 import Template, Environment, FileSystemLoader, select_autoescape
    JINJA_AVAILABLE = True
except ImportError:
    JINJA_AVAILABLE = False
    Template = Environment = FileSystemLoader = select_autoescape = None

from ..i18n import tr
from ..store.state import ProcessedFile

def format_output(
    files: List[ProcessedFile], fmt: str, include_tree: bool, system_prompt: str,
    dependency_map: Optional[Dict[str, Set[str]]] = None, template_path: str = "",
    include_mermaid: bool = False, deduplicate: bool = False
) -> str:
    if fmt == 'jsonl_chunk': return _to_jsonl_chunks(files)
    if fmt == 'jsonl_mini': return _to_jsonl_mini(files)

    if deduplicate and len(files) > 1:
        files = _deduplicate(files)

    tree_text = _generate_tree([f.path for f in files]) if include_tree else ""
    dep_text = _format_dependency_graph(dependency_map) if dependency_map else ""
    mermaid_text = _format_mermaid_graph(dependency_map) if (dependency_map and include_mermaid) else ""

    if fmt == 'custom':
        if not JINJA_AVAILABLE: return "Error: jinja2 library is not installed."
        if not template_path or not os.path.exists(template_path): return "Error: Template file not found."
        return _render_custom_template(template_path, files=files, tree=tree_text, dependencies=dependency_map, dependencies_text=dep_text, system_prompt=system_prompt)

    output = []
    if system_prompt:
        if fmt == 'xml': output.append(f"<system_prompt>\n{html.escape(system_prompt)}\n</system_prompt>\n")
        elif fmt == 'markdown': output.append(f"> **System Context:**\n> {system_prompt}\n\n---\n")
        else: output.append(f"SYSTEM PROMPT:\n{system_prompt}\n" + "=" * 50 + "\n")

    if include_tree and files:
        if fmt == 'markdown': output.append(f"## Directory Structure\n```text\n{tree_text}\n```\n")
        elif fmt == 'xml': output.append(f"<tree>\n{html.escape(tree_text)}\n</tree>")
        else: output.append("PROJECT STRUCTURE:\n" + tree_text + "\n" + "=" * 50 + "\n")

    if dependency_map:
        if dep_text:
            if fmt == 'markdown': output.append(f"## Dependencies\n```text\n{dep_text}\n```\n")
        if mermaid_text:
            output.append(f"## Mermaid Graph\n{mermaid_text}\n")
        elif fmt == 'xml':
            output.append(f"<dependencies>\n{html.escape(dep_text)}\n</dependencies>")
        else:
            output.append("DEPENDENCY GRAPH:\n" + dep_text + "\n" + "=" * 50 + "\n")

    if fmt == 'xml': output.append(_to_xml(files, dependency_map))
    elif fmt == 'markdown': output.append(_to_markdown(files, dependency_map))
    else: output.append(_to_plain(files, dependency_map))

    return "\n".join(output)

def _deduplicate(files: List[ProcessedFile]) -> List[ProcessedFile]:
    if not files: return files
    result = [files[0]]
    for i in range(1, len(files)):
        prev, curr = files[i - 1], files[i]
        if curr.content == prev.content:
            result.append(ProcessedFile(path=f"{curr.path}  (same content as {Path(prev.path).name})", content="", tokens=0))
        else:
            result.append(curr)
    return result

def _to_jsonl_mini(files: List[ProcessedFile]) -> str:
    return "\n".join(json.dumps({"p": f.path, "c": f.content, "t": f.tokens}, ensure_ascii=False, separators=(',', ':')) for f in files)

def get_search_markers(filepath: str) -> List[str]:
    return [f"FILE: {filepath}", f"## File: {filepath}", f'<file path="{filepath}">', filepath]

def generate_html_diff(source_text: str, target_text: str, colors: dict, fonts: dict) -> str:
    source_lines = source_text.splitlines()
    target_lines = target_text.splitlines()
    start = 0
    while start < len(source_lines) and start < len(target_lines) and source_lines[start] == target_lines[start]: start += 1
    end_s, end_t = len(source_lines) - 1, len(target_lines) - 1
    while end_s > start and end_t > start and source_lines[end_s] == target_lines[end_t]: end_s -= 1; end_t -= 1
    c_start = max(0, start - 5)

    html_diff = difflib.HtmlDiff(wrapcolumn=90).make_file(
        source_lines[c_start:min(len(source_lines), end_s + 6)],
        target_lines[c_start:min(len(target_lines), end_t + 6)],
        context=True, numlines=5
    )
    if c_start > 0:
        msg = tr("formatting.hidden_lines", default=f"... {c_start} identical lines hidden ...", count=c_start)
        html_diff = html_diff.replace('<body>', f'<body><div style="color:gray;padding:5px;">{msg}</div>')

    font_family = fonts.get("family", "monospace")
    custom_css = f'<style type="text/css">\ntable.diff {{font-family:{font_family}; font-size:12px; border:none; width:100%;}}\n.diff_add {{background-color:{colors.get("diff_add","#e6ffed")};}}\n.diff_sub {{background-color:{colors.get("diff_sub","#ffeef0")};}}\n.diff_chg {{background-color:{colors.get("diff_chg","#fff5b1")};}}\n.diff_header {{background-color:{colors.get("diff_hdr","#f0f0f0")}; color:#999;}}\ntd {{padding:2px 4px;}}\n</style>'
    return re.sub(r'<style type="text/css">.*?</style>', custom_css, html_diff, flags=re.DOTALL)

@lru_cache(maxsize=1)
def _get_jinja_env(template_dir: str):
    return Environment(loader=FileSystemLoader(template_dir), autoescape=select_autoescape(['html', 'xml']))

def _render_custom_template(template_path: str, **context: Any) -> str:
    try:
        template_dir = os.path.dirname(template_path)
        env = _get_jinja_env(template_dir)
        return env.get_template(os.path.basename(template_path)).render(**context)
    except Exception as e:
        return f"Error rendering template: {str(e)}"

def _format_mermaid_graph(dep_map: Dict[str, Set[str]]) -> str:
    if not dep_map: return ""
    lines = ["```mermaid", "graph TD;"]
    for file_path, imports in dep_map.items():
        if not imports: continue
        file_name = os.path.basename(file_path).replace('"', "'")
        for imp in sorted(list(imports)):
            lines.append(f'  "{file_name}" --> "{imp.replace("\"", "")}";')
    lines.append("```\n")
    return "\n".join(lines)

def _format_dependency_graph(dep_map: Dict[str, Set[str]]) -> str:
    if not dep_map: return ""
    lines = []
    for file_path in sorted(dep_map.keys()):
        imports = dep_map[file_path]
        if imports:
            lines.append(f"{os.path.basename(file_path)}")
            for imp in sorted(list(imports)): lines.append(f"  -> {imp}")
            lines.append("")
    return "\n".join(lines)

def _to_jsonl_chunks(files: List[ProcessedFile]) -> str:
    lines_out = []
    for f in files:
        content_lines = f.content.splitlines()
        if not content_lines: continue
        for i in range(0, len(content_lines), 150):
            chunk_str = "\n".join(content_lines[i:i + 150])
            lines_out.append(json.dumps({
                "file_path": f.path, "chunk_index": i // 150, "total_chunks": (len(content_lines) // 150) + 1,
                "tokens_approx": len(chunk_str) // 4, "content": chunk_str
            }, ensure_ascii=False))
    return "\n".join(lines_out)

def _generate_tree(paths: List[str]) -> str:
    if not paths: return ""
    base_dir = None
    try:
        common_path = os.path.commonpath(paths)
        if os.path.isfile(common_path): common_path = os.path.dirname(common_path)
        base_dir = Path(common_path).parent
    except Exception: pass

    tree_structure = {}
    for path in sorted(paths):
        p = Path(path)
        rel_parts = p.relative_to(base_dir).parts if base_dir else p.parts
        current = tree_structure
        for part in rel_parts: current = current.setdefault(part, {})

    lines = []
    def _build(structure, prefix=""):
        items = sorted(list(structure.keys()))
        for i, name in enumerate(items):
            is_last = (i == len(items) - 1)
            lines.append(f"{prefix}{'└── ' if is_last else '├── '}{name}")
            if structure[name]: _build(structure[name], prefix + ("    " if is_last else "│   "))
    _build(tree_structure)
    return "\n".join(lines)

def _to_xml(files: List[ProcessedFile], deps: Optional[Dict[str, Set[str]]]) -> str:
    out = ["<root>"]
    for f in files:
        out.append(f'  <file path="{f.path}">')
        if deps and f.path in deps and deps[f.path]:
            out.append('    <imports>')
            for d in sorted(list(deps[f.path])): out.append(f'      <import>{html.escape(d)}</import>')
            out.append('    </imports>')
        out.append(f'    <content>\n{html.escape(f.content)}\n    </content>\n  </file>')
    out.append("</root>")
    return "\n".join(out)

def _to_markdown(files: List[ProcessedFile], deps: Optional[Dict[str, Set[str]]]) -> str:
    out = []
    for f in files:
        ext = Path(f.path).suffix.lstrip('.') or 'txt'
        out.append(f"## File: {f.path}")
        if deps and f.path in deps and deps[f.path]:
            out.append(f"**Detected Imports:** {', '.join([f'`{d}`' for d in sorted(list(deps[f.path]))])}\n")
        out.append(f"```{ext}\n{f.content}\n```\n")
    return "\n".join(out)

def _to_plain(files: List[ProcessedFile], deps: Optional[Dict[str, Set[str]]]) -> str:
    out = []
    for f in files:
        out.append("=" * 50 + f"\nFILE: {f.path}")
        if deps and f.path in deps and deps[f.path]:
            out.append(f"IMPORTS: {', '.join(sorted(list(deps[f.path])))}")
        out.append("=" * 50 + f"\n{f.content}\n")
    return "\n".join(out)
