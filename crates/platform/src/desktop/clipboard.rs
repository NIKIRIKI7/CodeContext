use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use core::packing::ports::clipboard::{ClipboardPort, ClipboardPortError};

#[derive(Clone)]
pub struct SystemClipboard {
    fallback_buffer: Arc<Mutex<Option<String>>>,
    revision: Arc<AtomicU64>,
}
impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Debug for SystemClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemClipboard").finish_non_exhaustive()
    }
}
impl SystemClipboard {
    #[inline]
    pub fn new() -> Self {
        Self { fallback_buffer: Arc::new(Mutex::new(None)), revision: Arc::new(AtomicU64::new(1)) }
    }
    #[inline]
    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }
    pub fn read_text(&self) -> Result<String, ClipboardPortError> {
        if let Ok(mut cb) = arboard::Clipboard::new()
            && let Ok(text) = cb.get_text()
            && !text.is_empty()
        {
            return Ok(text);
        }
        let guard = self
            .fallback_buffer
            .lock()
            .map_err(|err| ClipboardPortError::ReadFailed(err.to_string()))?;
        Ok(guard.clone().unwrap_or_default())
    }
}
impl ClipboardPort for SystemClipboard {
    fn copy_text(&self, text: &str) -> Result<(), ClipboardPortError> {
        if let Ok(mut guard) = self.fallback_buffer.lock() {
            *guard = Some(text.to_owned());
        }
        self.revision.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut cb) = arboard::Clipboard::new() {
            cb.set_text(text.to_owned())
                .map_err(|err| ClipboardPortError::WriteFailed(err.to_string()))?;
        }
        Ok(())
    }
    fn read_text(&self) -> Result<String, ClipboardPortError> {
        Self::read_text(self)
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryClipboard {
    buffer: Arc<Mutex<String>>,
    revision: Arc<AtomicU64>,
}

impl Default for InMemoryClipboard {
    fn default() -> Self {
        Self { buffer: Arc::new(Mutex::new(String::new())), revision: Arc::new(AtomicU64::new(1)) }
    }
}

impl InMemoryClipboard {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_text(&self) -> String {
        self.buffer.lock().map(|g| g.clone()).unwrap_or_default()
    }

    #[inline]
    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }
}

impl ClipboardPort for InMemoryClipboard {
    fn copy_text(&self, text: &str) -> Result<(), ClipboardPortError> {
        let mut guard =
            self.buffer.lock().map_err(|err| ClipboardPortError::WriteFailed(err.to_string()))?;
        text.clone_into(&mut guard);
        self.revision.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn read_text(&self) -> Result<String, ClipboardPortError> {
        Ok(self.last_text())
    }
}
