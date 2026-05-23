"""
ProcessWorkspaceUseCase — сценарий обработки файлов и экспорта результата.

Исправлено:
- Заменены несуществующие action types (WORKFLOW_STARTED, LOG_MESSAGE_ADDED,
  PROCESSING_COMPLETED, FORMATTING_COMPLETED) на определённые в action_types.py.
- Убран import ProcessedFile изнутри метода (был признаком плохой структуры).
- Добавлен Optional[str] для save_path в pdf-экспорте.
"""

import asyncio
from typing import Optional, List, Dict, Set

from ..actions.action_types import (
    UI_SET_LOADING, UI_ADD_LOG,
    PROCESSING_SUCCESS, FORMATTING_SUCCESS,
    WORKFLOW_STARTED, WORKFLOW_PROGRESS, WORKFLOW_FINISHED, WORKFLOW_ERROR,
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


class ProcessWorkspaceUseCase:
    """
    Оркестрирует чтение → очистку → форматирование → экспорт.
    Не содержит бизнес-логики — только координацию сервисов.
    """

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
        """
        Выполняет полный pipeline обработки.

        Args:
            state: снимок состояния приложения.
            target: 'clipboard' | 'file' | 'pdf'
            save_path: путь для сохранения (обязателен для 'file' и 'pdf')
        """
        self._dispatcher.dispatch(UI_SET_LOADING, True)
        self._dispatcher.dispatch(WORKFLOW_STARTED, {
            'message': "Подготовка файлов...", 'progress': 0.1
        })

        try:
            files_to_process = [
                p for p in state.scanned_files_paths
                if p not in state.manual_exclusions
            ]

            if not files_to_process:
                self._dispatcher.dispatch(UI_ADD_LOG, "⚠️ Нет файлов для обработки (все исключены?)")
                return

            # 1. Чтение файлов
            self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                'message': f"Чтение {len(files_to_process)} файлов...", 'progress': 0.3
            })
            raw_files = await self._process_service.read_files_async(files_to_process)

            # 2. Анализ зависимостей (опционально)
            dependency_map: Optional[Dict[str, Set[str]]] = None
            if state.settings.include_dependencies:
                self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                    'message': "Анализ зависимостей...", 'progress': 0.5
                })
                dependency_map = await self._dependency_service.resolve_dependencies(raw_files)

            # 3. Обработка (CPU-heavy → отдельный поток)
            self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                'message': "Обработка и минификация...", 'progress': 0.6
            })
            processed: List[ProcessedFile] = await asyncio.to_thread(
                self._run_processing, raw_files, state.settings
            )
            self._dispatcher.dispatch(PROCESSING_SUCCESS, processed)

            # 4. Форматирование
            self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                'message': "Форматирование...", 'progress': 0.85
            })
            text_result: str = await asyncio.to_thread(
                self._run_formatting, processed, state.settings, dependency_map
            )
            total_tokens = sum(f.tokens for f in processed)
            self._dispatcher.dispatch(FORMATTING_SUCCESS, {
                'text': text_result, 'tokens': total_tokens
            })

            # 5. Экспорт
            self._dispatcher.dispatch(WORKFLOW_PROGRESS, {
                'message': "Сохранение...", 'progress': 0.95
            })
            await self._export(target, text_result, save_path)

            self._dispatcher.dispatch(WORKFLOW_FINISHED, None)
            self._dispatcher.dispatch(UI_ADD_LOG, f"✅ Готово. Токенов: ~{total_tokens}")

        except Exception as exc:
            self._dispatcher.dispatch(WORKFLOW_ERROR, str(exc))
            self._dispatcher.dispatch(UI_ADD_LOG, f"🔥 Ошибка обработки: {exc}")

        finally:
            self._dispatcher.dispatch(UI_SET_LOADING, False)

    # ------------------------------------------------------------------
    # Private helpers (CPU-bound, выполняются в threadpool)
    # ------------------------------------------------------------------

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
        )

    async def _export(self, target: str, text: str, save_path: Optional[str]):
        if target == 'clipboard':
            self._output_service.copy_to_clipboard(text)
            self._dispatcher.dispatch(UI_ADD_LOG, "📋 Скопировано в буфер обмена")
        elif target == 'file' and save_path:
            self._output_service.save_to_file(text, save_path)
            self._dispatcher.dispatch(UI_ADD_LOG, f"💾 Сохранено в {save_path}")
        elif target == 'pdf' and save_path:
            await asyncio.to_thread(self._output_service.save_to_pdf, text, save_path)
            self._dispatcher.dispatch(UI_ADD_LOG, f"📄 PDF создан: {save_path}")