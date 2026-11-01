//! Инфраструктурный адаптер высокоскоростного обхода ФС и сборки буфера.

mod content;
mod discovery;

pub use discovery::{IgnoreFileScanner, ScannedFile};
