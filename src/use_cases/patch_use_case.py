import json
import re
from src.i18n import tr
from ..actions.dispatcher import Dispatcher
from ..actions.action_types import UI_ADD_LOG
from ..services.patch_service import PatchService


class PatchUseCase:
    """Оркестрирует умный парсинг JSON и применение множества патчей."""

    def __init__(self, dispatcher: Dispatcher, patch_service: PatchService):
        self._dispatcher = dispatcher
        self._patch_service = patch_service

    def prepare_json_patch(self, patch_str: str, base_folders: list) -> list:
        if not patch_str or not patch_str.strip():
            self._dispatcher.dispatch(UI_ADD_LOG, tr("patch_use_case.patch.empty"))
            return []

        self._dispatcher.dispatch(UI_ADD_LOG, tr("patch_use_case.parsing.parsing_llm"))

        # 1. Попытка найти JSON-блоки (маркдаун)
        blocks = re.findall(r'```(?:json)?\s*(.*?)\s*```', patch_str, re.DOTALL)

        # 2. Если не нашли, ищем просто массив [...]
        if not blocks:
            match = re.search(r'\[\s*\{.*?\}\s*\]', patch_str, re.DOTALL)
            if match:
                blocks = [match.group(0)]

        # 3. Если всё еще не нашли, ищем один объект {...} (если нейросеть забыла массив)
        if not blocks:
            match = re.search(r'\{\s*"action".*?\}', patch_str, re.DOTALL)
            if match:
                blocks = ["[" + match.group(0) + "]"]

        # 4. Фолбэк на весь текст
        if not blocks:
            blocks = [patch_str.strip()]

        all_patches = []
        for block in blocks:
            block = block.strip()
            if not block: continue
            try:
                data = json.loads(block)
                if isinstance(data, list):
                    all_patches.extend(data)
                elif isinstance(data, dict):
                    all_patches.append(data)
            except json.JSONDecodeError as e:
                self._dispatcher.dispatch(UI_ADD_LOG, tr("patch_use_case.parsing.parse_error", error=e))
                continue

        if not all_patches:
            self._dispatcher.dispatch(UI_ADD_LOG, tr("patch_use_case.parsing.no_valid_json"))
            return []

        self._dispatcher.dispatch(UI_ADD_LOG, tr("patch_use_case.patch.preparing", count=len(all_patches)))
        return self._patch_service.prepare_patches(all_patches, base_folders)

    def apply_prepared(self, prepared_patches: list):
        if not prepared_patches:
            return 0, []

        applied_count, logs = self._patch_service.apply_prepared(prepared_patches)
        for log in logs:
            self._dispatcher.dispatch(UI_ADD_LOG, log)

        self._dispatcher.dispatch(UI_ADD_LOG, tr("patch_use_case.patch.applied", count=applied_count))

        return applied_count, logs