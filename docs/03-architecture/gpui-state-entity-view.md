# Архитектура UI на GPUI: Паттерн `State-Entity-View`

Для защиты проекта от изменений API в молодом фреймворке `gpui` и обеспечения 120 FPS при работе с проектами из 50 000 файлов в `crates/ui` действуют строгие конвенции.

---

## 1. Структура каждой фичи в `crates/ui/src/features/`

Каждая экранная фича (`main_window`, `context_packer`, `change_applier`, `settings_modal`) представляет собой изолированный модуль и обязана содержать три файла:

```text
features/<feature_name>/
├── mod.rs          # Экспорт публичного View и State фичи
├── state.rs        # Чистое состояние экрана и бизнес-команды (БЕЗ вёрстки!)
├── view.rs         # Корневой компонент GPUI (держит Entity<State> и подписывается на события)
└── components/     # Глупые (Stateless) визуальные подкомпоненты (не более 4 файлов)
```

### Жёсткие инварианты (проверяются `test_6` в `architecture.rs`):
1. **`state.rs` не знает про вёрстку:** В `state.rs` категорически запрещено реализовывать трейты `Render` или `RenderOnce`.
2. **`view.rs` — единственный владелец `Entity<State>`:** Только `view.rs` читает состояние из `Entity<...State>`.
3. **Инкапсуляция `components/`:** Внутренним компонентам в папке `components/` **запрещено принимать или хранить `Entity<...>`**.

---

## 2. Виртуализация данных (CQRS Read Projections для UI)

### 2.1. Плоское дерево файлов (`components/file_tree.rs`)
* В `context_packer::state` дерево проектируется в плоский вектор `Vec<FlatTreeNode>` с полями `depth: u8`, `is_dir: bool`, `is_expanded: bool`, `check_state: TriState`.
* Виджет `file_tree.rs` использует виртуальный список GPUI (`uniform_list`).

### 2.2. Виртуальный просмотрщик мультимегабайтного текста (`shared/containers/virtual_text.rs`)
* При генерации предпросмотра один раз вычисляется вектор байтовых смещений начала строк: `line_offsets: Vec<u32>` поверх буфера `Arc<str>`.
* Компонент `virtual_text.rs` рендерит через `uniform_list` только видимое окно строк за $O(1)$ с нулём аллокаций памяти на кадр.