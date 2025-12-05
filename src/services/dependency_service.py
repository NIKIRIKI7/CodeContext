import ast
import re
import asyncio
from typing import List, Dict, Set, Tuple


class DependencyService:
    """
    Сервис для анализа зависимостей.
    Работает с сырым контентом до минификации, чтобы корректно парсить AST/Regex.
    """

    async def resolve_dependencies(self, files: List[Dict[str, str]]) -> Dict[str, Set[str]]:
        """
        Асинхронно анализирует список файлов.
        files: Список словарей {'path': str, 'content': str, 'ext': str}
        Возвращает: { "полный_путь_к_файлу": {"import1", "import2"} }
        """
        if not files:
            return {}

        tasks = [self._process_single_file(file) for file in files]
        results: List[Tuple[str, Set[str]]] = await asyncio.gather(*tasks)

        dependency_map: Dict[str, Set[str]] = {}
        for path, imports in results:
            if imports:
                dependency_map[path] = imports

        return dependency_map

    async def _process_single_file(self, file: Dict[str, str]) -> Tuple[str, Set[str]]:
        return await asyncio.to_thread(self._analyze_sync, file)

    def _analyze_sync(self, file: Dict[str, str]) -> Tuple[str, Set[str]]:
        full_path = file['path']
        content = file['content']
        # Определяем расширение, если его нет явно, берем из пути
        ext = file.get('ext')
        if not ext:
            ext = "." + full_path.split('.')[-1].lower() if '.' in full_path else ""

        imports = set()

        # Если контента нет, возвращаем пустоту
        if not content:
            return full_path, imports

        if ext == '.py':
            imports = self._analyze_python(content)
        elif ext in ['.js', '.jsx', '.ts', '.tsx', '.vue']:
            imports = self._analyze_js_ts(content)

        return full_path, imports

    def _analyze_python(self, code: str) -> Set[str]:
        imports = set()
        try:
            tree = ast.parse(code)
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        imports.add(alias.name)
                elif isinstance(node, ast.ImportFrom):
                    module = node.module if node.module else ""
                    # Обработка относительных импортов (from . import x)
                    prefix = "." * node.level
                    if prefix or module:
                        imports.add(f"{prefix}{module}")
        except Exception:
            # При минификации AST может падать, но мы теперь подаем сырой код,
            # так что ошибки будут только если сам код изначально битый.
            pass
        return imports

    def _analyze_js_ts(self, code: str) -> Set[str]:
        imports = set()
        # Regex для import ... from ...
        import_pattern = re.compile(r'import\s+.*?\s+from\s+[\'"](.*?)[\'"]', re.MULTILINE)
        # Regex для require(...) или import(...)
        require_pattern = re.compile(r'(?:require|import)\s*\(\s*[\'"](.*?)[\'"]\s*\)', re.MULTILINE)

        for match in import_pattern.finditer(code):
            imports.add(match.group(1))

        for match in require_pattern.finditer(code):
            imports.add(match.group(1))

        return imports