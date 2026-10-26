use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use windows::core::{w, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, HWND, LPARAM, WPARAM,
};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::{
    AttachConsole, FreeConsole, GetConsoleMode, GetConsoleWindow, WriteConsoleW, CONSOLE_MODE,
    ENABLE_PROCESSED_OUTPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout;
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetClassNameW, GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId,
    SendMessageTimeoutW, GA_ROOTOWNER, GUITHREADINFO, SMTO_ABORTIFHUNG, SMTO_ERRORONEXIT,
};

use crate::model::{classify, ImeMode};

pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct OwnedHandle(pub HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn wide(value: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}

pub fn class_name(hwnd: HWND) -> String {
    let mut buffer = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) }.max(0) as usize;
    String::from_utf16_lossy(&buffer[..len])
}

pub fn process_name(hwnd: HWND) -> Option<String> {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    let handle =
        OwnedHandle(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?);
    let mut buffer = [0u16; 2048];
    let mut len = buffer.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(
            handle.0,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        )
    }
    .ok()?;
    let path = String::from_utf16_lossy(&buffer[..len as usize]);
    Some(path.rsplit(['\\', '/']).next()?.to_owned())
}

pub fn process_elevated(pid: u32) -> AppResult<bool> {
    let process =
        OwnedHandle(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)? });
    let mut raw = HANDLE::default();
    unsafe { OpenProcessToken(process.0, TOKEN_QUERY, &mut raw)? };
    let token = OwnedHandle(raw);
    let mut value = TOKEN_ELEVATION::default();
    let mut length = 0;
    unsafe {
        GetTokenInformation(
            token.0,
            TokenElevation,
            Some((&mut value as *mut TOKEN_ELEVATION).cast()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut length,
        )?;
    }
    Ok(value.TokenIsElevated != 0)
}

pub fn terminal_host() -> HWND {
    unsafe { GetAncestor(GetConsoleWindow(), GA_ROOTOWNER) }
}

pub fn is_terminal(hwnd: HWND) -> bool {
    !hwnd.0.is_null()
        && class_name(hwnd) == "CASCADIA_HOSTING_WINDOW_CLASS"
        && process_name(hwnd).is_some_and(|p| p.eq_ignore_ascii_case("WindowsTerminal.exe"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Focus {
    pub foreground: HWND,
    pub focused: HWND,
    pub thread_id: u32,
    pub layout: usize,
}

pub fn focus() -> Option<Focus> {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() {
            return None;
        }
        let thread_id = GetWindowThreadProcessId(foreground, None);
        let mut info = GUITHREADINFO {
            cbSize: size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        GetGUIThreadInfo(thread_id, &mut info).ok()?;
        let focused = if info.hwndFocus.0.is_null() {
            foreground
        } else {
            info.hwndFocus
        };
        let focus_thread = GetWindowThreadProcessId(focused, None);
        Some(Focus {
            foreground,
            focused,
            thread_id: focus_thread,
            layout: GetKeyboardLayout(focus_thread).0 as usize,
        })
    }
}

fn query(hwnd: HWND, command: usize) -> Option<usize> {
    let mut result = 0usize;
    let ok = unsafe {
        SendMessageTimeoutW(
            hwnd,
            0x0283,
            WPARAM(command),
            LPARAM(0),
            SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
            25,
            Some(&mut result),
        )
    };
    (ok.0 != 0).then_some(result)
}

pub struct Sample {
    pub focus: Option<Focus>,
    pub mode: ImeMode,
}

pub fn sample(host: HWND) -> Sample {
    let mut result = Sample {
        focus: focus(),
        mode: ImeMode::Unknown,
    };
    let Some(before) = result.focus else {
        return result;
    };
    if host != before.foreground || !is_terminal(before.foreground) {
        return result;
    }
    let language = (before.layout & 0xffff) as u16;
    let mut open = None;
    let mut conversion = None;
    if language == 0x0804 {
        let ime = unsafe { ImmGetDefaultIMEWnd(before.focused) };
        if !ime.0.is_null() {
            open = query(ime, 5);
            if open.is_some_and(|v| v != 0) {
                conversion = query(ime, 1);
            }
        }
    }
    if focus() == Some(before) {
        result.mode = classify(language, open, conversion);
    }
    result
}

pub fn attach(pid: u32) -> AppResult<()> {
    unsafe {
        let _ = FreeConsole();
        AttachConsole(pid)?;
    }
    Ok(())
}

fn console_output() -> AppResult<OwnedHandle> {
    Ok(OwnedHandle(unsafe {
        CreateFileW(
            w!("CONOUT$"),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
    }))
}

pub fn write_sequence(sequence: &str) -> AppResult<()> {
    let handle = console_output()?;
    let mut mode = CONSOLE_MODE::default();
    unsafe {
        GetConsoleMode(handle.0, &mut mode)?;
    }
    let required = ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    if !mode.contains(required) {
        return Err("console VT output is disabled; no console modes were changed".into());
    }
    let buffer: Vec<u16> = sequence.encode_utf16().collect();
    let mut written = 0;
    unsafe {
        WriteConsoleW(handle.0, &buffer, Some(&mut written), None)?;
    }
    if written as usize != buffer.len() {
        return Err("incomplete console write".into());
    }
    Ok(())
}
