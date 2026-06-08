import logging
import sys
import os
from logging.handlers import RotatingFileHandler


def setup_logger(name: str = "CodeContext") -> logging.Logger:
    """Настройка логгера для приложения (Консоль + Файл)"""
    logger = logging.getLogger(name)

    # Если логгер уже настроен, не добавляем хендлеры повторно
    if logger.hasHandlers():
        return logger

    # Предотвращаем краш при записи emoji в cp1251-терминал
    try:
        sys.stdout.reconfigure(errors='replace')
        sys.stderr.reconfigure(errors='replace')
    except Exception:
        pass

    logger.setLevel(logging.DEBUG)  # Перехватываем всё, фильтруем в хендлерах

    # Форматы логов
    file_formatter = logging.Formatter('%(asctime)s | %(levelname)-8s | [%(filename)s:%(lineno)d] | %(message)s')
    console_formatter = logging.Formatter('%(asctime)s - %(levelname)s - %(message)s')

    # 1. Вывод в консоль (INFO и выше)
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(logging.INFO)
    console_handler.setFormatter(console_formatter)
    logger.addHandler(console_handler)

    # 2. Вывод в файл (DEBUG и выше - логируем всё)
    from .config import get_app_data_dir
    logs_dir = os.path.join(get_app_data_dir(), "logs")

    try:
        os.makedirs(logs_dir, exist_ok=True)
        log_file = os.path.join(logs_dir, "app.log")

        # Ротация логов: макс 10 МБ на файл, храним 5 последних копий
        file_handler = RotatingFileHandler(log_file, maxBytes=10 * 1024 * 1024, backupCount=5, encoding='utf-8')
        file_handler.setLevel(logging.DEBUG)
        file_handler.setFormatter(file_formatter)
        logger.addHandler(file_handler)

        logger.info(f"📁 Файловое логирование инициализировано: {log_file}")
    except Exception as e:
        logger.error(f"❌ Ошибка создания папки логов: {e}")

    return logger


# Глобальный инстанс логгера для всего приложения
app_logger = setup_logger()