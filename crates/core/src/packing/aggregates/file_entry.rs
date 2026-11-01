//! Сущность узла файла в дереве проекта (`FileEntry`).

use serde::{Deserialize, Serialize};

use crate::shared_kernel::fs::file_kind::FileKind;

/// Метаданные и состояние выбора одного файла в индексе проекта (не хранит текст файла в ОЗУ).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Нормализованный относительный путь от корня проекта (всегда через `/`).
    pub rel_path: String,
    /// Размер файла на диске в байтах.
    pub size_bytes: u64,
    /// Классификация файла.
    pub kind: FileKind,
    /// Выбран ли файл чекбоксом для включения в контекст.
    pub is_selected: bool,
    /// Причина блокировки (если файл заблокирован как секрет или бинарник).
    pub lock_reason: Option<String>,
}

impl FileEntry {
    /// Создаёт новый узел файла с выбором по умолчанию в зависимости от его типа.
    #[inline]
    pub const fn new(rel_path: String, size_bytes: u64, kind: FileKind) -> Self {
        Self {
            rel_path,
            size_bytes,
            kind,
            is_selected: kind.is_packable_by_default(),
            lock_reason: None,
        }
    }

    /// Создаёт заблокированный узел файла секрета.
    #[inline]
    pub const fn new_blocked_secret(rel_path: String, size_bytes: u64, reason: String) -> Self {
        Self {
            rel_path,
            size_bytes,
            kind: FileKind::Secret,
            is_selected: false,
            lock_reason: Some(reason),
        }
    }

    /// Проверяет, может ли файл быть выбран пользователем.
    #[inline]
    pub const fn can_select(&self, is_secret_explicitly_allowed: bool) -> bool {
        match self.kind {
            FileKind::SourceCode | FileKind::Configuration | FileKind::Documentation => true,
            FileKind::Secret => is_secret_explicitly_allowed,
            FileKind::Binary | FileKind::SymlinkSkipped => false,
        }
    }

    /// Изменяет состояние выбора файла с соблюдением инвариантов безопасности.
    /// Возвращает `true`, если состояние `is_selected` реально изменилось.
    #[inline]
    pub fn set_selected(&mut self, selected: bool, is_secret_explicitly_allowed: bool) -> bool {
        let target = if selected { self.can_select(is_secret_explicitly_allowed) } else { false };
        if self.is_selected == target {
            return false;
        }
        self.is_selected = target;
        true
    }
}
