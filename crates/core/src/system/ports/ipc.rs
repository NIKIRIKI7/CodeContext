use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const IPC_PROTOCOL_MAGIC: [u8; 2] = *b"CC";
pub const IPC_PROTOCOL_VERSION: u8 = 1;
pub const IPC_HEADER_LEN: usize = 6;
pub const MAX_IPC_FRAME_BYTES: usize = 4096;
pub const MAX_IPC_PAYLOAD_BYTES: usize = MAX_IPC_FRAME_BYTES - IPC_HEADER_LEN;

const OPCODE_OPEN_PROJECT: u8 = 1;
const OPCODE_FOCUS_WINDOW: u8 = 2;
const OPCODE_PING: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcCommand {
    OpenProjectAndFocus { project_root: PathBuf },
    FocusWindow,
    Ping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcCommandRef<'a> {
    OpenProjectAndFocus { project_root: &'a Path },
    FocusWindow,
    Ping,
}

impl IpcCommandRef<'_> {
    #[inline]
    pub fn into_owned(self) -> IpcCommand {
        match self {
            Self::OpenProjectAndFocus { project_root } => {
                IpcCommand::OpenProjectAndFocus { project_root: project_root.to_path_buf() }
            }
            Self::FocusWindow => IpcCommand::FocusWindow,
            Self::Ping => IpcCommand::Ping,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemActivationEvent {
    OpenProjectAndFocus { project_root: PathBuf },
    ToggleOrFocusWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimaryElectionResult {
    AcquiredPrimary { stale_socket_recovered: bool },
    ForwardedToPrimary,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IpcPortError {
    #[error("Ошибка канала межпроцессного взаимодействия: {0}")]
    Transport(String),

    #[error("Некорректный заголовок или формат IPC-кадра: {0}")]
    InvalidFrame(&'static str),

    #[error("Размер IPC-пакета ({size} байт) превышает лимит безопасности ({max} байт)")]
    PayloadTooLarge { size: usize, max: usize },
}

pub fn encode_ipc_frame(
    command: &IpcCommand,
    out_buf: &mut [u8; MAX_IPC_FRAME_BYTES],
) -> Result<usize, IpcPortError> {
    let (opcode, payload_bytes): (u8, &[u8]) = match command {
        IpcCommand::OpenProjectAndFocus { project_root } => {
            let path_str = project_root
                .to_str()
                .ok_or(IpcPortError::InvalidFrame("Путь не является UTF-8"))?;
            if path_str.is_empty() || path_str.contains('\0') {
                return Err(IpcPortError::InvalidFrame("Пустой путь или запрещённый NUL-байт"));
            }
            (OPCODE_OPEN_PROJECT, path_str.as_bytes())
        }
        IpcCommand::FocusWindow => (OPCODE_FOCUS_WINDOW, &[]),
        IpcCommand::Ping => (OPCODE_PING, &[]),
    };

    let payload_len = payload_bytes.len();
    if payload_len > MAX_IPC_PAYLOAD_BYTES {
        return Err(IpcPortError::PayloadTooLarge {
            size: payload_len + IPC_HEADER_LEN,
            max: MAX_IPC_FRAME_BYTES,
        });
    }

    let len_u16 = u16::try_from(payload_len).unwrap_or(u16::MAX);
    let [len_hi, len_lo] = len_u16.to_be_bytes();
    let [magic_0, magic_1] = IPC_PROTOCOL_MAGIC;

    if let Some(header) = out_buf.get_mut(..IPC_HEADER_LEN) {
        header.copy_from_slice(&[magic_0, magic_1, IPC_PROTOCOL_VERSION, opcode, len_hi, len_lo]);
    }

    let total_len = IPC_HEADER_LEN + payload_len;
    if let Some(body) = out_buf.get_mut(IPC_HEADER_LEN..total_len) {
        body.copy_from_slice(payload_bytes);
    }

    Ok(total_len)
}

#[inline]
pub fn decode_ipc_frame_ref(frame: &[u8]) -> Result<IpcCommandRef<'_>, IpcPortError> {
    if frame.len() < IPC_HEADER_LEN {
        return Err(IpcPortError::InvalidFrame("Кадр короче 6-байтового заголовка"));
    }
    if frame.len() > MAX_IPC_FRAME_BYTES {
        return Err(IpcPortError::PayloadTooLarge { size: frame.len(), max: MAX_IPC_FRAME_BYTES });
    }

    if frame.get(0..2) != Some(&IPC_PROTOCOL_MAGIC) {
        return Err(IpcPortError::InvalidFrame("Неверная сигнатура магических байтов"));
    }
    if frame.get(2).copied() != Some(IPC_PROTOCOL_VERSION) {
        return Err(IpcPortError::InvalidFrame("Неподдерживаемая версия IPC-протокола"));
    }

    let opcode = frame.get(3).copied().unwrap_or(0);
    let len_hi = frame.get(4).copied().unwrap_or(0);
    let len_lo = frame.get(5).copied().unwrap_or(0);
    let payload_len = u16::from_be_bytes([len_hi, len_lo]) as usize;

    if payload_len > MAX_IPC_PAYLOAD_BYTES {
        return Err(IpcPortError::PayloadTooLarge {
            size: payload_len + IPC_HEADER_LEN,
            max: MAX_IPC_FRAME_BYTES,
        });
    }

    let expected_end = IPC_HEADER_LEN + payload_len;
    let Some(payload) = frame.get(IPC_HEADER_LEN..expected_end) else {
        return Err(IpcPortError::InvalidFrame("Обрезанное тело IPC-кадра"));
    };

    match opcode {
        OPCODE_OPEN_PROJECT => {
            if payload.is_empty() || payload.contains(&0) {
                return Err(IpcPortError::InvalidFrame("Пустой или содержащий NUL путь проекта"));
            }
            let path_str = std::str::from_utf8(payload)
                .map_err(|_| IpcPortError::InvalidFrame("Повреждённый UTF-8 в пути проекта"))?;
            Ok(IpcCommandRef::OpenProjectAndFocus { project_root: Path::new(path_str) })
        }
        OPCODE_FOCUS_WINDOW => {
            if !payload.is_empty() {
                return Err(IpcPortError::InvalidFrame("FocusWindow не должен содержать тело"));
            }
            Ok(IpcCommandRef::FocusWindow)
        }
        OPCODE_PING => {
            if !payload.is_empty() {
                return Err(IpcPortError::InvalidFrame("Ping не должен содержать тело"));
            }
            Ok(IpcCommandRef::Ping)
        }
        _ => Err(IpcPortError::InvalidFrame("Неизвестный код операции IPC")),
    }
}

#[inline]
pub fn decode_ipc_frame(frame: &[u8]) -> Result<IpcCommand, IpcPortError> {
    decode_ipc_frame_ref(frame).map(IpcCommandRef::into_owned)
}

pub trait SingleInstanceIpcPort: Send + Sync {
    fn try_send_project_path_to_existing_instance(
        &self,
        project_root: PathBuf,
    ) -> Result<bool, IpcPortError>;

    fn ensure_primary_or_forward(
        &self,
        initial_project_root: Option<PathBuf>,
    ) -> Result<PrimaryElectionResult, IpcPortError> {
        if let Some(path) = initial_project_root
            && self.try_send_project_path_to_existing_instance(path)?
        {
            return Ok(PrimaryElectionResult::ForwardedToPrimary);
        }
        Ok(PrimaryElectionResult::AcquiredPrimary { stale_socket_recovered: false })
    }

    fn try_recv_command(&self) -> Option<IpcCommand> {
        None
    }
}
