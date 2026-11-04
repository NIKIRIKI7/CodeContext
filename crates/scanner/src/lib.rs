//! Инфраструктурный адаптер высокоскоростного обхода ФС и сборки буфера.

mod content;
mod discovery;

pub use discovery::{
    CompiledFilterPlan, FolderWalkVerdict, IgnoreFileScanner, ScannedFile, ScannerError,
    is_valid_regex_pattern,
};
