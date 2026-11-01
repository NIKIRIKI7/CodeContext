# Сборка максимальной производительности, Тестирование и PGO

Архитектура и конфигурация компилятора **CodeContext** настроены на **максимальную скорость в рантайме** (холодный старт < 10 мс, сквозной обход и упаковка 4 МБ кода с диска ~21 мс, рендеринг UI 120 FPS).

---

## 1. Команды сборки и запуска

```bash
# 1. Сборка максимально оптимизированного бинарника
cargo build --release

# 2. Запуск GUI-приложения на максимальной скорости (через алиас `cargo r`)
cargo run --release --package app

# 3. Запуск в режиме Headless (мгновенная упаковка папки в буфер обмена)
cargo run --release --package app -- --pack-headless .
```

---

## 2. Запуск тестов и бенчмарков

```bash
# 1. Прогон всех модульных и интеграционных тестов в релизном режиме
cargo nextest run --release

# 2. Архитектурный линтер DDD и Ports & Adapters
cargo test --package app --test architecture -- --nocapture

# 3. Проверка инварианта ровно 1 аллокации кучи при сборке 4 МБ (dhat)
cargo test --package scanner --test heap_allocations -- --test-threads=1 --nocapture

# 4. Property-based фаззинг песочницы и парсера на 1 000 входов (proptest)
cargo nextest run --release --package apply --test proptest_invariants

# 5. Замер пропускной способности в ГБ/с (divan)
cargo bench --package scanner
```