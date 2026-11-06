use serde::{Deserialize, Serialize};

pub const DEFAULT_FILE_HEADER_PREFIX: &str = "=== FILE: ";
pub const DEFAULT_FILE_HEADER_SUFFIX: &str = " ===\n";
pub const DEFAULT_FILE_FOOTER: &str = "\n=== END FILE ===\n\n";

pub const DEFAULT_SEARCH_MARKER: &str = "<<<<<<< SEARCH";
pub const DEFAULT_DIVIDER_MARKER: &str = "=======";
pub const DEFAULT_REPLACE_MARKER: &str = ">>>>>>> REPLACE";

pub const PROMPT_SEARCH_REPLACE: &str = concat!(
    "ИНСТРУКЦИЯ ДЛЯ ГЕНЕРАЦИИ КОДА И ПАТЧЕЙ (ДЛЯ CODECONTEXT)\n",
    "Ты работаешь в связке с локальным парсером CodeContext. Все изменения файлов, добавление новых функций, создание модулей и удаление файлов ты ОБЯЗАН оформлять строго по приведённым ниже правилам.\n",
    "————\n",
    "1. ОБЩИЕ ПРАВИЛА\n",
    "1. Всегда оборачивай весь блок изменений в один общий блок: ```text ... ```.\n",
    "2. Не пиши пояснительный текст внутри блока text — только файлы и директивы парсера. Пояснения пиши до или после блока.\n",
    "3. Все пути файлов указывай строго относительно корня проекта, используя прямой слэш / (без ./, без абсолютных путей C:\\ или /home/).\n",
    "4. Для любых точечных правок и добавления кода ВСЕГДА используй SEARCH / REPLACE. Полную перезапись файла применяй только если файл меняется более чем на 70%.\n",
    "————\n",
    "2. СИНТАКСИС ОПЕРАЦИЙ\n",
    "А. Изменение существующего кода (SEARCH / REPLACE)\n",
    "=== FILE: crates/core/src/service.rs ===\n",
    "<<<<<<< SEARCH\n",
    "<точные строки из исходного файла с 2–3 строками неизменного контекста>\n",
    "=======\n",
    "<новый код, который встанет на место блока SEARCH>\n",
    ">>>>>>> REPLACE\n",
    "=== END FILE ===\n",
    "Б. ВСТАВКА НОВОГО КОДА ЧЕРЕЗ ЯКОРЬ (АНКОРИНГ)\n",
    "Если нужно добавить новую функцию, метод или импорт, выбери в файле уникальный код-якорь (соседнюю функцию или импорт), укажи его в SEARCH, а в REPLACE повтори код-якорь без изменений и добавь новую функцию до или после него.\n",
    "В. Создание нового файла\n",
    "=== CREATE FILE: crates/ui/src/new_module.rs ===\n",
    "<полное содержимое нового файла>\n",
    "=== END FILE ===\n",
    "Г. Полная перезапись существующего файла\n",
    "=== FILE: crates/ui/src/legacy.rs ===\n",
    "<полное новое содержимое файла>\n",
    "=== END FILE ===\n",
    "Д. Удаление файла\n",
    "=== DELETE FILE: crates/ui/src/obsolete.rs ===\n",
    "————\n",
    "3. ВАЖНЫЕ ТРЕБОВАНИЯ И ЗАПРЕТЫ\n",
    "1. Запрещён псевдокод: Категорически запрещено писать // ... остальной код ... или // TODO: keep existing code внутри блоков SEARCH или REPLACE.\n",
    "2. Точность якоря: Якорь в SEARCH должен быть уникальным в файле (захватывай имя функции, сигнатуру, ключевые слова).\n",
    "3. Несколько правок в одном файле: Объединяй их внутри одного заголовка === FILE: ... === ... === END FILE ===, создавая несколько последовательных блоков SEARCH / REPLACE.\n\n"
);

pub const PROMPT_CANONICAL_FULL: &str = concat!(
    "ИНСТРУКЦИЯ ДЛЯ ГЕНЕРАЦИИ ПОЛНЫХ ФАЙЛОВ (CODECONTEXT)\n",
    "Оформляй все изменённые и новые файлы строго в одном блоке ```text ... ``` в каноническом формате:\n",
    "=== FILE: path/to/file.rs ===\n",
    "<полное содержимое файла целиком без пропусков и сокращений>\n",
    "=== END FILE ===\n",
    "Для удаления файла используй директиву: === DELETE FILE: path/to/file.rs ===\n",
    "Категорически запрещено сокращать код комментариями вида `// ... existing code ...`.\n\n"
);

