import os
import re
import difflib
from typing import Tuple, List, Dict
from src.i18n import tr


def _resolve_existing_file(target_name: str, folders: List[str]) -> str | None:
    target_norm = os.path.normpath(target_name)
    for folder in folders:
        for root, _, files in os.walk(folder):
            for f in files:
                full_path = os.path.join(root, f)
                if full_path.endswith(target_norm):
                    return full_path
    return None


def apply_prepared(prepared_patches: List[Dict]) -> Tuple[int, List[str]]:
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
            logs.append(tr("patch_service.apply.error", file_target=p['file_target'], e=e))
    return applied_count, logs


def _find_match_indices(content: str, search_text: str) -> Tuple[int, int, float]:
    if not search_text.strip():
        return -1, -1, 0.0
    idx = content.find(search_text)
    if idx != -1:
        return idx, idx + len(search_text), 0.0

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
        return match.start(), match.end(), 0.0

    search_len = len(search_text)
    threshold = 0.85
    best_ratio = 0.0
    best_pos = (-1, -1)
    step = max(1, search_len // 10)
    for start in range(0, max(len(content) - search_len + 1, 1), step):
        window = content[start:start + search_len]
        ratio = difflib.SequenceMatcher(None, search_text, window).ratio()
        if ratio > best_ratio:
            best_ratio = ratio
            best_pos = (start, start + search_len)
        if best_ratio >= 0.95:
            break
    if best_ratio >= threshold:
        return best_pos[0], best_pos[1], best_ratio
    return -1, -1, 0.0


def prepare_patches(patches: List[Dict], base_folders: List[str]) -> List[Dict]:
    """Только подготавливает патчи в памяти (Dry Run) для предпросмотра."""
    prepared = []
    for item in patches:
        action = item.get('action', 'replace').lower()
        file_target = item.get('file')

        if not file_target:
            prepared.append({'success': False, 'file_target': 'Unknown', 'msg': tr("patch_service.prepare.missing_file")})
            continue

        search_text = item.get('search', '')
        content_text = item.get('content', item.get('replace', ''))

        if not base_folders:
            prepared.append({'success': False, 'file_target': file_target, 'msg': tr("patch_service.prepare.no_folders")})
            continue

        base_dir = base_folders[0]
        actual_path = _resolve_existing_file(file_target, base_folders)

        if action == 'create':
            clean_target = file_target.lstrip('\\/')
            target_path = os.path.join(base_dir, clean_target)
            prepared.append({
                'success': True, 'action': action, 'file_target': file_target,
                'actual_path': target_path, 'original_content': "",
                'patched_content': content_text, 'msg': tr("patch_service.prepare.created", clean_target=clean_target)
            })
            continue

        if not actual_path:
            prepared.append({'success': False, 'file_target': file_target, 'msg': tr("patch_service.prepare.not_found", file_target=file_target)})
            continue

        if action == 'delete':
            prepared.append({
                'success': True, 'action': action, 'file_target': file_target,
                'actual_path': actual_path, 'original_content': tr("patch_service.prepare.will_delete"),
                'patched_content': "", 'msg': tr("patch_service.prepare.deleted", basename=os.path.basename(actual_path))
            })
            continue

        with open(actual_path, 'r', encoding='utf-8') as f:
            file_content = f.read()

        new_content = file_content
        msg = ""
        fuzzy_marker = 0.0

        if action == 'append':
            new_content = file_content + ("\n" if not file_content.endswith("\n") else "") + content_text
            msg = tr("patch_service.prepare.appended", basename=os.path.basename(actual_path))
            success = True
        elif action == 'prepend':
            new_content = content_text + ("\n" if not content_text.endswith("\n") else "") + file_content
            msg = tr("patch_service.prepare.prepended", basename=os.path.basename(actual_path))
            success = True
        elif action in ('replace', 'insert_before', 'insert_after'):
            if not search_text:
                success, msg = False, tr("patch_service.prepare.empty_search")
            else:
                start_idx, end_idx, fuzzy_marker = _find_match_indices(file_content, search_text)
                if start_idx == -1:
                    success, msg = False, tr("patch_service.prepare.search_not_found")
                else:
                    if action == 'replace':
                        new_content = file_content[:start_idx] + content_text + file_content[end_idx:]
                        msg = tr("patch_service.prepare.replaced", basename=os.path.basename(actual_path))
                    elif action == 'insert_before':
                        new_content = file_content[:start_idx] + content_text + "\n" + file_content[start_idx:]
                        msg = tr("patch_service.prepare.inserted_before", basename=os.path.basename(actual_path))
                    elif action == 'insert_after':
                        new_content = file_content[:end_idx] + "\n" + content_text + file_content[end_idx:]
                        msg = tr("patch_service.prepare.inserted_after", basename=os.path.basename(actual_path))
                    success = True
        else:
            success, msg = False, tr("patch_service.prepare.unknown_action", action=action)

        if fuzzy_marker:
            msg += tr("patch_service.prepare.fuzzy_match", fuzzy_marker=fuzzy_marker)
        prepared.append({
            'success': success, 'action': action, 'file_target': file_target,
            'actual_path': actual_path, 'original_content': file_content,
            'patched_content': new_content, 'msg': msg
        })

    return prepared


class PatchService:
    # ponytail: delegate to module-level pure functions
    def prepare_patches(self, all_patches, base_folders):
        return prepare_patches(all_patches, base_folders)

    def apply_prepared(self, prepared_patches):
        return apply_prepared(prepared_patches)

