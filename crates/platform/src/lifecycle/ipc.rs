use std::collections::VecDeque;
use std::fs;
use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use core::system::ports::ipc::{
    IPC_HEADER_LEN, IPC_PROTOCOL_MAGIC, IPC_PROTOCOL_VERSION, IpcCommand, IpcPortError,
    MAX_IPC_FRAME_BYTES, MAX_IPC_PAYLOAD_BYTES, PrimaryElectionResult, SingleInstanceIpcPort,
};
pub use core::system::ports::ipc::{decode_ipc_frame, decode_ipc_frame_ref, encode_ipc_frame};
use interprocess::local_socket::traits::{Listener as _, Stream as _};
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, Listener, ListenerOptions, Name, NameType as _, Stream,
    ToFsName as _, ToNsName as _,
};

use crate::shell::resolve_headless_target_path;

const ACK_BYTE: u8 = b'K';
const CONNECT_RETRY_ATTEMPTS: usize = 6;
const CONNECT_RETRY_PAUSE_MS: u64 = 2;

static DEFAULT_ENDPOINT: OnceLock<IpcEndpointConfig> = OnceLock::new();
static ELECTION_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcEndpointConfig {
    pub endpoint_name: String,
    pub socket_file_path: PathBuf,
}

impl Default for IpcEndpointConfig {
    fn default() -> Self {
        default_ipc_endpoint_config().clone()
    }
}

impl IpcEndpointConfig {
    #[inline]
    pub fn for_testing(base_dir: &Path, label: &str) -> Self {
        let clean_label: String =
            label.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        let pid = std::process::id();
        let endpoint_name = format!("codecontext_{pid}_{clean_label}.sock");
        let socket_file_path = base_dir.join(&endpoint_name);
        Self { endpoint_name, socket_file_path }
    }
}

pub fn default_ipc_endpoint_config() -> &'static IpcEndpointConfig {
    DEFAULT_ENDPOINT.get_or_init(|| {
        let user_tag = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "default".to_owned());
        let clean_user: String =
            user_tag.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(16).collect();
        let endpoint_name = format!("codecontext_{clean_user}_v1.sock");

        let base_dir = if cfg!(target_os = "linux") {
            std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, PathBuf::from)
        } else if cfg!(target_os = "macos") {
            PathBuf::from("/tmp")
        } else {
            std::env::temp_dir()
        };

        let socket_file_path = base_dir.join(&endpoint_name);
        IpcEndpointConfig { endpoint_name, socket_file_path }
    })
}

pub fn encode_ipc_json_fallback(command: &IpcCommand) -> Result<Vec<u8>, IpcPortError> {
    let mut out = String::with_capacity(256);
    match command {
        IpcCommand::OpenProjectAndFocus { project_root } => {
            let path_str = project_root
                .to_str()
                .ok_or(IpcPortError::InvalidFrame("Путь не является UTF-8"))?;
            if path_str.is_empty() || path_str.contains('\0') {
                return Err(IpcPortError::InvalidFrame("Пустой путь или запрещённый NUL-байт"));
            }
            out.push_str("{\"op\":\"open\",\"path\":\"");
            for ch in path_str.chars() {
                if ch == '\x22' || ch == '\\' {
                    out.push('\\');
                }
                out.push(ch);
            }
            out.push_str("\"}");
        }
        IpcCommand::FocusWindow => out.push_str("{\"op\":\"focus\"}"),
        IpcCommand::Ping => out.push_str("{\"op\":\"ping\"}"),
    }
    if out.len() > MAX_IPC_FRAME_BYTES {
        return Err(IpcPortError::PayloadTooLarge { size: out.len(), max: MAX_IPC_FRAME_BYTES });
    }
    Ok(out.into_bytes())
}

