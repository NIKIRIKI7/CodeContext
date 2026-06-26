import json
import re
from src.i18n import tr
from ..services.patch_service import prepare_patches, apply_prepared
from ..store.state import AppState


def prepare_json_patch(state: AppState, patch_str: str, base_folders: list) -> list:
    if not patch_str or not patch_str.strip():
        state.add_log(tr("patch_use_case.patch.empty"))
        return []

    state.add_log(tr("patch_use_case.parsing.parsing_llm"))

    blocks = re.findall(r'```(?:json)?\s*(.*?)\s*```', patch_str, re.DOTALL)

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
            state.add_log(tr("patch_use_case.parsing.parse_error", error=e))
            continue

    if not all_patches:
        state.add_log(tr("patch_use_case.parsing.no_valid_json"))
        return []

    state.add_log(tr("patch_use_case.patch.preparing", count=len(all_patches)))
    return prepare_patches(all_patches, base_folders)


def apply_patches(state: AppState, prepared_patches: list):
    if not prepared_patches:
        return 0, []

    applied_count, logs = apply_prepared(prepared_patches)
    for log in logs:
        state.add_log(log)

    state.add_log(tr("patch_use_case.patch.applied", count=applied_count))
    return applied_count, logs
