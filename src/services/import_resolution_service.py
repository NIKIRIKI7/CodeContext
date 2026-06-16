import os
from typing import List

def _is_match(p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
    if p_no_ext.endswith(suffix) or p_no_ext == core_imp: return True
    if p_no_ext.endswith(f"{suffix}/index") or p_no_ext.endswith(f"{suffix}/ui/index"): return True
    if parts and p_no_ext.endswith(f"{suffix}/{parts[-1]}"): return True
    if p_no_ext.endswith(f"{suffix}/__init__") or p_no_ext.endswith(f"{suffix}/main"): return True

    if parts:
        p_norm = p_no_ext.replace("\\", "/")
        for mono_dir in ("/packages/", "/libs/", "/apps/", "/modules/"):
            idx = p_norm.find(mono_dir)
            if idx != -1:
                after_dir = p_norm[idx + len(mono_dir):]
                path_segments = after_dir.split("/")
                if path_segments and path_segments[0] in parts:
                    pkg_idx = parts.index(path_segments[0])
                    path_after_pkg = "/".join(path_segments[1:])
                    parts_after_pkg = parts[pkg_idx + 1:]
                    if not parts_after_pkg:
                        if path_after_pkg in ("src/index", "lib/index", "index"): return True
                    else:
                        suffix_from_parts = "/".join(parts_after_pkg)
                        if path_after_pkg in (suffix_from_parts, f"src/{suffix_from_parts}", f"lib/{suffix_from_parts}"):
                            return True
    return False

def resolve(import_str: str, available_paths: List[str]) -> List[str]:
    clean_imp = import_str
    if '.' in import_str and '/' not in import_str and not import_str.startswith('.'):
        clean_imp = import_str.replace('.', '/')
    for prefix in ('@/', '~/', '@', '#'):
        if clean_imp.startswith(prefix):
            clean_imp = clean_imp[len(prefix):]
            break

    parts = [p for p in clean_imp.replace('\\', '/').split('/') if p not in ('.', '..', '')]
    if not parts: return []

    core_imp = "/".join(parts)
    suffix = "/" + core_imp

    matched = []
    for p in available_paths:
        p_no_ext, _ = os.path.splitext(p.replace('\\', '/'))
        if _is_match(p_no_ext, core_imp, suffix, parts):
            matched.append(p)

    return list(set(matched))
