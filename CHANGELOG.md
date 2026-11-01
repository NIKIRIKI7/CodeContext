<div align="center">

**[🇷🇺 Русский](./CHANGELOG.md)** · **[🇬🇧 English](./CHANGELOG.en.md)** · **[📖 Документация (README)](./README.md)**

</div>

# История изменений (Changelog)

Все значимые изменения проекта **CodeContext** документируются в этом файле (синхронно с английской версией в [`CHANGELOG.en.md`](./CHANGELOG.en.md)).
Формат основан на [Keep a Changelog](https://keepachangelog.com/ru/1.1.0/), проект придерживается [Semantic Versioning](https://semver.org/lang/ru/).

> **Автоматическая генерация:** Для автоматического обновления обоих файлов (`CHANGELOG.md` и `CHANGELOG.en.md`) по истории коммитов Git используйте команду `just changelog <версия>` (или `just tag-release <версия>`).

---

## [0.1.0] - 2026-11-01

### Добавлено (Added)
- **Модуль A (`packing`)**: двухфазное параллельное сканирование (`ignore` + `rayon`) с учётом иерархических `.gitignore`, защитой `SecretGuard` (Сценарий A3), детекцией бинарных файлов по 8 КБ SIMD-пробе (`memchr`), языковой минификацией, разбивкой на тома (`Chunker`) и сборкой буфера с 0 реаллокаций (`dhat`).
- **Модуль B (`applying`)**: цепочка парсеров (`Canonical`, `Markdown`, `Search/Replace`, `Unified Diff`, `Create/Delete`), детерминированный `Autofix`, 6-этапный нечёткий матчер уровня Zed (коэффициент биграмм Сёренсена — Дайса + поддержка `...`), песочница путей `Sandbox` и транзакционное применение с откатом `Undo` (`.ctxpack/backups`).
- **Интеграция с ОС (`platform`)**: единое каскадное контекстное меню `CodeContext` из 3 действий (`Скопировать (с .gitignore)`, `Применить патч (автофикс)`, `Открыть в GUI`) для Windows Explorer (`HKCU`), macOS Finder (`Services`) и Linux (`Dolphin`, `Nemo`, `Nautilus`), глобальные хоткеи и Single-Instance IPC.
- **Графический интерфейс (`ui`)**: виртуализированное дерево на 50 000 файлов и просмотрщик многомегабайтного текста на `gpui` (120 FPS) с мгновенным переключением языков (`RU / EN`) и тем (`Dark / Light`).
- **Мультиплатформенный CI/CD и Автообновление**: сборка с PGO и Fat LTO под Windows (`x64`, `ARM64`), macOS (`Apple Silicon`, `Intel`, `Universal2`), Linux (`x86_64`, `ARM64` — `.deb`, `.rpm`, `.pkg.tar.zst`, `.AppImage`, `.tar.gz`, `.tar.xz`, `.tar.zst`) и проверка совместимости с FreeBSD и дистрибутивами Linux.