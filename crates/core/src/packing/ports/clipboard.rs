use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClipboardPortError {
    #[error("Не удалось записать текст в системный буфер обмена: {0}")]
    WriteFailed(String),
    #[error("Не удалось прочитать текст из системного буфер обмена: {0}")]
    ReadFailed(String),
}

pub trait ClipboardPort: Send + Sync {
    fn copy_text(&self, text: &str) -> Result<(), ClipboardPortError>;

    fn read_text(&self) -> Result<String, ClipboardPortError> {
        Ok(String::new())
    }
}
