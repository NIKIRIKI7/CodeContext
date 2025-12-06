import html
import os
from pathlib import Path
from typing import List, Dict, Set, Optional
from ..store.state import ProcessedFile


class FormattingService:
    """Сервис форматирования итогового текста"""

    def format_output(self,
                      files: List[ProcessedFile],
                      fmt: str,
                      include_tree: bool,
                      system_prompt: str,
                      dependency_map: Optional[Dict[str, Set[str]]] = None) -> str:
        output = []

        # 1. System Prompt
        if system_prompt:
            if fmt == 'xml':
                output.append(f"<system_prompt>\n{html.escape(system_prompt)}\n</system_prompt>\n")
            elif fmt == 'markdown':
                output.append(f"> **System Context:**\n> {system_prompt}\n\n---\n")
            else:
                output.append(f"SYSTEM PROMPT:\n{system_prompt}\n" + "=" * 50 + "\n")

        # 2. Project Tree
        if include_tree and files:
            paths = [f.path for f in files]
            tree = self._generate_tree(paths)
            if fmt == 'markdown':
                # ИСПРАВЛЕНО: Восстановлена строка вывода дерева для Markdown
                output.append(f"### Project Structure\n```\n{tree}\n```\n---\n")
            elif fmt == 'xml':
                output.append(f"<tree>\n{html.escape(tree)}\n</tree>")
            else:
                output.append("PROJECT STRUCTURE:\n" + tree + "\n" + "=" * 50 + "\n")

        # 3. Dependency Graph
        if dependency_map:
            graph_text = self._format_dependency_graph(dependency_map)
            if graph_text:
                if fmt == 'markdown':
                    # ИСПРАВЛЕНО: Восстановлена строка вывода графа для Markdown
                    output.append(f"### Dependency Graph\n```\n{graph_text}\n```\n---\n")
                elif fmt == 'xml':
                    output.append(f"<dependencies>\n{html.escape(graph_text)}\n</dependencies>")
                else:
                    output.append("DEPENDENCY GRAPH:\n" + graph_text + "\n" + "=" * 50 + "\n")

        # 4. File Contents
        if fmt == 'xml':
            output.append(self._to_xml(files, dependency_map))
        elif fmt == 'markdown':
            output.append(self._to_markdown(files, dependency_map))
        else:
            output.append(self._to_plain(files, dependency_map))

        return "\n".join(output)

    @staticmethod
    def _format_dependency_graph(dep_map: Dict[str, Set[str]]) -> str:
        """Генерация текстового представления графа из словаря"""
        if not dep_map:
            return ""

        lines = []
        # Сортируем файлы для стабильного вывода
        sorted_files = sorted(dep_map.keys())

        for file_path in sorted_files:
            imports = dep_map[file_path]
            if not imports:
                continue

            display_name = os.path.basename(file_path)
            lines.append(f"{display_name}")
            for imp in sorted(list(imports)):
                lines.append(f"  -> {imp}")
            lines.append("")  # Пустая строка для читаемости

        return "\n".join(lines)

    @staticmethod
    def _generate_tree(paths: List[str]) -> str:
        """Генерация ASCII дерева"""
        if not paths: return ""
        try:
            common_path = os.path.commonpath(paths)
            if os.path.isfile(common_path):
                common_path = os.path.dirname(common_path)
            base_dir = Path(common_path).parent
        except (ValueError, OSError):
            base_dir = None

        tree_structure = {}
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

        def _build(structure, prefix=""):
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
            out.append(f"## File: {f.path}\n")

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