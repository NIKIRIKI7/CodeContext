import os
import asyncio
import datetime
import gc
import json
from pathlib import Path
from typing import Optional, List

from ..store.state import AppState, ProcessedFile
from ..utils.config import get_app_data_dir, MAX_FILE_SIZE_MB
from ..utils.pipeline_utils import PipelineUtils
from src.i18n import tr
from ..services import dependency_service, formatting_service, output_service, token_service, cleaner_service, skeleton_service

_CHECKPOINT_FILE = os.path.join(get_app_data_dir(), "processing_checkpoint.json")

def read_file_sync(path: str) -> str | None:
    try:
        # ponytail: Отказ от mmap(). Дисковый кэш ОС делает read_text() достаточно быстрым.
        with open(path, 'rb') as f:
            if b'\x00' in f.read(1024): return None
        return Path(path).read_text(encoding='utf-8', errors='replace')
    except OSError:
        return None

async def read_file_async(path: str) -> str | None:
    return await asyncio.to_thread(read_file_sync, path)

def _save_checkpoint(processed: List[ProcessedFile]) -> None:
    try:
        with open(_CHECKPOINT_FILE, 'w', encoding='utf-8') as f:
            json.dump([{"path": p.path, "content": p.content, "tokens": p.tokens} for p in processed], f, ensure_ascii=False)
    except Exception: pass

def _clear_checkpoint() -> None:
    try: os.remove(_CHECKPOINT_FILE)
    except Exception: pass

async def _export(target: str, text: str, save_path: Optional[str], state: AppState):
    if target == 'editor':
        external_cmd = getattr(state.settings, 'external_editor', '')
        await asyncio.to_thread(output_service.open_in_editor, text, external_cmd)
        state.add_log(tr("process_use_case.export.editor"))
    elif target == 'clipboard':
        output_service.copy_to_clipboard(text)
    elif target == 'stdout':
        import sys
        out = sys.__stdout__
        out.reconfigure(encoding='utf-8')
        out.write(text + "\n")
        out.flush()
    elif target == 'preview':
        state.preview_text = text
        state.show_preview = True
    elif target == 'chat':
        state.chat_context = text
        state.show_chat = True
        state.add_log(tr("process_use_case.export.chat_loaded"))
    elif target == 'file' and save_path:
        output_service.save_to_file(text, save_path)
        state.add_log(tr("process_use_case.export.saved", path=save_path))

class ProcessWorkspaceUseCase:
    async def execute(self, state: AppState, target: str, save_path: Optional[str] = None) -> None:
        state.is_loading = True
        state.status_message = tr("process_use_case.workflow.preparing")
        state.notify()
        gc.disable()
        
        try:
            files_to_process = [p for p in state.scanned_files_paths if p not in state.manual_exclusions]
            if not files_to_process:
                state.add_log(tr("process_use_case.workflow.no_files"))
                return
                
            raw_files = []
            for path in files_to_process:
                if not os.path.exists(path) or os.path.getsize(path) > MAX_FILE_SIZE_MB * 1024 * 1024: continue
                content = await read_file_async(path)
                if content is not None:
                    raw_files.append({"path": path, "content": content, "ext": os.path.splitext(path)[1].lower()})
                    
            dependency_map = None
            if state.settings.include_dependencies or getattr(state.settings, 'include_mermaid', False):
                dependency_map = await dependency_service.resolve_dependencies(raw_files)
                
            if target == 'file' and save_path:
                await self._stream_to_file(files_to_process, state, save_path)
                return
                
            processed = await asyncio.to_thread(PipelineUtils.process_files_batch_parallel, raw_files, state.settings)
            
            if getattr(state.settings, 'save_checkpoints', False): _save_checkpoint(processed)
            state.processed_files = processed
            
            text_result = await asyncio.to_thread(
                formatting_service.format_output,
                processed, state.settings.output_format, state.settings.include_tree,
                state.settings.system_prompt, dependency_map, state.settings.template_path,
                getattr(state.settings, 'include_mermaid', False), getattr(state.settings, 'deduplicate', False)
            )
            
            if getattr(state.settings, 'save_checkpoints', False): _clear_checkpoint()
            
            final_tokens = await asyncio.to_thread(token_service.count_tokens, text_result)
            state.before_after_data = [{"path": p.path, "original": next((r['content'] for r in raw_files if r['path'] == p.path), ""), "processed": p.content} for p in processed]
            
            timestamp = datetime.datetime.now().strftime("%H:%M:%S")
            state.preview_history.insert(0, {"time": timestamp, "text": text_result, "tokens": final_tokens})
            if len(state.preview_history) > 20: state.preview_history.pop()
            
            state.final_output_text = text_result
            state.total_tokens = final_tokens
            await _export(target, text_result, save_path, state)
            
        except Exception as exc:
            state.add_log(f"CRITICAL ERROR: {exc}")
        finally:
            gc.enable()
            gc.collect()
            state.is_loading = False
            state.notify()

    async def _stream_to_file(self, file_paths: list, state: AppState, save_path: str) -> int:
        settings = state.settings
        header_text = await asyncio.to_thread(
            formatting_service.format_output,
            [], settings.output_format,
            settings.include_tree, settings.system_prompt, None,
            settings.template_path, getattr(settings, 'include_mermaid', False), False
        )
        total_tokens = token_service.count_tokens(header_text)
        with open(save_path, 'w', encoding='utf-8') as f:
            f.write(header_text + "\n")
            for path in file_paths:
                content = read_file_sync(path)
                if not content: continue
                ext = os.path.splitext(path)[1].lower()
                cleaned = cleaner_service.clean(content, ext, settings)
                if getattr(settings, 'skeleton_mode', False):
                    cleaned = skeleton_service.make_skeleton(cleaned, ext)
                file_chunk = f"{'='*50}\nFILE: {path}\n{'='*50}\n{cleaned}\n"
                total_tokens += token_service.count_tokens(file_chunk)
                f.write(file_chunk + "\n")
        return total_tokens
