import ast
import re
from typing import List


class SkeletonService:
    """
    Сервис для создания 'скелета' кода.
    Оставляет только определения классов, функций и импорты.
    """

    def make_skeleton(self, code: str, ext: str) -> str:
        if not code or not code.strip():
            return ""
        ext = ext.lower()
        if ext == '.py':
            return self._python_skeleton(code)
        if ext in ['.js', '.jsx', '.ts', '.tsx', '.vue', '.svelte', '.html', '.css', '.scss']:
            return self._web_skeleton(code)
        return code

    @staticmethod
    def _python_skeleton(code: str) -> str:
        try:
            tree = ast.parse(code)
        except SyntaxError as e:
            return f"# Syntax Error: {e}\n{code}"

        transformer = PythonSkeletonTransformer()
        new_tree = transformer.visit(tree)
        ast.fix_missing_locations(new_tree)

        if hasattr(ast, "unparse"):
            return ast.unparse(new_tree)  # type: ignore
        return code

    @staticmethod
    def _web_skeleton(code: str) -> str:
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

    def visit_FunctionDef(self, node: ast.FunctionDef) -> ast.FunctionDef:
        docstring = ast.get_docstring(node)
        new_body: List[ast.stmt] = []
        if docstring:
            new_body.append(ast.Expr(value=ast.Constant(value=docstring)))
        new_body.append(ast.Expr(value=ast.Constant(value=Ellipsis)))
        node.body = new_body
        return node

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> ast.AsyncFunctionDef:
        docstring = ast.get_docstring(node)
        new_body: List[ast.stmt] = []
        if docstring:
            new_body.append(ast.Expr(value=ast.Constant(value=docstring)))
        new_body.append(ast.Expr(value=ast.Constant(value=Ellipsis)))
        node.body = new_body
        return node

    def visit_ClassDef(self, node: ast.ClassDef) -> ast.ClassDef:
        self.generic_visit(node)
        return node