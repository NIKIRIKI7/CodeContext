import concurrent.futures
import math
import os
import tempfile
from typing import List, Dict, Any, Tuple, Set
from types import SimpleNamespace
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

    if is_entry:
        priority = 0
    elif is_config:
        priority = 1
    else:
        priority = 2

    return (priority, path.lower())


def _process_chunk_worker(chunk: List[Dict[str, str]], opts_dict: dict) -> List[dict]:
    from ..services.cleaner_service import CleanerService
    from ..services.skeleton_service import SkeletonService
    from ..services.token_service import TokenService
    cleaner = CleanerService()
    skeleton = SkeletonService()
    tokenizer = TokenService()
    opts = SimpleNamespace(**opts_dict)
    results = []
    for raw in chunk:
        cleaned = cleaner.clean(raw['content'], raw['ext'], opts)
        if opts.skeleton_mode:
            cleaned = skeleton.make_skeleton(cleaned, raw['ext'])
        tokens = tokenizer.count_tokens(cleaned)

        fd, temp_path = tempfile.mkstemp(prefix="cc_ipc_", suffix=".txt")
        with os.fdopen(fd, 'w', encoding='utf-8') as f:
            f.write(cleaned)

        results.append({
            "path": raw['path'],
            "temp_file": temp_path,
            "tokens": tokens
        })
    return results


class PipelineUtils:

    @staticmethod
    def process_files_batch_parallel(
            raw_files: List[Dict[str, str]],
            options: Any
    ) -> List[ProcessedFile]:
        if not raw_files:
            return []
        opts_dict = options.__dict__ if hasattr(options, '__dict__') else options

        if getattr(options, 'prioritize_entry_files', True):
            raw_files = sorted(raw_files, key=_priority_sort_key)

        workers = max(1, (os.cpu_count() or 2) - 1)
        chunk_size = math.ceil(len(raw_files) / workers)
        chunks = [raw_files[i:i + chunk_size] for i in range(0, len(raw_files), chunk_size)]
        processed_dicts = []
        with concurrent.futures.ProcessPoolExecutor(max_workers=workers) as executor:
            futures = [executor.submit(_process_chunk_worker, chunk, opts_dict) for chunk in chunks]
            for future in concurrent.futures.as_completed(futures):
                processed_dicts.extend(future.result())

        result = []
        for d in processed_dicts:
            content = ""
            try:
                with open(d["temp_file"], 'r', encoding='utf-8') as f:
                    content = f.read()
                os.remove(d["temp_file"])
            except Exception:
                pass
            result.append(ProcessedFile(path=d["path"], content=content, tokens=d["tokens"]))
        return result

    @staticmethod
    def process_files_batch(
            raw_files: List[Dict[str, str]],
            options: SimpleNamespace,
            cleaner_service: Any,
            skeleton_service: Any,
            token_service: Any
    ) -> List[ProcessedFile]:
        processed_files = []
        for raw in raw_files:
            path = raw['path']
            content = raw['content']
            ext = raw['ext']

            cleaned = cleaner_service.clean(content, ext, options)
            if options.skeleton_mode:
                cleaned = skeleton_service.make_skeleton(cleaned, ext)

            tokens = token_service.count_tokens(cleaned)
            processed_files.append(ProcessedFile(
                path=path,
                content=cleaned,
                tokens=tokens
            ))
        return processed_files

    @staticmethod
    async def resolve_and_collect_dependencies_async(
            initial_queue: List[Tuple[str, int]],
            visited_paths: Set[str],
            all_paths: List[str],
            is_deep: bool,
            process_service: Any,
            dependency_service: Any,
            import_resolution_service: Any
    ) -> None:
        queue = initial_queue
        while queue:
            curr_path, depth = queue.pop(0)
            if curr_path in visited_paths:
                continue

            if not is_deep and depth > 1:
                continue

            visited_paths.add(curr_path)

            raw_list = await process_service.read_files_async([curr_path])
            if not raw_list:
                continue

            dep_map = await dependency_service.resolve_dependencies(raw_list)
            imports = dep_map.get(curr_path, set())

            for imp in imports:
                matched_paths = import_resolution_service.resolve(imp, all_paths)
                for p in matched_paths:
                    if p not in visited_paths:
                        queue.append((p, depth + 1))
