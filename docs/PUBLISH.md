# Публикация CodeContext

## 1. PyPI (pip install codecontext-ai)

### Подготовка

PyPI уже настроен. Созданы файлы:

| Файл | Назначение |
|---|---|
| `pyproject.toml` | Метаданные, зависимости, entry point |
| `MANIFEST.in` | Включение data-файлов (темплейты, темы, ассеты) |
| `codecontext.py` | Точка входа для консольной команды `codecontext` |
| `src/__init__.py` | Превращает `src/` в Python-пакет |

### Сборка

```bash
# В корне проекта:
python -m pip install build twine
python -m build
```

Создадутся `dist/codecontext_ai-1.14.0-py3-none-any.whl` и `dist/codecontext-ai-1.14.0.tar.gz`.

### Публикация

1. Зарегистрироваться на https://pypi.org
2. Создать API-токен в https://pypi.org/manage/account/token/
3. Загрузить пакет:

```bash
python -m twine upload dist/*
# Ввести username: __token__
# Ввести password: <API-токен>
```

### Установка

```bash
pip install codecontext-ai
codecontext  # запуск GUI
codecontext --cli --path ./project  # CLI-режим
```

### Обновление версии

1. Обновить `version` в `pyproject.toml`
2. Обновить `VERSION.txt`
3. Пересобрать: `python -m build`
4. Загрузить: `python -m twine upload dist/*`

---

## 2. Официальный репозиторий Arch Linux (extra/community)

### Текущий статус

Пакет опубликован в AUR: https://aur.archlinux.org/packages/codecontext-ai

Для попадания в официальный репозиторий `extra` нужно:

### Критерии

- **Голоса**: обычно требуется ≥10 голосов на AUR
- **Популярность**: ≥1% использования по pkgstats
- **Лицензия**: совместимая (MIT — OK)
- **Качество**: PKGBUILD должен проходить `namcap` без ошибок
- **Лицензия сорцов**: 0BSD для PKGBUILD и вспомогательных файлов (уже добавлена `LICENSE`)

### Процесс

1. Набрать голоса на AUR (https://aur.archlinux.org/packages/codecontext-ai)
2. Найти Package Maintainer (Trusted User), который готов спонсировать пакет
3. Package Maintainer подаёт заявку в https://gitlab.archlinux.org/archlinux/packaging
4. Пакет проходит ревью и попадает в `extra-testing`, затем в `extra`

### Что потребуется от PKGBUILD для official

- Лицензия сорцов 0BSD — уже сделано
- `namcap PKGBUILD` без ошибок
- Чистый chroot-билд
- PGP-подпись для коммитов
- pkgrel увеличивается на 1 при переносе из AUR

### Если голосов пока мало

- Продолжать поддерживать AUR-пакет
- Обновлять по мере выхода релизов
- Отвечать на комментарии пользователей
- Со временем голосов станет больше

---

## 3. Установка из AUR (текущий способ)

```bash
yay -S codecontext-ai
# или вручную:
git clone https://aur.archlinux.org/codecontext-ai.git
cd codecontext-ai
makepkg -si
```

Ссылка: https://aur.archlinux.org/packages/codecontext-ai
