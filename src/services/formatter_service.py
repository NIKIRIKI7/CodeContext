"""Formatter service."""
import html
from pathlib import Path
from typing import List, Dict, Any
from .tree_service import TreeService

class FormatterService:
    @staticmethod
    def format(files_data: List[Dict[str, Any]], fmt: str, include_tree: bool = True, system_prompt: str = "") -> str:
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
        if include_tree:
            paths = [f['path'] for f in files_data]
            tree = TreeService.generate_tree(paths)
            if fmt == 'markdown':
                output.append("# Project Structure\n```text\n" + tree + "\n```\n---\n")
            elif fmt == 'xml':
                output.append(f"<tree>\n{html.escape(tree)}\n</tree>")
            else:
                output.append("PROJECT STRUCTURE:\n" + tree + "\n" + "="*50 + "\n")

        # 3. Content
        if fmt == 'xml':
            output.append(FormatterService.to_xml(files_data))
        elif fmt == 'markdown':
            output.append(FormatterService.to_markdown(files_data))
        else:
            output.append(FormatterService.to_plain(files_data))
            
        return "\n".join(output)

    @staticmethod
    def to_xml(files_data: List[Dict[str, Any]]) -> str:
        output = ["<root>"]
        for item in files_data:
            output.append(f'  <file path="{item["path"]}">')
            output.append(f'    {html.escape(item["content"])}')
            output.append(f'  </file>')
        output.append("</root>")
        return "\n".join(output)

    @staticmethod
    def to_markdown(files_data: List[Dict[str, Any]]) -> str:
        output = []
        for item in files_data:
            path = item['path']
            ext = Path(path).suffix.lstrip('.') or 'txt'
            output.append(f"### File: `{path}`")
            output.append(f"```{ext}")
            output.append(item['content'])
            output.append("```\n")
        return "\n".join(output)

    @staticmethod
    def to_plain(files_data: List[Dict[str, Any]]) -> str:
        output = []
        separator = "=" * 50
        for item in files_data:
            output.append(separator)
            output.append(f"FILE: {item['path']}")
            output.append(separator)
            output.append(item['content'])
            output.append("\n")
        return "\n".join(output)