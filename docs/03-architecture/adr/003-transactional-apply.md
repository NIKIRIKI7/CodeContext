# ADR-003: Транзакционное применение патчей (Staging + Backup Manifest)

* **Статус:** Принято
* **Контекст:** Безопасность данных пользователя при применении ответа ИИ в `crates/apply`

## Решение
Реализовать применение `ApplyPlan` как атомарную бизнес-транзакцию (**Unit of Work** в `crates/apply/src/storage/transaction.rs`):
1. **Pre-flight Validation:** Все пути проверяются через `sandbox.rs` ДО любых операций с диском.
2. **Backup Snapshot & Manifest:** Создание резервной копии в `.ctxpack/backups/<timestamp>/` с `manifest.json`.
3. **Staging Write + Atomic Rename:** Запись во временный файл с `File::sync_all()` и атомарное переименование `fs::rename`.