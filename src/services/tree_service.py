"""Service to generate ASCII project tree structure."""
from pathlib import Path
from typing import List

class TreeService:
    @staticmethod
    def generate_tree(paths: List[str]) -> str:
        """
        Generates a visual ASCII tree from a list of file paths.
        """
        if not paths:
            return ""

        # Build nested dict structure
        tree_structure = {}
        for path in sorted(paths):
            parts = Path(path).parts
            current = tree_structure
            for part in parts:
                current = current.setdefault(part, {})

        lines = ["Project Structure:"]
        
        def _build_lines(structure, prefix=""):
            items = list(structure.keys())
            # Sort: directories first (those having children), then files
            # But here 'structure' keys are mix. Heuristic: if value is empty dict, likely file (in this context)
            # Actually, simplify: just sort alphabetically
            items.sort()
            
            for i, name in enumerate(items):
                is_last = (i == len(items) - 1)
                connector = "└── " if is_last else "├── "
                lines.append(f"{prefix}{connector}{name}")
                
                children = structure[name]
                if children:
                    extension = "    " if is_last else "│   "
                    _build_lines(children, prefix + extension)

        # Handle multiple roots (drive letters) or common prefix logic could be added here
        # For now, we assume paths are somewhat relative or we show full roots
        _build_lines(tree_structure)
        return "\n".join(lines)