pub const PROMPT_UNIFIED_DIFF: &str = concat!(
    "ИНСТРУКЦИЯ ДЛЯ ГЕНЕРАЦИИ ПАТЧЕЙ UNIFIED DIFF (CODECONTEXT)\n",
    "Оформляй все изменения в формате стандартного Unified Diff внутри блока ```diff ... ```:\n",
    "--- a/path/to/file.rs\n",
    "+++ b/path/to/file.rs\n",
    "@@ -start,count +start,count @@\n",
    " контекстная строка\n",
    "-удаляемая строка\n",
    "+добавляемая строка\n",
    "Указывай точные относительные пути файлов от корня проекта и минимум 3 строки неизменного контекста.\n\n"
);

pub const PROMPT_ARCH_STRICT: &str = concat!(
    "АРХИТЕКТУРНЫЕ ТРЕБОВАНИЯ И КАЧЕСТВО КОДА\n",
    "- Строго соблюдай существующую архитектуру проекта, границы модулей и соглашения об именовании.\n",
    "- Не используй `.unwrap()`, `.expect()`, `panic!()`, `todo!()` или небезопасные блоки `unsafe` в рабочем коде.\n",
    "- Пиши чистый, детерминированный и производительный код без лишних аллокаций памяти.\n\n"
);

