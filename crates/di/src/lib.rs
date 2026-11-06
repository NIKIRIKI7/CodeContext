//! Контейнер внедрения зависимостей (Composition Root).

pub mod dependency_injector;

pub use apply::BackupSnapshotInfo;
pub use dependency_injector::{
    AppContainer, ClipboardAdapter, HousekeepingStats, ParsedResponseFile,
};
