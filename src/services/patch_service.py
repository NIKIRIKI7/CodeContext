import os
from typing import Tuple, List, Dict


class PatchService:
    """Сервис для применения множества action-based патчей к файловой системе."""

    def apply_patches(self, patches: List[Dict], base_folders: List[str]) -> Tuple[int, List[str]]:
        logs = []
        applied_count = 0

        for i, item in enumerate(patches):
            action = item.get('action', 'replace').lower()
            file_target = item.get('file')

            if not file_target:
                logs.append(f"⚠️ Пропущен патч #{i + 1}: не указан 'file'.")
                continue

            try:
                success, msg = self._handle_action(action, item, base_folders)
                logs.append(msg)
                if success:
                    applied_count += 1
            except Exception as e:
                logs.append(f"❌ Критическая ошибка при обработке '{file_target}': {e}")

        return applied_count, logs

    def _handle_action(self, action: str, item: Dict, base_folders: List[str]) -> Tuple[bool, str]:
        file_target = item['file']
        search_text = item.get('search', '')

        # Поддержка 'replace' как алиаса для 'content' (для обратной совместимости)
        content_text = item.get('content', item.get('replace', ''))

        if not base_folders:
            return False, "⚠️ Нет выбранных папок рабочей области."

        base_dir = base_folders[0]

        # 1. Действия, НЕ требующие существования файла:
        if action == 'create':
            clean_target = file_target.lstrip('\\/')
            target_path = os.path.join(base_dir, clean_target)
            os.makedirs(os.path.dirname(target_path), exist_ok=True)
            with open(target_path, 'w', encoding='utf-8') as f:
                f.write(content_text)
            return True, f"🌟 Создан файл: {clean_target}"

        # --- Ищем существующий файл для остальных действий ---
        actual_path = self._resolve_existing_file(file_target, base_folders)
        if not actual_path:
            return False, f"⚠️ Файл не найден на диске: {file_target}"
        file_name = os.path.basename(actual_path)

        # 2. Удаление
        if action == 'delete':
            if os.path.isfile(actual_path):
                os.remove(actual_path)
                return True, f"🗑️ Удален файл: {file_name}"
            return False, f"⚠️ Не удалось удалить файл (не найден): {actual_path}"

        # --- Чтение содержимого для операций модификации ---
        with open(actual_path, 'r', encoding='utf-8') as f:
            file_content = f.read()

        new_content = file_content

        # 3. Добавление в конец/начало
        if action == 'append':
            new_content = file_content + ("\n" if not file_content.endswith("\n") else "") + content_text
            msg = f"➕ Добавлен текст в конец: {file_name}"

        elif action == 'prepend':
            new_content = content_text + ("\n" if not content_text.endswith("\n") else "") + file_content
            msg = f"➕ Добавлен текст в начало: {file_name}"

        # 4. Точечная замена или вставка
        elif action in ('replace', 'insert_before', 'insert_after'):
            if not search_text:
                return False, f"⚠️ Пропуск {action}: пустой 'search' для {file_target}"

            if search_text not in file_content:
                return False, f"⚠️ Строка поиска не найдена в {file_name}. LLM ошиблась в отступах или спецсимволах."

            if action == 'replace':
                new_content = file_content.replace(search_text, content_text)
                msg = f"✅ Успешно заменен блок в {file_name}"
            elif action == 'insert_before':
                new_content = file_content.replace(search_text, content_text + "\n" + search_text)
                msg = f"⤴️ Вставлен текст перед блоком в {file_name}"
            elif action == 'insert_after':
                new_content = file_content.replace(search_text, search_text + "\n" + content_text)
                msg = f"⤵️ Вставлен текст после блока в {file_name}"
        else:
            return False, f"⚠️ Неизвестный action: '{action}'"

        # Запись изменений
        with open(actual_path, 'w', encoding='utf-8') as f:
            f.write(new_content)

        return True, msg

    def _resolve_existing_file(self, target_name: str, folders: List[str]) -> str:
        """Ищет уже существующий файл по имени или частичному пути."""
        target_norm = os.path.normpath(target_name)
        for folder in folders:
            for root, _, files in os.walk(folder):
                for f in files:
                    full_path = os.path.join(root, f)
                    if full_path.endswith(target_norm):
                        return full_path
        return None