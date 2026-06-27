import concurrent.futures
import os
from collections import deque
from types import SimpleNamespace
from typing import List, Dict, Any, Tuple, Set

from ..store.state import ProcessedFile

_ENTRY_PRIORITY_NAMES = {
    'main.py', 'app.py', 'index.py', 'cli.py',
    'index.js', 'index.ts', 'app.js', 'app.ts', 'main.js', 'main.ts',
    'index.jsx', 'index.tsx', 'App.jsx', 'App.tsx',
    'main.go', 'main.java', 'main.cpp',
}

def _priority_sort_key(item: Dict[str, str]) -> tuple:
    path = item.get('path', '')
    basename = os.path.basename(path)
    ext = os.path.splitext(path)[1].lower()
    is_entry = basename in _ENTRY_PRIORITY_NAMES
    is_config = ext in ('.json', '.yaml', '.yml', '.toml', '.ini', '.env', '.cfg', '.conf', '.xml')

    if is_entry: return 0, path.lower()
    elif is_config: return 1, path.lower()
    return 2, path.lower()

def _process_single_worker(raw: Dict[str, str], opts_dict: dict) -> ProcessedFile:
    from ..services import cleaner_service, skeleton_service, token_service
    opts = SimpleNamespace(**opts_dict)

    cleaned = cleaner_service.clean(raw['content'], raw['ext'], opts)
    if getattr(opts, 'skeleton_mode', False):
        cleaned = skeleton_service.make_skeleton(cleaned, raw['ext'])

    tokens = token_service.count_tokens(cleaned)
    return ProcessedFile(path=raw['path'], content=cleaned, tokens=tokens)

def process_files_batch_parallel(raw_files: List[Dict[str, str]], options: Any) -> List[ProcessedFile]:
    if not raw_files: return []

    opts_dict = options.__dict__ if hasattr(options, '__dict__') else options
    if getattr(options, 'prioritize_entry_files', True):
        raw_files = sorted(raw_files, key=_priority_sort_key)

    result = []

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = [executor.submit(_process_single_worker, raw, opts_dict) for raw in raw_files]
        for future in concurrent.futures.as_completed(futures):
            try:
                result.append(future.result())
            except Exception as e:
                print(f"Error processing file: {e}")

    return result


# ponytail: queue.pop(0) was O(n). Swapped to deque.popleft for O(1).
async def resolve_and_collect_dependencies_async(initial_queue: List[Tuple[str, int]], visited_paths: Set[str], all_paths: List[str], is_deep: bool, fs_repo: Any) -> None:
    from ..services import dependency_service, import_resolution_service
    queue = deque(initial_queue)
    while queue:
        curr_path, depth = queue.popleft()
        if curr_path in visited_paths: continue
        if not is_deep and depth > 1: continue

        visited_paths.add(curr_path)
        content = await fs_repo.read_file_async(curr_path)
        if not content: continue

        dep_map = await dependency_service.resolve_dependencies([{"path": curr_path, "content": content}])
        imports = dep_map.get(curr_path, set())

        for imp in imports:
            for p in import_resolution_service.resolve(imp, all_paths):
                if p not in visited_paths:
                    queue.append((p, depth + 1))
