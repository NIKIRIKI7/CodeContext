import os
import re
from typing import Tuple, List, Dict


class PatchService:
    """Сервис для применения action-based патчей с поддержкой Dry-Run."""

    def prepare_patches(self, patches: List[Dict], base_folders: List[str]) -> List[Dict]:
        """Только подготавливает патчи в памяти (Dry Run) для предпросмотра."""
        prepared = []
        for item in patches:
            action = item.get('action', 'replace').lower()
            file_target = item.get('file')

            if not file_target:
                prepared.append({'success': False, 'file_target': 'Unknown', 'msg': "Пропущен патч: нет 'file'"})
                continue

            search_text = item.get('search', '')
            content_text = item.get('content', item.get('replace', ''))

            if not base_folders:
                prepared.append({'success': False, 'file_target': file_target, 'msg': "Нет выбранных папок."})
                continue

            base_dir = base_folders[0]
            actual_path = self._resolve_existing_file(file_target, base_folders)

            if action == 'create':
                clean_target = file_target.lstrip('\\/')
                target_path = os.path.join(base_dir, clean_target)
                prepared.append({
                    'success': True, 'action': action, 'file_target': file_target,
                    'actual_path': target_path, 'original_content': "",
                    'patched_content': content_text, 'msg': f"🌟 Создан файл: {clean_target}"
                })
                continue

            if not actual_path:
                prepared.append({'success': False, 'file_target': file_target, 'msg': f"Файл не найден: {file_target}"})
                continue

            if action == 'delete':
                prepared.append({
                    'success': True, 'action': action, 'file_target': file_target,
                    'actual_path': actual_path, 'original_content': "(Файл будет удален)",
                    'patched_content': "", 'msg': f"🗑️ Удален файл: {os.path.basename(actual_path)}"
                })
                continue

            with open(actual_path, 'r', encoding='utf-8') as f:
                file_content = f.read()

            new_content = file_content
            success = False
            msg = ""

            if action == 'append':
                new_content = file_content + ("\n" if not file_content.endswith("\n") else "") + content_text
                msg = f"➕ Добавлен текст в конец: {os.path.basename(actual_path)}"
                success = True
            elif action == 'prepend':
                new_content = content_text + ("\n" if not content_text.endswith("\n") else "") + file_content
                msg = f"➕ Добавлен текст в начало: {os.path.basename(actual_path)}"
                success = True
            elif action in ('replace', 'insert_before', 'insert_after'):
                if not search_text:
                    success, msg = False, "Пустой 'search'"
                else:
                    start_idx, end_idx = self._find_match_indices(file_content, search_text)
                    if start_idx == -1:
                        success, msg = False, "Строка поиска не найдена."
                    else:
                        if action == 'replace':
                            new_content = file_content[:start_idx] + content_text + file_content[end_idx:]
                            msg = f"✅ Заменен блок в {os.path.basename(actual_path)}"
                        elif action == 'insert_before':
                            new_content = file_content[:start_idx] + content_text + "\n" + file_content[start_idx:]
                            msg = f"⤴️ Вставлен текст перед в {os.path.basename(actual_path)}"
                        elif action == 'insert_after':
                            new_content = file_content[:end_idx] + "\n" + content_text + file_content[end_idx:]
                            msg = f"⤵️ Вставлен текст после в {os.path.basename(actual_path)}"
                        success = True
            else:
                success, msg = False, f"Неизвестный action: '{action}'"

            prepared.append({
                'success': success, 'action': action, 'file_target': file_target,
                'actual_path': actual_path, 'original_content': file_content,
                'patched_content': new_content, 'msg': msg
            })

        return prepared

    def apply_prepared(self, prepared_patches: List[Dict]) -> Tuple[int, List[str]]:
        """Физически применяет выбранные патчи на диск."""
        applied_count = 0
        logs = []
        for p in prepared_patches:
            if not p['success']:
                continue
            try:
                if p['action'] == 'delete':
                    if os.path.isfile(p['actual_path']):
                        os.remove(p['actual_path'])
                else:
                    os.makedirs(os.path.dirname(p['actual_path']), exist_ok=True)
                    with open(p['actual_path'], 'w', encoding='utf-8') as f:
                        f.write(p['patched_content'])
                applied_count += 1
                logs.append(p['msg'])
            except Exception as e:
                logs.append(f"❌ Ошибка {p['file_target']}: {e}")
        return applied_count, logs

    def _find_match_indices(self, content: str, search_text: str) -> Tuple[int, int]:
        if not search_text.strip():
            return -1, -1
        idx = content.find(search_text)
        if idx != -1:
            return idx, idx + len(search_text)

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

    def _resolve_existing_file(self, target_name: str, folders: List[str]) -> str:
        target_norm = os.path.normpath(target_name)
        for folder in folders:
            for root, _, files in os.walk(folder):
                for f in files:
                    full_path = os.path.join(root, f)
                    if full_path.endswith(target_norm):
                        return full_path
        return None