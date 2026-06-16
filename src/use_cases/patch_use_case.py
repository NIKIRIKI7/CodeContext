import json
import re
from src.i18n import tr
from ..services.patch_service import PatchService
from ..store.state import AppState


class PatchUseCase:
    def __init__(self, state: AppState, patch_service: PatchService):
        self.state = state
        self._patch_service = patch_service

    def prepare_json_patch(self, patch_str: str, base_folders: list) -> list:
        if not patch_str or not patch_str.strip():
            self.state.add_log(tr("patch_use_case.patch.empty"))
            return []

        self.state.add_log(tr("patch_use_case.parsing.parsing_llm"))

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
                self.state.add_log(tr("patch_use_case.parsing.parse_error", error=e))
                continue

        if not all_patches:
            self.state.add_log(tr("patch_use_case.parsing.no_valid_json"))
            return []

        self.state.add_log(tr("patch_use_case.patch.preparing", count=len(all_patches)))
        return self._patch_service.prepare_patches(all_patches, base_folders)

    def apply_prepared(self, prepared_patches: list):
        if not prepared_patches:
            return 0, []

        applied_count, logs = self._patch_service.apply_prepared(prepared_patches)
        for log in logs:
            self.state.add_log(log)

        self.state.add_log(tr("patch_use_case.patch.applied", count=applied_count))
        return applied_count, logs
