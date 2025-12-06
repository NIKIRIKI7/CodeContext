from typing import List, Dict, Any
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
        """
        Синхронная обработка пакета файлов:
        1. Очистка (Cleaner)
        2. Создание скелета (Skeleton, если включено)
        3. Подсчет токенов (Token)
        """
        processed_files = []

        for raw in raw_files:
            path = raw['path']
            content = raw['content']
            ext = raw['ext']

            # 1. Очистка
            cleaned = cleaner_service.clean(content, ext, options)

            # 2. Скелет (если нужен)
            if options.skeleton_mode:
                cleaned = skeleton_service.make_skeleton(cleaned, ext)

            # 3. Токены
            tokens = token_service.count_tokens(cleaned)

            processed_files.append(ProcessedFile(
                path=path,
                content=cleaned,
                tokens=tokens
            ))

        return processed_files