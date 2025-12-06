import ast
import re
from abc import ABC, abstractmethod
from typing import List, Set

class DependencyParserStrategy(ABC):
    """Интерфейс стратегии парсинга зависимостей"""

    @property
    @abstractmethod
    def supported_extensions(self) -> List[str]:
        """Возвращает список расширений, которые поддерживает эта стратегия"""
        pass

    @abstractmethod
    def parse(self, code: str) -> Set[str]:
        """Извлекает зависимости из кода"""
        pass


class PythonDependencyParser(DependencyParserStrategy):
    """Стратегия для Python (AST)"""

    @property
    def supported_extensions(self) -> List[str]:
        return ['.py']

    def parse(self, code: str) -> Set[str]:
        imports = set()
        try:
            tree = ast.parse(code)
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        imports.add(alias.name)
                elif isinstance(node, ast.ImportFrom):
                    module = node.module if node.module else ""
                    prefix = "." * node.level
                    if prefix or module:
                        imports.add(f"{prefix}{module}")
        except Exception:
            pass
        return imports


class WebDependencyParser(DependencyParserStrategy):
    """Стратегия для JS/TS/Vue/React (Regex)"""

    @property
    def supported_extensions(self) -> List[str]:
        return ['.js', '.jsx', '.ts', '.tsx', '.vue', '.svelte', '.mjs']

    def parse(self, code: str) -> Set[str]:
        imports = set()
        import_pattern = re.compile(r'import\s+.*?\s+from\s+[\'"](.*?)[\'"]', re.MULTILINE)
        require_pattern = re.compile(r'(?:require|import)\s*\(\s*[\'"](.*?)[\'"]\s*\)', re.MULTILINE)

        for match in import_pattern.finditer(code):
            imports.add(match.group(1))

        for match in require_pattern.finditer(code):
            imports.add(match.group(1))

        return imports