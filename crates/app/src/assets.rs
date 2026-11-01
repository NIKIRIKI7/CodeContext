//! Встроенные ресурсы и метаданные приложения.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use ui::shared::primitives::icon::{list_icon_paths, load_icon_bytes};

/// Канонический заголовок главного окна приложения.
pub(crate) const APP_WINDOW_TITLE: &str = "CodeContext";

/// Статический источник векторных иконок Lucide (с поддержкой Dead Code Elimination).
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(load_icon_bytes(path))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(list_icon_paths())
    }
}
