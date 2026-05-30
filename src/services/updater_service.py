import os
import sys
import json
import urllib.request
from urllib.error import URLError, HTTPError
import re
import platform
import asyncio
from typing import Optional, Tuple, Callable
from ..utils.logger import app_logger


class UpdaterService:
    """Сервис для проверки и загрузки обновлений с GitHub с полным логированием."""
    REPO_API_URL = "https://api.github.com/repos/NIKIRIKI7/CodeContext/releases"

    async def check_for_updates(self, current_version: str, include_prerelease: bool) -> Optional[dict]:
        return await asyncio.to_thread(self._check_sync, current_version, include_prerelease)

    def _check_sync(self, current_version: str, include_prerelease: bool) -> Optional[dict]:
        app_logger.info(f"[Updater] Проверка обновлений. Текущая: {current_version}, Prerelease: {include_prerelease}")
        try:
            req = urllib.request.Request(
                self.REPO_API_URL,
                headers={
                    "Accept": "application/vnd.github.v3+json",
                    "User-Agent": "CodeContextAI-App"
                }
            )
            app_logger.debug(f"[Updater] Отправка запроса к {self.REPO_API_URL}")

            with urllib.request.urlopen(req, timeout=10) as response:
                releases = json.loads(response.read().decode('utf-8'))

            app_logger.debug(f"[Updater] Получено {len(releases)} релизов от GitHub.")

            curr_tuple = self._parse_version(current_version)
            latest_release = None

            for release in releases:
                if release.get("prerelease") and not include_prerelease:
                    continue

                rel_tuple = self._parse_version(release.get("tag_name", "v0.0.0"))
                app_logger.debug(f"[Updater] Анализ релиза: {release.get('tag_name')} -> parsed: {rel_tuple}")

                if rel_tuple > curr_tuple:
                    latest_release = release
                    break

            if not latest_release:
                app_logger.info("[Updater] Новых версий не найдено. Установлена самая актуальная.")
                return None

            sys_os = platform.system()
            asset_url = None

            app_logger.info(f"[Updater] Найдена новая версия: {latest_release.get('tag_name')}. Поиск ассетов под {sys_os}...")
            for asset in latest_release.get('assets', []):
                name = asset['name'].lower()
                app_logger.debug(f"[Updater] Проверка ассета: {name}")
                if sys_os == "Windows" and "windows" in name and name.endswith(".exe"):
                    asset_url = asset['browser_download_url']
                    break
                elif sys_os == "Linux" and "linux" in name and not name.endswith(".exe"):
                    asset_url = asset['browser_download_url']
                    break

            if not asset_url:
                app_logger.warning(f"[Updater] Ассет для {sys_os} не найден в релизе {latest_release.get('tag_name')}.")
                return None

            app_logger.info(f"[Updater] Ассет успешно найден: {asset_url}")
            return {
                "version": latest_release.get("tag_name"),
                "notes": latest_release.get("body", "Описание отсутствует."),
                "download_url": asset_url
            }

        except HTTPError as e:
            app_logger.error(f"[Updater] Ошибка API GitHub: Код {e.code} - {e.read().decode('utf-8')}")
            raise RuntimeError(f"Сбой API GitHub: Код {e.code}")
        except Exception as e:
            app_logger.error(f"[Updater] Критическая ошибка проверки обновлений: {e}")
            raise RuntimeError(f"Ошибка соединения: {e}")

    async def download_and_install(self, download_url: str, progress_cb: Callable[[float], None]) -> bool:
        app_logger.info(f"[Updater] Запуск скачивания с URL: {download_url}")
        if not getattr(sys, 'frozen', False):
            app_logger.error("[Updater] Автообновление вызвано не из скомпилированного файла.")
            raise RuntimeError("Автообновление работает только в скомпилированной версии (PyInstaller).")

        current_exe = sys.executable
        old_exe = current_exe + ".old"
        return await asyncio.to_thread(self._download_sync, download_url, current_exe, old_exe, progress_cb)

    def _download_sync(self, url: str, target: str, old_target: str, cb: Callable[[float], None]) -> bool:
        try:
            if os.path.exists(target):
                app_logger.info(f"[Updater] Переименование текущего файла {target} -> {old_target}")
                os.rename(target, old_target)

            req = urllib.request.Request(url, headers={"User-Agent": "CodeContextAI-App"})
            app_logger.info("[Updater] Открытие стрима для скачивания файла...")

            with urllib.request.urlopen(req, timeout=30) as response, open(target, 'wb') as out_file:
                total_size = int(response.info().get('Content-Length', 0))
                app_logger.info(f"[Updater] Размер файла: {total_size / 1024 / 1024:.2f} MB")

                downloaded = 0
                block_size = 32768
                last_log_mb = 0

                while True:
                    buffer = response.read(block_size)
                    if not buffer:
                        break
                    out_file.write(buffer)
                    downloaded += len(buffer)

                    if total_size > 0 and cb:
                        cb(downloaded / total_size)

                    current_mb = int(downloaded / (1024 * 1024))
                    if current_mb > last_log_mb:
                        app_logger.debug(f"[Updater] Скачано: {current_mb} MB / {total_size / 1024 / 1024:.2f} MB")
                        last_log_mb = current_mb

            if platform.system() == "Linux":
                app_logger.info("[Updater] Установка прав 755 для Linux")
                os.chmod(target, 0o755)

            app_logger.info("[Updater] Скачивание и подмена файла успешно завершены.")
            return True
        except Exception as e:
            app_logger.error(f"[Updater] Ошибка при скачивании: {e}")
            if os.path.exists(old_target):
                app_logger.info("[Updater] Откат: восстановление старого исполняемого файла.")
                if os.path.exists(target):
                    os.remove(target)
                os.rename(old_target, target)
            raise RuntimeError(f"Сбой загрузки: {e}")

    def _parse_version(self, v_str: str) -> Tuple[int, int, int, int, int]:
        match = re.match(r'v?(\d+)\.(\d+)\.(\d+)(?:-pre\.(\d+))?', v_str)
        if not match:
            return (0, 0, 0, 0, 0)
        maj, min, pat, pre = match.groups()
        if pre:
            return (int(maj), int(min), int(pat), -1, int(pre))
        return (int(maj), int(min), int(pat), 0, 0)
