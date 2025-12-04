"""Formatter service for different output structures."""
import html
from pathlib import Path
from typing import List, Dict, Any


class FormatterService:
    """Форматирование вывода в разные структуры."""
    
    @staticmethod
    def to_xml(files_data: List[Dict[str, Any]]) -> str:
        """
        Строгий XML формат.
        
        Args:
            files_data: List of file data dictionaries
            
        Returns:
            XML formatted string
        """
        output = ["<root>"]
        for item in files_data:
            path = item['path']
            content = html.escape(item['content'])  # Экранирование спецсимволов
            output.append(f'  <file path="{path}">')
            output.append(f'    {content}')
            output.append(f'  </file>')
        output.append("</root>")
        return "\n".join(output)

    @staticmethod
    def to_markdown(files_data: List[Dict[str, Any]]) -> str:
        """
        Markdown с code blocks.
        
        Args:
            files_data: List of file data dictionaries
            
        Returns:
            Markdown formatted string
        """
        output = []
        for item in files_data:
            path = item['path']
            ext = Path(path).suffix.lstrip('.')
            content = item['content']
            output.append(f"### File: `{path}`")
            output.append(f"```{ext}")
            output.append(content)
            output.append("```\n")
        return "\n".join(output)

    @staticmethod
    def to_plain(files_data: List[Dict[str, Any]]) -> str:
        """
        Простой текстовый формат с разделителями.
        
        Args:
            files_data: List of file data dictionaries
            
        Returns:
            Plain text formatted string
        """
        output = []
        separator = "=" * 50
        for item in files_data:
            output.append(separator)
            output.append(f"FILE: {item['path']}")
            output.append(separator)
            output.append(item['content'])
            output.append("\n")
        return "\n".join(output)