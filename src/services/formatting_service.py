import html
from pathlib import Path
from typing import List
from ..store.state import ProcessedFile

class FormattingService:
    """Сервис форматирования итогового текста"""
    
    def format_output(self, files: List[ProcessedFile], fmt: str, include_tree: bool, system_prompt: str) -> str:
        output = []
        
        # 1. System Prompt
        if system_prompt:
            if fmt == 'xml':
                output.append(f"<system_prompt>\n{html.escape(system_prompt)}\n</system_prompt>\n")
            elif fmt == 'markdown':
                output.append(f"> **System Context:**\n> {system_prompt}\n\n---\n")
            else:
                output.append(f"SYSTEM PROMPT:\n{system_prompt}\n" + "="*50 + "\n")

        # 2. Project Tree
        if include_tree and files:
            paths = [f.path for f in files]
            tree = self._generate_tree(paths)
            if fmt == 'markdown':
                output.append(f"```\n{tree}\n```\n")
            elif fmt == 'xml':
                output.append(f"<tree>\n{html.escape(tree)}\n</tree>")
            else:
                output.append("PROJECT STRUCTURE:\n" + tree + "\n" + "="*50 + "\n")

        # 3. File Contents
        if fmt == 'xml':
            output.append(self._to_xml(files))
        elif fmt == 'markdown':
            output.append(self._to_markdown(files))
        else:
            output.append(self._to_plain(files))
            
        return "\n".join(output)

    def _generate_tree(self, paths: List[str]) -> str:
        if not paths: return ""
        tree_structure = {}
        for path in sorted(paths):
            parts = Path(path).parts
            current = tree_structure
            for part in parts:
                current = current.setdefault(part, {})
        
        lines = ["Project Structure:"]
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
        
        _build(tree_structure)
        return "\n".join(lines)

    def _to_xml(self, files: List[ProcessedFile]) -> str:
        out = ["<root>"]
        for f in files:
            out.append(f'  <file path="{f.path}">')
            out.append(f'    {html.escape(f.content)}')
            out.append(f'  </file>')
        out.append("</root>")
        return "\n".join(out)

    def _to_markdown(self, files: List[ProcessedFile]) -> str:
        out = []
        for f in files:
            ext = Path(f.path).suffix.lstrip('.') or 'txt'
            out.append(f"### File: {f.path}")
            out.append(f"```{ext}")
            out.append(f.content)
            out.append("```\n")
        return "\n".join(out)

    def _to_plain(self, files: List[ProcessedFile]) -> str:
        out = []
        sep = "=" * 50
        for f in files:
            out.append(sep)
            out.append(f"FILE: {f.path}")
            out.append(sep)
            out.append(f.content)
            out.append("\n")
        return "\n".join(out)