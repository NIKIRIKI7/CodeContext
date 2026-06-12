import html
import json
import os
import re
import difflib
from pathlib import Path
from typing import List, Dict, Set, Optional, Any

try:
    from jinja2 import Template, Environment, FileSystemLoader, select_autoescape

    JINJA_AVAILABLE = True
except ImportError:
    JINJA_AVAILABLE = False
    Template = Environment = FileSystemLoader = select_autoescape = None

from ..i18n import tr
from ..store.state import ProcessedFile


class FormattingService:
    """Сервис форматирования итогового текста с поддержкой Jinja2 и генерацией Diff"""

    _env_cache: dict[str, Any] = {}

    def format_output(self,
                      files: List[ProcessedFile],
                      fmt: str,
                      include_tree: bool,
                      system_prompt: str,
                      dependency_map: Optional[Dict[str, Set[str]]] = None,
                      template_path: str = "",
                      include_mermaid: bool = False,
                      deduplicate: bool = False) -> str:
        if fmt == 'jsonl_chunk':
            return self._to_jsonl_chunks(files)
        if fmt == 'jsonl_mini':
            return self._to_jsonl_mini(files)

        if deduplicate and len(files) > 1:
            files = self._deduplicate(files)

        tree_text = self._generate_tree([f.path for f in files]) if include_tree else ""
        dep_text = self._format_dependency_graph(dependency_map) if dependency_map else ""
        mermaid_text = self._format_mermaid_graph(dependency_map) if (dependency_map and include_mermaid) else ""

        if fmt == 'custom':
            if not JINJA_AVAILABLE:
                return "Error: jinja2 library is not installed. Run 'pip install jinja2'."
            if not template_path or not os.path.exists(template_path):
                return "Error: Template file not found. Please select a .jinja2 file."
            return self._render_custom_template(
                template_path,
                files=files,
                tree=tree_text,
                dependencies=dependency_map,
                dependencies_text=dep_text,
                system_prompt=system_prompt
            )

        output = []
        if system_prompt:
            if fmt == 'xml':
                output.append(f"<system_prompt>\n{html.escape(system_prompt)}\n</system_prompt>\n")
            elif fmt == 'markdown':
                output.append(f"> **System Context:**\n> {system_prompt}\n\n---\n")
            else:
                output.append(f"SYSTEM PROMPT:\n{system_prompt}\n" + "=" * 50 + "\n")

        if include_tree and files:
            if fmt == 'markdown':
                output.append(f"### PROJECT STRUCTURE\n```text\n{tree_text}\n```\n")
            elif fmt == 'xml':
                output.append(f"<tree>\n{html.escape(tree_text)}\n</tree>")
            else:
                output.append("PROJECT STRUCTURE:\n" + tree_text + "\n" + "=" * 50 + "\n")

        if dependency_map:
            if dep_text:
                if fmt == 'markdown':
                    output.append(f"### DEPENDENCY GRAPH\n```text\n{dep_text}\n```\n")
                    if mermaid_text:
                        output.append(f"### ARCHITECTURE DIAGRAM\n{mermaid_text}\n")
                elif fmt == 'xml':
                    output.append(f"<dependencies>\n{html.escape(dep_text)}\n</dependencies>")
                else:
                    output.append("DEPENDENCY GRAPH:\n" + dep_text + "\n" + "=" * 50 + "\n")

        if fmt == 'xml':
            output.append(self._to_xml(files, dependency_map))
        elif fmt == 'markdown':
            output.append(self._to_markdown(files, dependency_map))
        else:
            output.append(self._to_plain(files, dependency_map))

        return "\n".join(output)

    @staticmethod
    def _deduplicate(files: List[ProcessedFile]) -> List[ProcessedFile]:
        """Удаляет дубликаты содержимого соседних файлов, оставляя пометку"""
        if not files:
            return files
        result = [files[0]]
        for i in range(1, len(files)):
            prev = files[i - 1]
            curr = files[i]
            if curr.content == prev.content:
                dup = ProcessedFile(
                    path=f"{curr.path}  (same content as {Path(prev.path).name})",
                    content="",
                    tokens=0
                )
                result.append(dup)
            else:
                result.append(curr)
        return result

    @staticmethod
    def _to_jsonl_mini(files: List[ProcessedFile]) -> str:
        """Ультра-компактный JSONL: каждый файл — одна строка с минимальными ключами"""
        lines_out = []
        for f in files:
            obj = {
                "p": f.path,
                "c": f.content,
                "t": f.tokens,
            }
            lines_out.append(json.dumps(obj, ensure_ascii=False, separators=(',', ':')))
        return "\n".join(lines_out)

    @staticmethod
    def get_search_markers(filepath: str) -> List[str]:
        """Возвращает возможные варианты заголовка файла для навигации в предпросмотре."""
        return [
            f"FILE: {filepath}",  # plain format
            f"### FILE: {filepath}",  # markdown format
            f'<file path="{filepath}">',  # xml format
            filepath  # fallback
        ]

    @staticmethod
    def generate_html_diff(source_text: str, target_text: str, colors: dict, fonts: dict) -> str:
        """Создает HTML Diff с заданными цветами темы и схлопыванием идентичных краёв."""
        font_family = fonts.get("family", "monospace")
        font_size = fonts.get("size", "14px")

        source_lines = source_text.splitlines()
        target_lines = target_text.splitlines()

        start = 0
        while start < len(source_lines) and start < len(target_lines) and source_lines[start] == target_lines[start]:
            start += 1

        end_s = len(source_lines) - 1
        end_t = len(target_lines) - 1
        while end_s > start and end_t > start and source_lines[end_s] == target_lines[end_t]:
            end_s -= 1
            end_t -= 1

        ctx = 5
        c_start = max(0, start - ctx)
        c_end_s = min(len(source_lines), end_s + ctx + 1)
        c_end_t = min(len(target_lines), end_t + ctx + 1)

        html_diff = difflib.HtmlDiff(wrapcolumn=90).make_file(
            source_lines[c_start:c_end_s],
            target_lines[c_start:c_end_t],
            context=True, numlines=5
        )

        if c_start > 0:
            msg = tr("formatting.hidden_lines", default=f"... {c_start} identical lines hidden ...", count=c_start)
            html_diff = html_diff.replace('<body>', f'<body><div style="color:gray;padding:5px;">{msg}</div>')

        custom_css = f'''
        <style type="text/css">
            body {{
                font-family: {font_family};
                font-size: {font_size};
                background-color: {colors.get('bg', '#ffffff')};
                color: {colors.get('text', '#000000')};
            }}
            table.diff {{ width: 100%; border-collapse: collapse; font-family: {font_family}; }}
            table.diff tbody {{ font-family: {font_family}; }}
            table.diff td {{ padding: 2px 4px; border: 1px solid {colors.get('border', '#cccccc')}; }}
            table.diff th {{ background-color: {colors.get('diff_hdr', '#f0f0f0')}; border: 1px solid {colors.get('border', '#cccccc')}; color: {colors.get('text', '#000000')}; }}
            table.diff .diff_add {{ background-color: {colors.get('diff_add', '#e6ffed')}; color: {colors.get('text', '#000000')}; }}
            table.diff .diff_sub {{ background-color: {colors.get('diff_sub', '#ffeef0')}; color: {colors.get('text', '#000000')}; }}
            table.diff .diff_chg {{ background-color: {colors.get('diff_chg', '#fff5b1')}; color: {colors.get('text', '#000000')}; }}
            table.diff .diff_header {{ background-color: {colors.get('diff_hdr', '#f0f0f0')}; }}
            table.diff .diff_next {{ background-color: {colors.get('diff_hdr', '#f0f0f0')}; }}
        </style>
        '''
        html_diff = re.sub(r'<style type="text/css">.*?</style>', custom_css, html_diff, flags=re.DOTALL)
        return html_diff

    @staticmethod
    def _render_custom_template(template_path: str, **context: Any) -> str:
        if Environment is None or FileSystemLoader is None or select_autoescape is None:
            return "Error: Jinja2 is not available."
        try:
            template_dir = os.path.dirname(template_path)
            template_file = os.path.basename(template_path)

            if template_dir not in FormattingService._env_cache:
                env = Environment(
                    loader=FileSystemLoader(template_dir),
                    autoescape=select_autoescape(['html', 'xml'])
                )
                FormattingService._env_cache[template_dir] = env

            env = FormattingService._env_cache[template_dir]
            template = env.get_template(template_file)
            return template.render(**context)
        except Exception as e:
            return f"Error rendering template: {str(e)}"

    @staticmethod
    def _format_mermaid_graph(dep_map: Dict[str, Set[str]]) -> str:
        if not dep_map:
            return ""
        lines = ["```mermaid", "graph TD;"]
        for file_path, imports in dep_map.items():
            if not imports:
                continue
            file_name = os.path.basename(file_path)
            for imp in sorted(list(imports)):
                safe_name = file_name.replace('"', "'")
                safe_imp = imp.replace('"', "'")
                lines.append(f'  "{safe_name}" --> "{safe_imp}";')
        lines.append("```\n")
        return "\n".join(lines)

    @staticmethod
    def _format_dependency_graph(dep_map: Dict[str, Set[str]]) -> str:
        if not dep_map:
            return ""
        lines = []
        sorted_files = sorted(dep_map.keys())
        for file_path in sorted_files:
            imports = dep_map[file_path]
            if not imports: continue
            display_name = os.path.basename(file_path)
            lines.append(f"{display_name}")
            for imp in sorted(list(imports)):
                lines.append(f"  -> {imp}")
            lines.append("")
        return "\n".join(lines)

    @staticmethod
    def _to_jsonl_chunks(files: List[ProcessedFile]) -> str:
        lines_out = []
        chunk_line_limit = 150
        for f in files:
            content_lines = f.content.splitlines()
            if not content_lines:
                continue
            for i in range(0, len(content_lines), chunk_line_limit):
                chunk_str = "\n".join(content_lines[i:i + chunk_line_limit])
                tokens_approx = len(chunk_str) // 4
                chunk_data = {
                    "file_path": f.path,
                    "chunk_index": i // chunk_line_limit,
                    "total_chunks": (len(content_lines) // chunk_line_limit) + 1,
                    "tokens_approx": tokens_approx,
                    "content": chunk_str
                }
                lines_out.append(json.dumps(chunk_data, ensure_ascii=False))
        return "\n".join(lines_out)

    @staticmethod
    def _generate_tree(paths: List[str]) -> str:
        if not paths: return ""
        base_dir: Optional[Path] = None
        try:
            common_path = os.path.commonpath(paths)
            if os.path.isfile(common_path):
                common_path = os.path.dirname(common_path)
            base_dir = Path(common_path).parent
        except (ValueError, OSError):
            pass

        tree_structure: Dict[str, Any] = {}
        for path in sorted(paths):
            p = Path(path)
            if base_dir:
                try:
                    rel_parts = p.relative_to(base_dir).parts
                except ValueError:
                    rel_parts = p.parts
            else:
                rel_parts = p.parts

            current = tree_structure
            for part in rel_parts:
                current = current.setdefault(part, {})

        lines = []

        def _build(structure: Dict[str, Any], prefix: str = ""):
            items = sorted(list(structure.keys()))
            for i, name in enumerate(items):
                is_last = (i == len(items) - 1)
                connector = "└── " if is_last else "├── "
                lines.append(f"{prefix}{connector}{name}")
                children = structure[name]
                if children:
                    extension = "    " if is_last else "│   "
                    _build(children, prefix + extension)

        if not base_dir:
            lines.append("Project Structure (Absolute Paths):")
        _build(tree_structure)
        return "\n".join(lines)

    @staticmethod
    def _to_xml(files: List[ProcessedFile], deps: Optional[Dict[str, Set[str]]]) -> str:
        out = ["<root>"]
        for f in files:
            out.append(f'  <file path="{f.path}">')
            if deps and f.path in deps:
                sorted_deps = sorted(list(deps[f.path]))
                if sorted_deps:
                    out.append('    <imports>')
                    for d in sorted_deps:
                        out.append(f'      <import>{html.escape(d)}</import>')
                    out.append('    </imports>')
            out.append(f'    <content>\n{html.escape(f.content)}\n    </content>')
            out.append(f'  </file>')
        out.append("</root>")
        return "\n".join(out)

    @staticmethod
    def _to_markdown(files: List[ProcessedFile], deps: Optional[Dict[str, Set[str]]]) -> str:
        out = []
        for f in files:
            ext = Path(f.path).suffix.lstrip('.') or 'txt'
            out.append(f"### FILE: {f.path}")
            if deps and f.path in deps:
                sorted_deps = sorted(list(deps[f.path]))
                if sorted_deps:
                    deps_str = ", ".join([f"`{d}`" for d in sorted_deps])
                    out.append(f"**Detected Imports:** {deps_str}\n")
            out.append(f"```{ext}")
            out.append(f.content)
            out.append("```\n")
        return "\n".join(out)

    @staticmethod
    def _to_plain(files: List[ProcessedFile], deps: Optional[Dict[str, Set[str]]]) -> str:
        out = []
        sep = "=" * 50
        for f in files:
            out.append(sep)
            out.append(f"FILE: {f.path}")
            if deps and f.path in deps:
                sorted_deps = sorted(list(deps[f.path]))
                if sorted_deps:
                    out.append(f"IMPORTS: {', '.join(sorted_deps)}")
            out.append(sep)
            out.append(f.content)
            out.append("\n")
        return "\n".join(out)
