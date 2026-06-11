import asyncio
import datetime
from typing import Optional, List, Dict, Set
from ..actions.action_types import (
    UI_SET_LOADING, UI_ADD_LOG, UI_SHOW_PREVIEW, UI_SHOW_CHAT,
    PROCESSING_SUCCESS, FORMATTING_SUCCESS,
    WORKFLOW_STARTED, WORKFLOW_PROGRESS, WORKFLOW_FINISHED, WORKFLOW_ERROR,
    HISTORY_ADD, SET_BEFORE_AFTER
)
from ..actions.dispatcher import Dispatcher
from ..services.cleaner_service import CleanerService
from ..services.dependency_service import DependencyService
from ..services.formatting_service import FormattingService
from ..services.output_service import OutputService
from ..services.processing_service import ProcessingService
from ..services.skeleton_service import SkeletonService
from ..services.token_service import TokenService
from ..store.state import AppState, ProcessedFile
from ..utils.pipeline_utils import PipelineUtils
from src.i18n import tr


class ProcessWorkspaceUseCase:
    """Оркестрирует чтение — очистку — форматирование — экспорт."""

    def __init__(
        self,
        dispatcher: Dispatcher,
        process_service: ProcessingService,
        dependency_service: DependencyService,
        cleaner_service: CleanerService,
        skeleton_service: SkeletonService,
        token_service: TokenService,
        format_service: FormattingService,
        output_service: OutputService,
    ):
        self._dispatcher = dispatcher
        self._process_service = process_service
        self._dependency_service = dependency_service
        self._cleaner_service = cleaner_service
        self._skeleton_service = skeleton_service
        self._token_service = token_service
        self._format_service = format_service
        self._output_service = output_service

    async def execute(
        self,
        state: AppState,
        target: str,
        save_path: Optional[str] = None,
    ) -> None:
        self._dispatcher.dispatch(UI_SET_LOADING, True)
        self._dispatcher.dispatch(WORKFLOW_STARTED, {
            'message': tr("process_use_case.workflow.preparing"), 'progress': 0.1
        })

        try:
            files_to_process = [
                p for p in state.scanned_files_paths
                if p not in state.manual_exclusions
            ]

            if not files_to_process:
                self._dispatcher.dispatch(UI_ADD_LOG, tr("process_use_case.workflow.no_files"))
                return

            self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                'message': tr("process_use_case.workflow.reading", count=len(files_to_process)), 'progress': 0.3
            })

            raw_files = await self._process_service.read_files_async(files_to_process)

            dependency_map: Optional[Dict[str, Set[str]]] = None
            if state.settings.include_dependencies or state.settings.include_mermaid:
                self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                    'message': tr("process_use_case.workflow.analyzing"), 'progress': 0.5
                })
                dependency_map = await self._dependency_service.resolve_dependencies(raw_files)

            self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                'message': tr("process_use_case.workflow.processing_minifying"), 'progress': 0.6
            })

            processed: List[ProcessedFile] = await asyncio.to_thread(
                self._run_processing, raw_files, state.settings
            )

            self._dispatcher.dispatch(PROCESSING_SUCCESS, processed)

            self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                'message': tr("process_use_case.workflow.formatting"), 'progress': 0.85
            })

            text_result: str = await asyncio.to_thread(
                self._run_formatting, processed, state.settings, dependency_map
            )

            final_tokens = await asyncio.to_thread(self._token_service.count_tokens, text_result)

            before_after = []
            for r, p in zip(raw_files, processed):
                before_after.append({"path": p.path, "original": r['content'], "processed": p.content})

            self._dispatcher.dispatch(SET_BEFORE_AFTER, before_after)

            timestamp = datetime.datetime.now().strftime("%H:%M:%S")
            self._dispatcher.dispatch(HISTORY_ADD, {
                "time": timestamp, "text": text_result, "tokens": final_tokens
            })

            self._dispatcher.dispatch(FORMATTING_SUCCESS, {
                'text': text_result, 'tokens': final_tokens
            })

            self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                'message': tr("process_use_case.workflow.saving"), 'progress': 0.95
            })

            await self._export(target, text_result, save_path)

            self._dispatcher.dispatch(WORKFLOW_FINISHED, None)
            self._dispatcher.dispatch(UI_ADD_LOG, tr("process_use_case.workflow.done", count=final_tokens))

        except Exception as exc:
            self._dispatcher.dispatch(WORKFLOW_ERROR, str(exc))
            self._dispatcher.dispatch(UI_ADD_LOG, tr("process_use_case.workflow.error", error=exc))
        finally:
            self._dispatcher.dispatch(UI_SET_LOADING, False)

    def _run_processing(self, raw_files, settings) -> List[ProcessedFile]:
        return PipelineUtils.process_files_batch(
            raw_files=raw_files,
            options=settings,
            cleaner_service=self._cleaner_service,
            skeleton_service=self._skeleton_service,
            token_service=self._token_service,
        )

    def _run_formatting(self, processed, settings, dep_map) -> str:
        return self._format_service.format_output(
            files=processed,
            fmt=settings.output_format,
            include_tree=settings.include_tree,
            system_prompt=settings.system_prompt,
            dependency_map=dep_map,
            template_path=settings.template_path,
            include_mermaid=settings.include_mermaid
        )

    async def _export(self, target: str, text: str, save_path: Optional[str]):
        if target == 'editor':
            external_cmd = self._dispatcher._store.state.settings.external_editor
            await asyncio.to_thread(self._output_service.open_in_editor, text, external_cmd)
            self._dispatcher.dispatch(UI_ADD_LOG, tr("process_use_case.export.editor"))
        elif target == 'clipboard':
            self._output_service.copy_to_clipboard(text)
        elif target == 'stdout':
            import sys
            out = sys.__stdout__
            out.reconfigure(encoding='utf-8')
            out.write(text)
            out.write("\n")
            out.flush()
        elif target == 'preview':
            self._dispatcher.dispatch(UI_SHOW_PREVIEW, text)
        elif target == 'chat':
            self._dispatcher.dispatch(UI_SHOW_CHAT, text)
            self._dispatcher.dispatch(UI_ADD_LOG, tr("process_use_case.export.chat_loaded"))
        elif target == 'file' and save_path:
            self._output_service.save_to_file(text, save_path)
            self._dispatcher.dispatch(UI_ADD_LOG, tr("process_use_case.export.saved", path=save_path))
        elif target == 'pdf' and save_path:
            await asyncio.to_thread(self._output_service.save_to_pdf, text, save_path)
            self._dispatcher.dispatch(UI_ADD_LOG, tr("process_use_case.export.pdf_created", path=save_path))
