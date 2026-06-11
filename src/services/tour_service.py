from typing import List, Dict
from src.i18n import tr


class TourService:
    """
    Доменный сервис интерактивного тура.
    Покрывает 100% функционала приложения для новых пользователей.
    """

    def get_tour_steps(self) -> List[Dict[str, str]]:
        return [
            {
                "title": tr("tour_service.welcome.title"),
                "text": tr("tour_service.welcome.text")
            },
            {
                "title": tr("tour_service.folders_pr.title"),
                "text": tr("tour_service.folders_pr.text")
            },
            {
                "title": tr("tour_service.filtering.title"),
                "text": tr("tour_service.filtering.text")
            },
            {
                "title": tr("tour_service.compression.title"),
                "text": tr("tour_service.compression.text")
            },
            {
                "title": tr("tour_service.architecture.title"),
                "text": tr("tour_service.architecture.text")
            },
            {
                "title": tr("tour_service.editor_preview.title"),
                "text": tr("tour_service.editor_preview.text")
            },
            {
                "title": tr("tour_service.local_llm.title"),
                "text": tr("tour_service.local_llm.text")
            },
            {
                "title": tr("tour_service.ai_chat.title"),
                "text": tr("tour_service.ai_chat.text")
            },
            {
                "title": tr("tour_service.patcher.title"),
                "text": tr("tour_service.patcher.text")
            },
            {
                "title": tr("tour_service.safety_diff.title"),
                "text": tr("tour_service.safety_diff.text")
            },
            {
                "title": tr("tour_service.os_integration.title"),
                "text": tr("tour_service.os_integration.text")
            }
        ]
