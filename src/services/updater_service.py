import os
import sys
import json
import urllib.request
from urllib.error import URLError, HTTPError
import re
import platform
import asyncio
import zipfile
import shutil
from typing import Optional, Tuple, Callable
from ..utils.logger import app_logger
from src.i18n import tr

class UpdaterService:
    """Сервис для проверки и загрузки обновлений с GitHub с полной поддержкой Mac/Linux/Win."""

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
            sys_arch = platform.machine().lower()
            asset_url = None
            
            app_logger.info(f"[Updater] Найдена новая версия: {latest_release.get('tag_name')}. Поиск ассетов под {sys_os} ({sys_arch})...")

            for asset in latest_release.get('assets', []):
                name = asset['name'].lower()
                app_logger.debug(f"[Updater] Проверка ассета: {name}")

                if sys_os == "Windows" and "windows" in name and name.endswith(".exe"):
                    asset_url = asset['browser_download_url']
                    break
                elif sys_os == "Linux" and "linux" in name and not name.endswith(".exe"):
                    asset_url = asset['browser_download_url']
                    break
                elif sys_os == "Darwin" and "macos" in name and name.endswith(".zip"):
                    if ("arm64" in sys_arch or "aarch64" in sys_arch) and "arm64" in name:
                        asset_url = asset['browser_download_url']
                        break
                    elif ("x86_64" in sys_arch or "amd64" in sys_arch) and "x86_64" in name:
                        asset_url = asset['browser_download_url']
                        break

            if not asset_url:
                app_logger.warning(f"[Updater] Ассет для {sys_os} ({sys_arch}) не найден в релизе {latest_release.get('tag_name')}.")
                return None

            app_logger.info(f"[Updater] Ассет успешно найден: {asset_url}")
            return {
                "version": latest_release.get("tag_name"),
                "notes": latest_release.get("body", tr("updater_service.check.no_description")),
                "download_url": asset_url
            }

        except HTTPError as e:
            app_logger.error(f"[Updater] Ошибка API GitHub: Код {e.code} - {e.read().decode('utf-8')}")
            raise RuntimeError(tr("updater_service.check.github_api_error", code=e.code))
        except Exception as e:
            app_logger.error(f"[Updater] Критическая ошибка проверки обновлений: {e}")
            raise RuntimeError(tr("updater_service.check.connection_error", error=e))

    async def download_and_install(self, download_url: str, progress_cb: Callable[[float], None]) -> bool:
        app_logger.info(f"[Updater] Запуск скачивания с URL: {download_url}")
        
        if not getattr(sys, 'frozen', False):
            app_logger.error("[Updater] Автообновление вызвано не из скомпилированного файла.")
            raise RuntimeError(tr("updater_service.download.not_compiled"))

        current_exe = sys.executable
        return await asyncio.to_thread(self._download_sync, download_url, current_exe, progress_cb)

    def _get_mac_app_path(self, exe_path: str) -> str:
        """Находит корень .app бандла на macOS"""
        parts = exe_path.split(os.sep)
        if "Contents" in parts and "MacOS" in parts:
            app_idx = parts.index("Contents") - 1
            return os.sep.join(parts[:app_idx+1])
        return exe_path

    def _download_sync(self, url: str, target: str, cb: Callable[[float], None]) -> bool:
        sys_os = platform.system()
        is_mac = (sys_os == "Darwin")
        
        if is_mac:
            target_app = self._get_mac_app_path(target)
            old_target = target_app + ".old"
            new_target = target_app + ".new_download.zip"
            extract_dir = target_app + "_extracted"
        else:
            old_target = target + ".old"
            new_target = target + ".new"
            extract_dir = ""

        try:
            for temp_file in [new_target, old_target if not is_mac else old_target, extract_dir]:
                if temp_file and os.path.exists(temp_file):
                    if os.path.isdir(temp_file):
                        shutil.rmtree(temp_file, ignore_errors=True)
                    else:
                        try: os.remove(temp_file)
                        except OSError: pass

            req = urllib.request.Request(url, headers={"User-Agent": "CodeContextAI-App"})
            app_logger.info(f"[Updater] Скачивание во временный файл: {new_target}")

            with urllib.request.urlopen(req, timeout=30) as response, open(new_target, 'wb') as out_file:
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
            
            app_logger.info("[Updater] Файл скачан. Выполняем безопасную подмену...")

            if is_mac:
                app_logger.info("[Updater] Распаковка .zip для macOS...")
                with zipfile.ZipFile(new_target, 'r') as zip_ref:
                    zip_ref.extractall(extract_dir)
                
                extracted_app = None
                for item in os.listdir(extract_dir):
                    if item.endswith(".app"):
                        extracted_app = os.path.join(extract_dir, item)
                        break
                        
                if not extracted_app:
                    raise RuntimeError(tr("updater_service.download.no_app_in_archive"))
                    
                if os.path.exists(target_app):
                    app_logger.info(f"[Updater] Перемещение старого .app: {target_app} -> {old_target}")
                    os.rename(target_app, old_target)
                    
                app_logger.info(f"[Updater] Установка новой версии: {extracted_app} -> {target_app}")
                shutil.move(extracted_app, target_app)
                
                binary_name = os.path.basename(target)
                new_binary_path = os.path.join(target_app, "Contents", "MacOS", binary_name)
                if os.path.exists(new_binary_path):
                    os.chmod(new_binary_path, 0o755)
                    
                os.remove(new_target)
                shutil.rmtree(extract_dir, ignore_errors=True)
                
            else:
                if os.path.exists(target):
                    app_logger.info(f"[Updater] Перемещение запущенного файла: {target} -> {old_target}")
                    os.replace(target, old_target)
                    
                app_logger.info(f"[Updater] Установка новой версии: {new_target} -> {target}")
                os.replace(new_target, target)
                
                if sys_os == "Linux":
                    app_logger.info("[Updater] Установка прав 755 для Linux")
                    os.chmod(target, 0o755)

            app_logger.info("[Updater] Установка успешно завершена.")
            return True

        except Exception as e:
            app_logger.error(f"[Updater] Ошибка при скачивании/установке: {e}")
            
            if is_mac:
                if os.path.exists(new_target):
                    try: os.remove(new_target)
                    except OSError: pass
                if os.path.exists(extract_dir):
                    shutil.rmtree(extract_dir, ignore_errors=True)
                if not os.path.exists(target_app) and os.path.exists(old_target):
                    app_logger.info("[Updater] Откат: восстановление старого .app")
                    os.rename(old_target, target_app)
            else:
                if os.path.exists(new_target):
                    try: os.remove(new_target)
                    except OSError: pass
                if not os.path.exists(target) and os.path.exists(old_target):
                    app_logger.info("[Updater] Откат: восстановление старого исполняемого файла.")
                    try: os.replace(old_target, target)
                    except OSError as rollback_err:
                        app_logger.error(f"Критическая ошибка отката: {rollback_err}")
            
            raise RuntimeError(tr("updater_service.download.download_failed", error=e))

    def _parse_version(self, v_str: str) -> Tuple[int, int, int, int, int]:
        match = re.match(r'v?(\d+)\.(\d+)\.(\d+)(?:-pre\.(\d+))?', v_str)
        if not match:
            return (0, 0, 0, 0, 0)
        maj, min, pat, pre = match.groups()
        if pre:
            return (int(maj), int(min), int(pat), -1, int(pre))
        return (int(maj), int(min), int(pat), 0, 0)
