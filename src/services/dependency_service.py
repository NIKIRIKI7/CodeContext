import ast
import re
import asyncio
from typing import List, Dict, Set, Tuple

def _parse_python(code: str) -> Set[str]:
    imports = set()
    try:
        tree = ast.parse(code)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names: imports.add(alias.name)
            elif isinstance(node, ast.ImportFrom):
                module = node.module if node.module else ""
                prefix = "." * node.level
                if prefix or module: imports.add(f"{prefix}{module}")
    except (SyntaxError, ValueError):
        pass
    return imports

def _parse_web(code: str) -> Set[str]:
    imports = set()
    for match in re.finditer(r'import\s+.*?\s+from\s+[\'"](.*?)[\'"]', code, re.MULTILINE):
        imports.add(match.group(1))
    for match in re.finditer(r'(?:require|import)\s*\(\s*[\'"](.*?)[\'"]\s*\)', code, re.MULTILINE):
        imports.add(match.group(1))
    return imports

_PARSERS = {
    '.py': _parse_python,
    '.js': _parse_web, '.jsx': _parse_web, '.ts': _parse_web,
    '.tsx': _parse_web, '.vue': _parse_web, '.svelte': _parse_web, '.mjs': _parse_web
}

async def resolve_dependencies(files: List[Dict[str, str]]) -> Dict[str, Set[str]]:
    if not files: return {}

    def _analyze_sync(file: Dict[str, str]) -> Tuple[str, Set[str]]:
        full_path = file['path']
        content = file['content']
        if not content: return full_path, set()

        ext = file.get('ext', '')
        if not ext and '.' in full_path:
            ext = "." + full_path.split('.')[-1].lower()

        parser = _PARSERS.get(ext.lower())
        return full_path, parser(content) if parser else set()

    tasks = [asyncio.to_thread(_analyze_sync, f) for f in files]
    results = await asyncio.gather(*tasks)

    return {path: imports for path, imports in results if imports}
