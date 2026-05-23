import os
import re
from typing import Tuple, List, Dict


class PatchService:
    """Сервис для применения множества action-based патчей к файловой системе с умным поиском."""

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

    def _find_match_indices(self, content: str, search_text: str) -> Tuple[int, int]:
        """Умный поиск, игнорирующий разницу в отступах, пробелах и переносах строк."""
        if not search_text.strip():
            return -1, -1

        # 1. Попытка точного поиска (самый быстрый путь)
        idx = content.find(search_text)
        if idx != -1:
            return idx, idx + len(search_text)

        # 2. Умный поиск через Regex (устойчивость к пробелам и \r\n)
        tokens = re.split(r'(\s+)', search_text.strip())
        pattern_parts = []
        for t in tokens:
            if t.isspace():
                pattern_parts.append(r'\s+')
            else:
                pattern_parts.append(re.escape(t))

        pattern = "".join(pattern_parts)
        match = re.search(pattern, content)
        if match:
            return match.start(), match.end()

        return -1, -1

    def _handle_action(self, action: str, item: Dict, base_folders: List[str]) -> Tuple[bool, str]:
        file_target = item['file']
        search_text = item.get('search', '')
        content_text = item.get('content', item.get('replace', ''))

        if not base_folders:
            return False, "⚠️ Нет выбранных папок рабочей области."

        base_dir = base_folders[0]

        if action == 'create':
            clean_target = file_target.lstrip('\\/')
            target_path = os.path.join(base_dir, clean_target)
            os.makedirs(os.path.dirname(target_path), exist_ok=True)
            with open(target_path, 'w', encoding='utf-8') as f:
                f.write(content_text)
            return True, f"🌟 Создан файл: {clean_target}"

        actual_path = self._resolve_existing_file(file_target, base_folders)
        if not actual_path:
            return False, f"⚠️ Файл не найден на диске: {file_target}"
        file_name = os.path.basename(actual_path)

        if action == 'delete':
            if os.path.isfile(actual_path):
                os.remove(actual_path)
                return True, f"🗑️ Удален файл: {file_name}"
            return False, f"⚠️ Не удалось удалить файл: {actual_path}"

        with open(actual_path, 'r', encoding='utf-8') as f:
            file_content = f.read()

        new_content = file_content

        if action == 'append':
            new_content = file_content + ("\n" if not file_content.endswith("\n") else "") + content_text
            msg = f"➕ Добавлен текст в конец: {file_name}"

        elif action == 'prepend':
            new_content = content_text + ("\n" if not content_text.endswith("\n") else "") + file_content
            msg = f"➕ Добавлен текст в начало: {file_name}"

        elif action in ('replace', 'insert_before', 'insert_after'):
            if not search_text:
                return False, f"⚠️ Пропуск {action}: пустой 'search' для {file_target}"

            start_idx, end_idx = self._find_match_indices(file_content, search_text)

            if start_idx == -1:
                return False, f"⚠️ Строка поиска не найдена в {file_name}. Убедитесь, что код существует."

            if action == 'replace':
                new_content = file_content[:start_idx] + content_text + file_content[end_idx:]
                msg = f"✅ Успешно заменен блок в {file_name}"
            elif action == 'insert_before':
                new_content = file_content[:start_idx] + content_text + "\n" + file_content[start_idx:]
                msg = f"⤴️ Вставлен текст перед блоком в {file_name}"
            elif action == 'insert_after':
                new_content = file_content[:end_idx] + "\n" + content_text + file_content[end_idx:]
                msg = f"⤵️ Вставлен текст после блока в {file_name}"
        else:
            return False, f"⚠️ Неизвестный action: '{action}'"

        with open(actual_path, 'w', encoding='utf-8') as f:
            f.write(new_content)

        return True, msg

    def _resolve_existing_file(self, target_name: str, folders: List[str]) -> str:
        target_norm = os.path.normpath(target_name)
        for folder in folders:
            for root, _, files in os.walk(folder):
                for f in files:
                    full_path = os.path.join(root, f)
                    if full_path.endswith(target_norm):
                        return full_path
        return None