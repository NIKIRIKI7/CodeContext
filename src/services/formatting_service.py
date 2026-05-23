import html
import os
from pathlib import Path
from typing import List, Dict, Set, Optional, Any

try:
    from jinja2 import Template, Environment, FileSystemLoader, select_autoescape

    JINJA_AVAILABLE = True
except ImportError:
    JINJA_AVAILABLE = False
    Template = Environment = FileSystemLoader = select_autoescape = None  # type: ignore

from ..store.state import ProcessedFile


class FormattingService:
    """Сервис форматирования итогового текста с поддержкой Jinja2"""

    def format_output(self,
                      files: List[ProcessedFile],
                      fmt: str,
                      include_tree: bool,
                      system_prompt: str,
                      dependency_map: Optional[Dict[str, Set[str]]] = None,
                      template_path: str = "") -> str:
        tree_text = self._generate_tree([f.path for f in files]) if include_tree else ""
        dep_text = self._format_dependency_graph(dependency_map) if dependency_map else ""

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
                output.append(f"### Project Structure\n```text\n{tree_text}\n```\n")
            elif fmt == 'xml':
                output.append(f"<tree>\n{html.escape(tree_text)}\n</tree>")
            else:
                output.append("PROJECT STRUCTURE:\n" + tree_text + "\n" + "=" * 50 + "\n")

        if dependency_map:
            if dep_text:
                if fmt == 'markdown':
                    output.append(f"### Dependency Graph\n```text\n{dep_text}\n```\n")
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
    def _render_custom_template(template_path: str, **context: Any) -> str:
        """Рендер Jinja2 шаблона из файла"""
        if Environment is None or FileSystemLoader is None or select_autoescape is None:
            return "Error: Jinja2 is not available."

        try:
            template_dir = os.path.dirname(template_path)
            template_file = os.path.basename(template_path)

            env = Environment(
                loader=FileSystemLoader(template_dir),
                autoescape=select_autoescape(['html', 'xml'])
            )
            template = env.get_template(template_file)
            return template.render(**context)
        except Exception as e:
            return f"Error rendering template: {str(e)}"

    @staticmethod
    def _format_dependency_graph(dep_map: Dict[str, Set[str]]) -> str:
        """Генерация текстового представления графа из словаря"""
        if not dep_map:
            return ""
        lines = []
        sorted_files = sorted(dep_map.keys())
        for file_path in sorted_files:
            imports = dep_map[file_path]
            if not imports:
                continue
            display_name = os.path.basename(file_path)
            lines.append(f"{display_name}")
            for imp in sorted(list(imports)):
                lines.append(f"  -> {imp}")
            lines.append("")
        return "\n".join(lines)

    @staticmethod
    def _generate_tree(paths: List[str]) -> str:
        """Генерация ASCII дерева"""
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
            out.append(f"### `{f.path}`\n")
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