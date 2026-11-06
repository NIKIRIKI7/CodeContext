# Декларативный UI-протокол (`PluginViewSpec`)

Для рендеринга пользовательских вкладок и панелей на частоте **120 FPS** без внедрения WebView плагины возвращают структуру `PluginViewSpec`, состоящую из узлов `PluginWidgetNode`.

---

## 1. Поддерживаемые виджеты `PluginWidgetNode`

* **`section`** — карточка с заголовком и дочерними элементами (`title`, `children`).
* **`row` / `column`** — горизонтальный или вертикальный Flexbox-контейнер (`children`).
* **`label`** — текстовая строка (`text`, `monospace: bool`, `muted: bool`).
* **`badge`** — статусный бейдж (`text`, `tone: "neutral" | "accent" | "success" | "warning" | "danger"`).
* **`button`** — интерактивная кнопка (`id`, `label`, `primary: bool`), генерирующая событие `PluginUiEvent`.
* **`switch`** — переключатель (`id`, `label`, `checked: bool`).
* **`code_preview`** — виртуализированное поле просмотра кода или логов (`id`, `content`).