import json
import re
from ..actions.dispatcher import Dispatcher
from ..actions.action_types import UI_ADD_LOG
from ..services.patch_service import PatchService


class PatchUseCase:
    """Оркестрирует умный парсинг JSON и применение множества патчей."""

    def __init__(self, dispatcher: Dispatcher, patch_service: PatchService):
        self._dispatcher = dispatcher
        self._patch_service = patch_service

    def apply_json_patch(self, patch_str: str, base_folders: list):
        if not patch_str or not patch_str.strip():
            self._dispatcher.dispatch(UI_ADD_LOG, "⚠️ Пустая строка патча.")
            return

        self._dispatcher.dispatch(UI_ADD_LOG, "🔍 Парсинг ответа LLM...")

        # Ищем все markdown-блоки с JSON (LLM часто разбивает ответ на части)
        blocks = re.findall(r'```(?:json)?\s*(.*?)\s*```', patch_str, re.DOTALL)

        # Если блоков нет, предполагаем, что весь текст — это и есть чистый JSON
        if not blocks:
            blocks = [patch_str.strip()]

        all_patches = []

        for block in blocks:
            block = block.strip()
            if not block:
                continue
            try:
                data = json.loads(block)
                if isinstance(data, list):
                    all_patches.extend(data)
                elif isinstance(data, dict):
                    # Иногда LLM возвращает один объект вместо массива
                    all_patches.append(data)
            except json.JSONDecodeError:
                # Пропускаем блоки, которые не смогли распарситься (может быть обычный текст)
                continue

        if not all_patches:
            self._dispatcher.dispatch(UI_ADD_LOG, "❌ Ошибка: Не найдено валидных JSON-инструкций в тексте.")
            self._dispatcher.dispatch(UI_ADD_LOG,
                                      "💡 Совет: Убедитесь, что ответ LLM содержит массив [...] в формате JSON.")
            return

        self._dispatcher.dispatch(UI_ADD_LOG, f"⚙️ Найдено патчей для применения: {len(all_patches)}")

        applied_count, logs = self._patch_service.apply_patches(all_patches, base_folders)

        for log in logs:
            self._dispatcher.dispatch(UI_ADD_LOG, log)

        if applied_count == len(all_patches):
            self._dispatcher.dispatch(UI_ADD_LOG,
                                      f"🎉 Итог: Успешно применено {applied_count}/{len(all_patches)} патчей!")
        else:
            self._dispatcher.dispatch(UI_ADD_LOG,
                                      f"⚠️ Итог: Применено {applied_count}/{len(all_patches)} патчей. Проверьте логи.")