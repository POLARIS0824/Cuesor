use std::mem::size_of;
use windows::core::{w, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, HWND, LPARAM, WPARAM,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::{
    AttachConsole, FreeConsole, GetConsoleMode, GetConsoleWindow, WriteConsoleW, CONSOLE_MODE,
    ENABLE_PROCESSED_OUTPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
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

pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
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
    pub ime_hwnd: HWND,
    pub open: Option<usize>,
    pub conversion: Option<usize>,
    pub mode: ImeMode,
    pub stable: bool,
}

pub fn sample(host: Option<HWND>) -> Sample {
    let mut result = Sample {
        focus: focus(),
        ime_hwnd: HWND::default(),
        open: None,
        conversion: None,
        mode: ImeMode::Unknown,
        stable: false,
    };
    let Some(before) = result.focus else {
        return result;
    };
    if host.is_some_and(|h| h != before.foreground) || !is_terminal(before.foreground) {
        return result;
    }
    let language = (before.layout & 0xffff) as u16;
    if language == 0x0804 {
        result.ime_hwnd = unsafe { ImmGetDefaultIMEWnd(before.focused) };
        if !result.ime_hwnd.0.is_null() {
            result.open = query(result.ime_hwnd, 5);
            if result.open.is_some_and(|v| v != 0) {
                result.conversion = query(result.ime_hwnd, 1);
            }
        }
    }
    result.stable = focus() == Some(before);
    if result.stable {
        result.mode = classify(language, result.open, result.conversion);
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

pub fn output_mode() -> AppResult<u32> {
    let handle = console_output()?;
    let mut mode = CONSOLE_MODE::default();
    unsafe {
        GetConsoleMode(handle.0, &mut mode)?;
    }
    Ok(mode.0)
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
