use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use core::packing::ports::clipboard::{ClipboardPort, ClipboardPortError};

#[derive(Clone, Default)]
pub struct SystemClipboard {
    os_clipboard: Arc<Mutex<Option<arboard::Clipboard>>>,
    fallback_buffer: Arc<Mutex<Option<String>>>,
}

impl std::fmt::Debug for SystemClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemClipboard").finish_non_exhaustive()
    }
}

impl SystemClipboard {
    #[inline]
    pub fn new() -> Self {
        Self {
            os_clipboard: Arc::new(Mutex::new(arboard::Clipboard::new().ok())),
            fallback_buffer: Arc::new(Mutex::new(None)),
        }
    }

    pub fn read_text(&self) -> Result<String, ClipboardPortError> {
        if let Ok(mut guard) = self.os_clipboard.lock() {
            if guard.is_none() {
                *guard = arboard::Clipboard::new().ok();
            }
            if let Some(ref mut cb) = *guard
                && let Ok(text) = cb.get_text()
            {
                return Ok(text);
            }
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
        if let Ok(mut guard) = self.os_clipboard.lock() {
            if guard.is_none() {
                *guard = arboard::Clipboard::new().ok();
            }
            if let Some(ref mut cb) = *guard {
                cb.set_text(text.to_owned())
                    .map_err(|err| ClipboardPortError::WriteFailed(err.to_string()))?;
            }
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
