mod process;
mod module;
pub use crate::mem::module::*;
pub use crate::mem::process::*;

use std::ops::Deref;
use regex::bytes::Regex;
use winapi::shared::ntdef::HANDLE;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::tlhelp32::CreateToolhelp32Snapshot;

/// Wrapper around the windows `HANDLE` returned from
/// `kernel32::CreateToolhelp32Snapshot`.
pub struct SnapshotHandle {
    pub handle: HANDLE,
}

impl SnapshotHandle {
    /// Constructs a new `SnapshotHandle`.
    ///
    /// Calls the `kernel32::CreateToolhelp32Snapshot` windows api.
    pub fn new(pid: u32, flags: u32) -> Option<Self> {
        let handle = unsafe { CreateToolhelp32Snapshot(flags, pid) };
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return None;
        }

        Some(SnapshotHandle { handle })
    }
}

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl Deref for SnapshotHandle {
    type Target = HANDLE;
    fn deref(&self) -> &HANDLE {
        &self.handle
    }
}

pub trait Constructor {
    fn new() -> Self;
}


/// Enables the user to generate a byte regex out of the normal signature
/// format.
pub fn generate_regex(raw: &str) -> Option<Regex> {
    let mut res = raw
        .to_string()
        .split_whitespace()
        .map(|x| match &x {
            &"?" => ".".to_string(),
            x => format!("\\x{}", x),
        })
        .collect::<Vec<_>>()
        .join("");
    res.insert_str(0, "(?s-u)");
    Regex::new(&res).ok()
}

/// Find pattern.
pub fn find_pattern(data: &[u8], pattern: &str) -> Option<usize> {
    generate_regex(pattern)
        .and_then(|r| r.find(data))
        .and_then(|m| Some(m.start()))
}