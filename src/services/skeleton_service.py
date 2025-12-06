import ast
import re
import sys

class SkeletonService:
    """
    Сервис для создания 'скелета' кода.
    Оставляет только определения классов, функций и импорты.
    """

    def make_skeleton(self, code: str, ext: str) -> str:
        """
        Главный метод. Выбирает стратегию в зависимости от расширения.
        """
        if not code or not code.strip():
            return ""

        ext = ext.lower()

        # Python Strategy (AST)
        if ext == '.py':
            return self._python_skeleton(code)

        # Web Strategy (JS, TS, Vue, Svelte, Angular, etc.)
        if ext in ['.js', '.jsx', '.ts', '.tsx', '.vue', '.svelte', '.html', '.css', '.scss']:
            return self._web_skeleton(code)

        # Fallback: Return original
        return code

    @staticmethod
    def _python_skeleton(code: str) -> str:
        """
        Использует AST для Python.
        Оставляет сигнатуры, докстринги, удаляет тела методов.
        """
        try:
            tree = ast.parse(code)
        except SyntaxError as e:
            # Если парсинг не удался (например, из-за неправильной минификации)
            return f"# [Skeleton Error] Could not parse Python code: {e}\n# Check indentation or disable Minify.\n\n{code}"

        transformer = PythonSkeletonTransformer()
        new_tree = transformer.visit(tree)
        ast.fix_missing_locations(new_tree)

        if sys.version_info >= (3, 9):
            return ast.unparse(new_tree)
        else:
            return code

    @staticmethod
    def _web_skeleton(code: str) -> str:
        """
        Regex-based подход для JS/TS и фреймворков.
        Выделяет импорты, экспорты, классы и функции.
        """
        lines = code.splitlines()
        skeleton_lines = []

        keep_patterns = [
            r'^\s*(import|export|from|require)\s+',
            r'^\s*(abstract\s+)?class\s+\w+',
            r'^\s*interface\s+\w+',
            r'^\s*type\s+\w+\s*=',
            r'^\s*enum\s+\w+',
            r'^\s*@\w+',
            r'^\s*(async\s+)?function\s*\w*',
            # Упрощенный поиск методов и стрелочных функций
            r'^\s*(public|private|protected|static|readonly|async|override)*\s*\w+\s*\(.*\).*{?',
            r'^\s*(const|let|var)\s+\w+\s*[:=]\s*(\(.*\)|.*)\s*=>',
            r'^\s*<script',
            r'^\s*</script',
            r'^\s*<template',
            r'^\s*</template',
            r'^\s*<style',
            r'^\s*</style',
        ]

        compiled_patterns = [re.compile(p) for p in keep_patterns]

        for line in lines:
            line_stripped = line.strip()
            if not line_stripped:
                continue

            for pattern in compiled_patterns:
                if pattern.match(line):
                    # Попытка аккуратно закрыть однострочные блоки
                    if line_stripped.endswith('{'):
                        skeleton_lines.append(line + " // ... }")
                    elif '=>' in line_stripped and '{' in line_stripped:
                        skeleton_lines.append(line.split('{')[0] + "{ // ... }")
                    else:
                        skeleton_lines.append(line)
                    break

        if not skeleton_lines:
            return "// (Empty or Logic-only file)"

        return "\n".join(skeleton_lines)


class PythonSkeletonTransformer(ast.NodeTransformer):
    """AST Transformer для Python: заменяет тела функций на '...'"""

    def visit_FunctionDef(self, node):
        # Сохраняем Docstring
        docstring = ast.get_docstring(node)

        new_body = []
        if docstring:
            new_body.append(ast.Expr(value=ast.Constant(value=docstring)))

        # Добавляем Ellipsis (...)
        new_body.append(ast.Expr(value=ast.Constant(value=Ellipsis)))

        node.body = new_body
        return node

    def visit_AsyncFunctionDef(self, node):
        return self.visit_FunctionDef(node)

    def visit_ClassDef(self, node):
        # Рекурсивно обрабатываем методы класса
        self.generic_visit(node)
        return node