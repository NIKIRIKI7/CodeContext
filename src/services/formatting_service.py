import html
import os
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
                output.append(f"SYSTEM PROMPT:\n{system_prompt}\n" + "=" * 50 + "\n")

        # 2. Project Tree
        if include_tree and files:
            paths = [f.path for f in files]
            tree = self._generate_tree(paths)
            if fmt == 'markdown':
                output.append(f"```\n{tree}\n```\n")
            elif fmt == 'xml':
                output.append(f"<tree>\n{html.escape(tree)}\n</tree>")
            else:
                output.append("PROJECT STRUCTURE:\n" + tree + "\n" + "=" * 50 + "\n")

        # 3. File Contents
        if fmt == 'xml':
            output.append(self._to_xml(files))
        elif fmt == 'markdown':
            output.append(self._to_markdown(files))
        else:
            output.append(self._to_plain(files))

        return "\n".join(output)

    def _generate_tree(self, paths: List[str]) -> str:
        """Генерация ASCII дерева с относительными путями"""
        if not paths: return ""

        # 1. Определяем базовую директорию, чтобы отрезать лишний путь (C:\Users...)
        try:
            # Находим общий путь для всех файлов (например: C:\Projects\MyBot)
            common_path = os.path.commonpath(paths)

            # Если common_path указывает на файл (если выбран один файл), берем его папку
            if os.path.isfile(common_path):
                common_path = os.path.dirname(common_path)

            # Берем РОДИТЕЛЯ общего пути, чтобы в дереве осталась корневая папка проекта
            # Например, если общий путь C:\Projects\MyBot, то base_dir будет C:\Projects
            # Тогда в дереве первым уровнем будет MyBot
            base_dir = Path(common_path).parent
        except (ValueError, OSError):
            # Если файлы на разных дисках или другая ошибка, используем абсолютные пути
            base_dir = None

        tree_structure = {}
        for path in sorted(paths):
            p = Path(path)

            # Пытаемся получить относительный путь
            if base_dir:
                try:
                    rel_parts = p.relative_to(base_dir).parts
                except ValueError:
                    rel_parts = p.parts  # Фолбэк на абсолютный
            else:
                rel_parts = p.parts

            # Строим вложенный словарь
            current = tree_structure
            for part in rel_parts:
                current = current.setdefault(part, {})

        # Рекурсивная функция для отрисовки линий
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

        # Если base_dir не определен (разные диски), добавляем заголовок
        if not base_dir:
            lines.append("Project Structure (Absolute Paths):")
        else:
            # Если структура начинается с одного корня, можно добавить заголовок, если нужно
            # Но обычно само дерево уже содержит корень
            pass

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