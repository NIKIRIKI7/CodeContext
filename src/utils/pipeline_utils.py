from typing import List, Dict, Any, Tuple, Set
from types import SimpleNamespace
from ..store.state import ProcessedFile


class PipelineUtils:
    """
    Утилиты для обработки файлов, которые можно переиспользовать
    в CLI и GUI контроллерах.
    """

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
        """
        Асинхронный обход графа зависимостей (BFS).
        Используется в CLI и GUI контроллерах для избежания дублирования кода.
        """
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