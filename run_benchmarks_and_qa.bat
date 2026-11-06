@echo off
chcp 65001 >nul
setlocal EnableDelayedExpansion

echo ====================================================================
echo       CodeContext v0.2.5: Полный контроль качества и Бенчмарки
echo ====================================================================
echo.

echo [1/6] Форматирование кода (cargo fmt)...
cargo fmt --all -- --check
if errorlevel 1 (
    echo [ОШИБКА] Нарушено форматирование кода. Запустите: cargo fmt
    exit /b 1
)

echo [2/6] Статический анализ (cargo clippy -D warnings)...
cargo clippy --workspace --all-targets -- -D warnings
if errorlevel 1 (
    echo [ОШИБКА] Clippy обнаружил предупреждения или ошибки.
    exit /b 1
)

echo [3/6] Архитектурные фитнес-функции DDD и Ports-and-Adapters...
cargo test --package app --test architecture --release -- --nocapture
if errorlevel 1 (
    echo [ОШИБКА] Нарушены архитектурные границы DDD/GPUI.
    exit /b 1
)

echo [4/6] Проверка инварианта 1 аллокации кучи (dhat)...
cargo test --package scanner --test heap_allocations --release -- --test-threads=1 --nocapture
if errorlevel 1 (
    echo [ОШИБКА] Нарушен инвариант аллокаций кучи.
    exit /b 1
)

echo [5/6] Прогон модульных и интеграционных тестов воркспейса...
cargo test --workspace --release
if errorlevel 1 (
    echo [ОШИБКА] Один или несколько тестов завершились с ошибкой.
    exit /b 1
)

echo [6/6] Запуск бенчмарков пропускной способности (apply, scanner, ui)...
cargo bench --package apply --bench apply_throughput -- bench_8_parse_and_diff_212_files_1_7mb_against_disk
cargo bench --package apply --bench apply_throughput -- bench_9_rename_and_run_directives_plan
cargo bench --package scanner --bench pack_throughput -- bench_1c_real_pipeline_end_to_end_4mb
cargo bench --package ui --bench ui_projections -- bench_2_toggle_dir_checkbox_and_reproject_50k

echo.
echo ====================================================================
echo [УСПЕХ] Все проверки качества и бенчмарки успешно пройдены!
echo ====================================================================
endlocal