pub fn decode_ipc_json_fallback(raw: &[u8]) -> Result<IpcCommand, IpcPortError> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| IpcPortError::InvalidFrame("Невалидный UTF-8 в JSON"))?
        .trim();
    if text == "{\"op\":\"focus\"}" {
        return Ok(IpcCommand::FocusWindow);
    }
    if text == "{\"op\":\"ping\"}" {
        return Ok(IpcCommand::Ping);
    }
    if let Some(rest) = text.strip_prefix("{\"op\":\"open\",\"path\":\"")
        && let Some(escaped_path) = rest.strip_suffix("\"}")
    {
        let unescaped = escaped_path.replace("\\\\", "\\").replace("\\\"", "\"");
        return Ok(IpcCommand::OpenProjectAndFocus { project_root: PathBuf::from(unescaped) });
    }
    Err(IpcPortError::InvalidFrame("Не удалось разобрать JSON-команду"))
}

struct ActiveServerGuard {
    config: IpcEndpointConfig,
    shutdown: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Drop for ActiveServerGuard {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Ok(name) = build_socket_name(&self.config) {
            let _ = Stream::connect(name);
        }
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
        if self.config.socket_file_path.exists() {
            let _ = fs::remove_file(&self.config.socket_file_path);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IpcTransportMode {
    InMemory,
    OsLocalSocket,
}

#[derive(Clone)]
pub struct SystemSingleInstanceIpc {
    mode: IpcTransportMode,
    config: IpcEndpointConfig,
    incoming_queue: Arc<Mutex<VecDeque<IpcCommand>>>,
    last_forwarded_path: Arc<Mutex<Option<PathBuf>>>,
    active_server: Arc<Mutex<Option<ActiveServerGuard>>>,
}

impl std::fmt::Debug for SystemSingleInstanceIpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemSingleInstanceIpc")
            .field("mode", &self.mode)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Default for SystemSingleInstanceIpc {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemSingleInstanceIpc {
    #[inline]
    pub fn new() -> Self {
        Self::with_endpoint_config(default_ipc_endpoint_config().clone())
    }

    #[inline]
    pub fn new_in_memory() -> Self {
        Self {
            mode: IpcTransportMode::InMemory,
            config: default_ipc_endpoint_config().clone(),
            incoming_queue: Arc::new(Mutex::new(VecDeque::new())),
            last_forwarded_path: Arc::new(Mutex::new(None)),
            active_server: Arc::new(Mutex::new(None)),
        }
    }

    #[inline]
    pub fn with_endpoint_config(config: IpcEndpointConfig) -> Self {
        Self {
            mode: IpcTransportMode::OsLocalSocket,
            config,
            incoming_queue: Arc::new(Mutex::new(VecDeque::new())),
            last_forwarded_path: Arc::new(Mutex::new(None)),
            active_server: Arc::new(Mutex::new(None)),
        }
    }

    #[inline]
    pub const fn endpoint_config(&self) -> &IpcEndpointConfig {
        &self.config
    }

    #[inline]
    pub fn last_forwarded_path(&self) -> Option<PathBuf> {
        self.last_forwarded_path.lock().ok().and_then(|g| g.clone())
    }

    pub fn push_simulated_command(&self, command: IpcCommand) {
        if let IpcCommand::OpenProjectAndFocus { ref project_root } = command
            && let Ok(mut last) = self.last_forwarded_path.lock()
        {
            *last = Some(project_root.clone());
        }
        if let Ok(mut queue) = self.incoming_queue.lock() {
            queue.push_back(command);
        }
    }

    pub fn send_command_to_primary(&self, command: &IpcCommand) -> Result<bool, IpcPortError> {
        if self.mode == IpcTransportMode::InMemory {
            self.push_simulated_command(command.clone());
            return Ok(false);
        }
        match try_connect_and_send_frame(&self.config, command) {
            Ok(()) => Ok(true),
            Err(IpcPortError::Transport(_)) => Ok(false),
            Err(protocol_err) => Err(protocol_err),
        }
    }

    pub fn send_raw_bytes_for_testing(&self, raw_bytes: &[u8]) -> io::Result<()> {
        let name = build_socket_name(&self.config)?;
        let mut stream = Stream::connect(name)?;
        stream.write_all(raw_bytes)
    }
}

impl SingleInstanceIpcPort for SystemSingleInstanceIpc {
    fn try_send_project_path_to_existing_instance(
        &self,
        project_root: PathBuf,
    ) -> Result<bool, IpcPortError> {
        let sanitized = sanitize_if_needed(project_root);
        let cmd = IpcCommand::OpenProjectAndFocus { project_root: sanitized };
        let sent = self.send_command_to_primary(&cmd)?;
        if let IpcCommand::OpenProjectAndFocus { project_root } = cmd
            && let Ok(mut guard) = self.last_forwarded_path.lock()
        {
            *guard = Some(project_root);
        }
        Ok(sent)
    }

    fn ensure_primary_or_forward(
        &self,
        initial_project_root: Option<PathBuf>,
    ) -> Result<PrimaryElectionResult, IpcPortError> {
        let forward_cmd = initial_project_root.map_or(IpcCommand::FocusWindow, |p| {
            IpcCommand::OpenProjectAndFocus { project_root: sanitize_if_needed(p) }
        });

        if self.mode == IpcTransportMode::InMemory {
            return Ok(PrimaryElectionResult::AcquiredPrimary { stale_socket_recovered: false });
        }

        if self.send_command_to_primary(&forward_cmd)? {
            return Ok(PrimaryElectionResult::ForwardedToPrimary);
        }

        let _election_guard = ELECTION_LOCK.lock().ok();

        if self.send_command_to_primary(&forward_cmd)? {
            return Ok(PrimaryElectionResult::ForwardedToPrimary);
        }

        let mut stale_socket_recovered = false;

        for _attempt in 0..CONNECT_RETRY_ATTEMPTS {
            match bind_listener(&self.config) {
                Ok(listener) => {
                    let guard = spawn_accept_worker(
                        listener,
                        self.config.clone(),
                        Arc::clone(&self.incoming_queue),
                    );
                    if let Ok(mut srv_slot) = self.active_server.lock() {
                        *srv_slot = Some(guard);
                    }
                    return Ok(PrimaryElectionResult::AcquiredPrimary { stale_socket_recovered });
                }
                Err(err)
                    if err.kind() == io::ErrorKind::AddrInUse
                        || err.kind() == io::ErrorKind::PermissionDenied =>
                {
                    if self.send_command_to_primary(&forward_cmd)? {
                        return Ok(PrimaryElectionResult::ForwardedToPrimary);
                    }
                    if !stale_socket_recovered && self.config.socket_file_path.exists() {
                        let _ = fs::remove_file(&self.config.socket_file_path);
                        stale_socket_recovered = true;
                    } else {
                        thread::sleep(Duration::from_millis(CONNECT_RETRY_PAUSE_MS));
                    }
                }
                Err(err) => {
                    return Err(IpcPortError::Transport(err.to_string()));
                }
            }
        }

        if self.send_command_to_primary(&forward_cmd)? {
            return Ok(PrimaryElectionResult::ForwardedToPrimary);
        }

        Err(IpcPortError::Transport(
            "Не удалось ни занять IPC-сокет, ни передать команду".to_owned(),
        ))
    }

    fn try_recv_command(&self) -> Option<IpcCommand> {
        let mut guard = self.incoming_queue.lock().ok()?;
        guard.pop_front()
    }
}

#[inline]
fn sanitize_if_needed(path: PathBuf) -> PathBuf {
    let Some(s) = path.to_str() else {
        return path;
    };
    let needs_explorer_sanitize = s.contains('\x22')
        || s.contains('\'')
        || s.starts_with("\\\\?\\")
        || s.ends_with("\\.")
        || s.ends_with("/.");
    if needs_explorer_sanitize || path.is_file() { resolve_headless_target_path(s) } else { path }
}

fn build_socket_name(config: &IpcEndpointConfig) -> io::Result<Name<'_>> {
    if cfg!(unix) || !GenericNamespaced::is_supported() {
        if let Some(parent) = config.socket_file_path.parent()
            && !parent.as_os_str().is_empty()
        {
            let _ = fs::create_dir_all(parent);
        }
        config.socket_file_path.as_os_str().to_fs_name::<GenericFilePath>()
    } else {
        config.endpoint_name.as_str().to_ns_name::<GenericNamespaced>()
    }
}

fn bind_listener(config: &IpcEndpointConfig) -> io::Result<Listener> {
    let name = build_socket_name(config)?;
    ListenerOptions::new().name(name).create_sync()
}

fn try_connect_and_send_frame(
    config: &IpcEndpointConfig,
    command: &IpcCommand,
) -> Result<(), IpcPortError> {
    let mut frame_buf = [0_u8; MAX_IPC_FRAME_BYTES];
    let frame_len = encode_ipc_frame(command, &mut frame_buf)?;
    let frame_slice = frame_buf.get(..frame_len).unwrap_or(&[]);

    let name = build_socket_name(config).map_err(|e| IpcPortError::Transport(e.to_string()))?;
    let mut stream = Stream::connect(name).map_err(|e| IpcPortError::Transport(e.to_string()))?;

    stream.write_all(frame_slice).map_err(|e| IpcPortError::Transport(e.to_string()))?;

    let mut ack = [0_u8; 1];
    stream.read_exact(&mut ack).map_err(|e| IpcPortError::Transport(e.to_string()))?;
    let [ack_byte] = ack;
    if ack_byte != ACK_BYTE {
        return Err(IpcPortError::InvalidFrame("Неверный байт подтверждения (ACK) от сервера"));
    }
    Ok(())
}

fn spawn_accept_worker(
    listener: Listener,
    config: IpcEndpointConfig,
    queue: Arc<Mutex<VecDeque<IpcCommand>>>,
) -> ActiveServerGuard {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_for_worker = Arc::clone(&shutdown);

    let worker = thread::spawn(move || {
        let mut frame_buf = [0_u8; MAX_IPC_FRAME_BYTES];
        while !shutdown_for_worker.load(Ordering::Relaxed) {
            let Ok(mut stream) = listener.accept() else {
                if shutdown_for_worker.load(Ordering::Relaxed) {
                    break;
                }
                continue;
            };
            if shutdown_for_worker.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(cmd) = read_bounded_frame_from_stream(&mut stream, &mut frame_buf) {
                if let Ok(mut guard) = queue.lock() {
                    guard.push_back(cmd);
                }
                let _ = stream.write_all(&[ACK_BYTE]);
            }
        }
    });

    ActiveServerGuard { config, shutdown, worker: Some(worker) }
}

fn read_bounded_frame_from_stream(
    stream: &mut Stream,
    buf: &mut [u8; MAX_IPC_FRAME_BYTES],
) -> Result<IpcCommand, IpcPortError> {
    let mut read_total = 0_usize;
    while read_total < IPC_HEADER_LEN {
        let rem = buf
            .get_mut(read_total..)
            .ok_or(IpcPortError::InvalidFrame("Срез буфера чтения недоступен"))?;
        let n = stream.read(rem).map_err(|e| IpcPortError::Transport(e.to_string()))?;
        if n == 0 {
            return Err(IpcPortError::InvalidFrame("Неожиданный EOF при чтении IPC-заголовка"));
        }
        read_total += n;
    }

    if buf.get(0..2) != Some(&IPC_PROTOCOL_MAGIC) {
        return Err(IpcPortError::InvalidFrame("Неверные магические байты заголовка"));
    }
    if buf.get(2).copied() != Some(IPC_PROTOCOL_VERSION) {
        return Err(IpcPortError::InvalidFrame("Неверная версия протокола"));
    }

    let len_hi = buf.get(4).copied().unwrap_or(0);
    let len_lo = buf.get(5).copied().unwrap_or(0);
    let payload_len = u16::from_be_bytes([len_hi, len_lo]) as usize;

    if payload_len > MAX_IPC_PAYLOAD_BYTES {
        return Err(IpcPortError::PayloadTooLarge {
            size: payload_len + IPC_HEADER_LEN,
            max: MAX_IPC_FRAME_BYTES,
        });
    }

    let expected_total = IPC_HEADER_LEN + payload_len;
    if read_total < expected_total {
        let missing_slice = buf
            .get_mut(read_total..expected_total)
            .ok_or(IpcPortError::InvalidFrame("Срез тела вышел за границы буфера"))?;
        stream.read_exact(missing_slice).map_err(|e| IpcPortError::Transport(e.to_string()))?;
    }

    let full_frame = buf.get(..expected_total).unwrap_or(&[]);
    decode_ipc_frame(full_frame)
}
