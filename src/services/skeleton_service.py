import ast
import re

def make_skeleton(code: str, ext: str) -> str:
    if not code or not code.strip(): return ""
    ext = ext.lower()
    if ext == '.py': return _python_skeleton(code)
    if ext in ['.js', '.jsx', '.ts', '.tsx', '.vue', '.svelte', '.html', '.css', '.scss']:
        return _web_skeleton(code)
    return code

def _python_skeleton(code: str) -> str:
    try: tree = ast.parse(code)
    except SyntaxError: return f"# Skeleton Error: Invalid Syntax\n{code[:500]}..."

    transformer = PythonSkeletonTransformer()
    new_tree = transformer.visit(tree)
    ast.fix_missing_locations(new_tree)
    if hasattr(ast, "unparse"):
        return ast.unparse(new_tree)
    return code

def _web_skeleton(code: str) -> str:
    lines = code.splitlines()
    skeleton_lines = []
    keep_patterns = [
        re.compile(p) for p in [
            r'^\s*(import|export|from|require)\s+',
            r'^\s*(abstract\s+)?class\s+\w+',
            r'^\s*interface\s+\w+',
            r'^\s*type\s+\w+\s*=',
            r'^\s*enum\s+\w+',
            r'^\s*@\w+',
            r'^\s*(async\s+)?function\s*\w*',
            r'^\s*(public|private|protected|static|readonly|async|override)*\s*\w+\s*\(.*\).*{?',
            r'^\s*(const|let|var)\s+\w+\s*[:=]\s*(\(.*\)|.*)\s*=>',
            r'^\s*<script', r'^\s*</script', r'^\s*<template', r'^\s*</template',
            r'^\s*<style', r'^\s*</style',
        ]
    ]
    for line in lines:
        line_stripped = line.strip()
        if not line_stripped: continue
        for pattern in keep_patterns:
            if pattern.match(line):
                if line_stripped.endswith('{'): skeleton_lines.append(line + " // ... }")
                elif '=>' in line_stripped and '{' in line_stripped: skeleton_lines.append(line.split('{')[0] + "{ // ... }")
                else: skeleton_lines.append(line)
                break
    if not skeleton_lines: return "// (Empty or Logic-only file)"
    return "\n".join(skeleton_lines)

class PythonSkeletonTransformer(ast.NodeTransformer):
    def visit_FunctionDef(self, node: ast.FunctionDef) -> ast.FunctionDef:
        docstring = ast.get_docstring(node)
        new_body = []
        if docstring: new_body.append(ast.Expr(value=ast.Constant(value=docstring)))
        new_body.append(ast.Expr(value=ast.Constant(value=Ellipsis)))
        node.body = new_body
        return node

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> ast.AsyncFunctionDef:
        docstring = ast.get_docstring(node)
        new_body = []
        if docstring: new_body.append(ast.Expr(value=ast.Constant(value=docstring)))
        new_body.append(ast.Expr(value=ast.Constant(value=Ellipsis)))
        node.body = new_body
        return node

    def visit_ClassDef(self, node: ast.ClassDef) -> ast.ClassDef:
        self.generic_visit(node)
        return node