pub const PROMPT_BEHAVIOR_CONCISE: &str = concat!(
    "ФОРМАТ ОТВЕТА И ПОВЕДЕНИЕ\n",
    "- Отвечай сразу по делу без длинных вступлений, приветствий и пересказа условия задачи.\n",
    "- Перед выдачей патча убедись, что все импорты, типы и вызовы функций согласованы и код компилируется без предупреждений.\n\n"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptCategory {
    #[default]
    Patcher,
    Architecture,
    Behavior,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptBlockItem {
    pub id: String,
    pub title: String,
    pub category: PromptCategory,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_category: Option<String>,
    pub content: String,
    pub enabled: bool,
}

pub fn default_prompt_blocks() -> Vec<PromptBlockItem> {
    vec![
        PromptBlockItem {
            id: "tech_patcher_full".to_owned(),
            title: "Технический промпт патчера CodeContext (SEARCH/REPLACE + Анкоринг)".to_owned(),
            category: PromptCategory::Patcher,
            custom_category: None,
            content: PROMPT_SEARCH_REPLACE.trim_end().to_owned(),
            enabled: false,
        },
        PromptBlockItem {
            id: "tech_canonical_full".to_owned(),
            title: "Полная перезапись файлов (Canonical Full Files)".to_owned(),
            category: PromptCategory::Patcher,
            custom_category: None,
            content: PROMPT_CANONICAL_FULL.trim_end().to_owned(),
            enabled: false,
        },
        PromptBlockItem {
            id: "tech_unified_diff".to_owned(),
            title: "Патчи в формате Git Unified Diff".to_owned(),
            category: PromptCategory::Patcher,
            custom_category: None,
            content: PROMPT_UNIFIED_DIFF.trim_end().to_owned(),
            enabled: false,
        },
        PromptBlockItem {
            id: "arch_strict_quality".to_owned(),
            title: "Строгий контроль архитектуры (без unwrap / panic / unsafe)".to_owned(),
            category: PromptCategory::Architecture,
            custom_category: None,
            content: PROMPT_ARCH_STRICT.trim_end().to_owned(),
            enabled: false,
        },
        PromptBlockItem {
            id: "behavior_concise".to_owned(),
            title: "Краткий инженерный ответ без воды и преамбулы".to_owned(),
            category: PromptCategory::Behavior,
            custom_category: None,
            content: PROMPT_BEHAVIOR_CONCISE.trim_end().to_owned(),
            enabled: false,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemPromptMode {
    #[default]
    None,
    SearchReplacePatch,
    CanonicalFullFiles,
    UnifiedDiffPatch,
    Custom,
}

impl SystemPromptMode {
    #[inline]
    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::None)
    }

    #[must_use]
    #[inline]
    pub const fn next_in_cycle(self) -> Self {
        match self {
            Self::None => Self::SearchReplacePatch,
            Self::SearchReplacePatch => Self::CanonicalFullFiles,
            Self::CanonicalFullFiles => Self::UnifiedDiffPatch,
            Self::UnifiedDiffPatch => Self::Custom,
            Self::Custom => Self::None,
        }
    }

    #[inline]
    pub const fn built_in_prompt(self) -> &'static str {
        match self {
            Self::None | Self::Custom => "",
            Self::SearchReplacePatch => PROMPT_SEARCH_REPLACE,
            Self::CanonicalFullFiles => PROMPT_CANONICAL_FULL,
            Self::UnifiedDiffPatch => PROMPT_UNIFIED_DIFF,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchReplaceMarkers {
    pub search_header: String,
    pub divider: String,
    pub replace_footer: String,
}

impl Default for SearchReplaceMarkers {
    fn default() -> Self {
        Self {
            search_header: DEFAULT_SEARCH_MARKER.to_owned(),
            divider: DEFAULT_DIVIDER_MARKER.to_owned(),
            replace_footer: DEFAULT_REPLACE_MARKER.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserConfig {
    pub sync_with_output_template: bool,
    pub enable_markdown_fences: bool,
    pub enable_search_replace: bool,
    pub enable_unified_diff: bool,
    pub strip_project_root_prefix: bool,
    pub comment_path_prefixes: Vec<String>,
    pub search_replace_markers: SearchReplaceMarkers,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::canonical()
    }
}

impl ParserConfig {
    #[inline]
    pub fn canonical() -> Self {
        Self {
            sync_with_output_template: true,
            enable_markdown_fences: true,
            enable_search_replace: true,
            enable_unified_diff: true,
            strip_project_root_prefix: true,
            comment_path_prefixes: vec![
                "filepath:".to_owned(),
                "file:".to_owned(),
                "path:".to_owned(),
                "файл:".to_owned(),
            ],
            search_replace_markers: SearchReplaceMarkers::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputTemplate {
    pub include_file_tree: bool,
    #[serde(default)]
    pub system_prompt_preset: SystemPromptMode,
    #[serde(default = "default_prompt_blocks")]
    pub prompt_blocks: Vec<PromptBlockItem>,
    pub preamble: String,
    pub file_header_prefix: String,
    pub file_header_suffix: String,
    pub file_footer: String,
    pub postamble: String,
    pub max_chunk_bytes: Option<usize>,
}

impl Default for OutputTemplate {
    fn default() -> Self {
        Self::canonical()
    }
}

impl OutputTemplate {
    #[inline]
    pub fn canonical() -> Self {
        Self {
            include_file_tree: false,
            system_prompt_preset: SystemPromptMode::None,
            prompt_blocks: default_prompt_blocks(),
            preamble: String::new(),
            file_header_prefix: DEFAULT_FILE_HEADER_PREFIX.to_owned(),
            file_header_suffix: DEFAULT_FILE_HEADER_SUFFIX.to_owned(),
            file_footer: DEFAULT_FILE_FOOTER.to_owned(),
            postamble: String::new(),
            max_chunk_bytes: None,
        }
    }

    #[inline]
    pub fn active_prompt_blocks_count(&self) -> usize {
        let blocks_active = self.prompt_blocks.iter().filter(|b| b.enabled).count();
        if blocks_active > 0 {
            blocks_active
        } else {
            usize::from(self.system_prompt_preset.is_enabled() || !self.preamble.trim().is_empty())
        }
    }

    pub fn sync_preamble_from_blocks(&mut self) {
        let mut combined = String::new();
        for block in self.prompt_blocks.iter().filter(|b| b.enabled) {
            let trimmed = block.content.trim();
            if !trimmed.is_empty() {
                if !combined.is_empty() {
                    combined.push_str("\n\n");
                }
                combined.push_str(trimmed);
            }
        }
        if combined.is_empty() {
            self.preamble.clear();
            self.system_prompt_preset = SystemPromptMode::None;
        } else {
            combined.push_str("\n\n");
            self.preamble = combined;
            self.system_prompt_preset = SystemPromptMode::Custom;
        }
    }

    /// Возвращает итоговый текст преамбулы / системного промпта с учётом выбранного режима.
    #[inline]
    pub fn effective_preamble(&self) -> &str {
        match self.system_prompt_preset {
            SystemPromptMode::None => self.preamble.as_str(),
            SystemPromptMode::Custom => {
                if self.preamble.is_empty() {
                    PROMPT_SEARCH_REPLACE
                } else {
                    self.preamble.as_str()
                }
            }
            SystemPromptMode::SearchReplacePatch
            | SystemPromptMode::CanonicalFullFiles
            | SystemPromptMode::UnifiedDiffPatch => {
                if self.preamble.is_empty() {
                    self.system_prompt_preset.built_in_prompt()
                } else {
                    self.preamble.as_str()
                }
            }
        }
    }

    #[must_use]
    #[inline]
    pub fn with_file_tree(mut self, include_file_tree: bool) -> Self {
        self.include_file_tree = include_file_tree;
        self
    }

    #[must_use]
    #[inline]
    pub fn with_max_chunk_bytes(mut self, max_chunk_bytes: Option<usize>) -> Self {
        self.max_chunk_bytes = max_chunk_bytes;
        self
    }

    #[inline]
    pub fn fixed_marker_bytes_per_file(&self) -> usize {
        self.file_header_prefix.len() + self.file_header_suffix.len() + self.file_footer.len()
    }

    #[inline]
    pub fn header_suffix_trimmed(&self) -> &str {
        self.file_header_suffix.trim_end_matches(['\r', '\n'])
    }

    #[inline]
    pub fn footer_trimmed(&self) -> &str {
        self.file_footer.trim_matches(['\r', '\n'])
    }
}
