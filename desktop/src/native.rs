use std::{
    mem::size_of,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        OnceLock,
    },
};

use cuesor::{
    platform::{wide, AppResult, OwnedHandle},
    process,
};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{
            GetLastError, COLORREF, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, POINT, WPARAM,
        },
        Graphics::Gdi::HBRUSH,
        System::{
            LibraryLoader::GetModuleHandleW,
            Registry::*,
            Threading::{
                CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject,
                EVENT_MODIFY_STATE,
            },
        },
        UI::{
            Controls::Dialogs::{ChooseColorW, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW},
            Shell::*,
            WindowsAndMessaging::*,
        },
    },
};

#[derive(Clone)]
pub enum TrayAction {
    Show,
    Toggle,
    Exit,
    ColorChosen(bool, Option<u32>),
    ElevationFinished(Result<(), String>),
}
static TRAY_ACTIONS: OnceLock<Sender<TrayAction>> = OnceLock::new();
static TASKBAR_CREATED: OnceLock<u32> = OnceLock::new();
static INDICATOR_ENABLED: AtomicBool = AtomicBool::new(true);
const TRAY_CALLBACK: u32 = WM_APP + 1;

pub fn set_indicator_enabled(enabled: bool) {
    INDICATOR_ENABLED.store(enabled, Ordering::Relaxed);
}

pub struct Instance {
    _mutex: OwnedHandle,
    pub show_event: OwnedHandle,
}

impl Instance {
    pub fn acquire() -> AppResult<Option<Self>> {
        let scope = process::desktop_scope()?;
        let mutex_name = wide(&scope);
        let event_name = wide(format!("{scope}-show"));
        let mutex = OwnedHandle(unsafe { CreateMutexW(None, false, PCWSTR(mutex_name.as_ptr()))? });
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            for _ in 0..20 {
                if let Ok(raw) =
                    unsafe { OpenEventW(EVENT_MODIFY_STATE, false, PCWSTR(event_name.as_ptr())) }
                {
                    let event = OwnedHandle(raw);
                    unsafe {
                        SetEvent(event.0)?;
                    }
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            return Ok(None);
        }
        let show_event =
            OwnedHandle(unsafe { CreateEventW(None, false, false, PCWSTR(event_name.as_ptr()))? });
        Ok(Some(Self {
            _mutex: mutex,
            show_event,
        }))
    }

    pub fn show_requested(&self) -> bool {
        unsafe {
            WaitForSingleObject(self.show_event.0, 0) == windows::Win32::Foundation::WAIT_OBJECT_0
        }
    }
}

pub fn color_dialog(initial: u32, owner: HWND) -> Option<u32> {
    let mut custom = [COLORREF(0); 16];
    let mut dialog = CHOOSECOLORW {
        lStructSize: size_of::<CHOOSECOLORW>() as u32,
        hwndOwner: owner,
        rgbResult: COLORREF(((initial & 0xff) << 16) | (initial & 0xff00) | (initial >> 16)),
        lpCustColors: custom.as_mut_ptr(),
        Flags: CC_FULLOPEN | CC_RGBINIT,
        ..Default::default()
    };
    if unsafe { ChooseColorW(&mut dialog) }.as_bool() {
        let bgr = dialog.rgbResult.0;
        Some(((bgr & 0xff) << 16) | (bgr & 0xff00) | (bgr >> 16))
    } else {
        None
    }
}

pub fn choose_color(chinese: bool, initial: u32, owner: HWND) {
    // A modal loop on GPUI's callback thread re-enters its borrowed App.
    // Run the native loop separately and deliver the result through the UI queue.
    let owner = owner.0 as usize;
    std::thread::spawn(move || {
        let color = color_dialog(initial, HWND(owner as *mut _));
        send(TrayAction::ColorChosen(chinese, color));
    });
}

pub fn set_auto_start(enabled: bool, executable: &Path) -> AppResult<()> {
    unsafe {
        let mut key = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
        .ok()?;
        let result = if enabled {
            let command = wide(format!("\"{}\" --background", executable.display()));
            let bytes =
                std::slice::from_raw_parts(command.as_ptr().cast::<u8>(), command.len() * 2);
            RegSetValueExW(key, w!("Cuesor"), 0, REG_SZ, Some(bytes)).ok()
        } else {
            let result = RegDeleteValueW(key, w!("Cuesor"));
            if result == windows::Win32::Foundation::ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                result.ok()
            }
        };
        let _ = RegCloseKey(key);
        result?;
    }
    Ok(())
}

pub fn set_run_as_admin(enabled: bool, executable: &Path) -> AppResult<()> {
    unsafe {
        let mut key = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers"),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
        .ok()?;
        let path_wide = wide(executable.as_os_str());
        let result = if enabled {
            let flag = wide("~ RUNASADMIN");
            let bytes = std::slice::from_raw_parts(flag.as_ptr().cast::<u8>(), flag.len() * 2);
            RegSetValueExW(key, PCWSTR(path_wide.as_ptr()), 0, REG_SZ, Some(bytes)).ok()
        } else {
            let result = RegDeleteValueW(key, PCWSTR(path_wide.as_ptr()));
            if result == windows::Win32::Foundation::ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                result.ok()
            }
        };
        let _ = RegCloseKey(key);
        result?;
    }
    Ok(())
}

pub fn error_message(message: &str) {
    let text = wide(message);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("Cuesor"),
            MB_ICONERROR | MB_OK,
        );
    }
}

pub fn restart_elevated() -> AppResult<()> {
    let executable = wide(std::env::current_exe()?.as_os_str());
    let mut args = vec![format!("--wait-for {}", std::process::id())];
    for arg in std::env::args().skip(1) {
        if arg == "--background" {
            args.push(arg);
        }
    }
    let arguments = wide(args.join(" "));
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("runas"),
            PCWSTR(executable.as_ptr()),
            PCWSTR(arguments.as_ptr()),
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err(crate::i18n::error_elevation_failed(cuesor::i18n::Locale::current()).into());
    }
    Ok(())
}

pub fn request_elevation() {
    std::thread::spawn(|| {
        send(TrayAction::ElevationFinished(
            restart_elevated().map_err(|e| e.to_string()),
        ))
    });
}

pub struct Tray {
    hwnd: usize,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Tray {
    pub fn start(sender: Sender<TrayAction>) -> AppResult<Self> {
        TRAY_ACTIONS
            .set(sender)
            .map_err(|_| "tray already initialized")?;
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let result = unsafe { tray_loop(&ready_tx) };
            if let Err(error) = result {
                let _ = ready_tx.send(Err(error.to_string()));
            }
        });
        let hwnd = ready_rx
            .recv()
            .map_err(|_| "tray thread exited")?
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
        Ok(Self {
            hwnd,
            thread: Some(thread),
        })
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            let _ = PostMessageW(HWND(self.hwnd as *mut _), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

unsafe fn tray_loop(ready: &std::sync::mpsc::Sender<Result<usize, String>>) -> AppResult<()> {
    let instance = GetModuleHandleW(None)?;
    let class = WNDCLASSW {
        lpfnWndProc: Some(tray_proc),
        hInstance: instance.into(),
        lpszClassName: w!("Cuesor.Tray"),
        hbrBackground: HBRUSH::default(),
        ..Default::default()
    };
    if RegisterClassW(&class) == 0 {
        return Err(windows::core::Error::from_win32().into());
    }
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        class.lpszClassName,
        w!("Cuesor tray"),
        WINDOW_STYLE::default(),
        0,
        0,
        0,
        0,
        None,
        None,
        instance,
        None,
    )?;
    let icon = tray_icon()?;
    let mut notification = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_TIP | NIF_MESSAGE,
        uCallbackMessage: TRAY_CALLBACK,
        hIcon: icon,
        ..Default::default()
    };
    let tip = crate::i18n::tray_tooltip(cuesor::i18n::Locale::current());
    for (slot, unit) in notification
        .szTip
        .iter_mut()
        .zip(tip.encode_utf16())
    {
        *slot = unit;
    }
    if !Shell_NotifyIconW(NIM_ADD, &notification).as_bool() {
        let _ = DestroyIcon(icon);
        let _ = DestroyWindow(hwnd);
        return Err(crate::i18n::tray_error_icon(cuesor::i18n::Locale::current()).into());
    }
    TASKBAR_CREATED.get_or_init(|| RegisterWindowMessageW(w!("TaskbarCreated")));
    SetWindowLongPtrW(
        hwnd,
        GWLP_USERDATA,
        &notification as *const NOTIFYICONDATAW as isize,
    );
    let _ = ready.send(Ok(hwnd.0 as usize));
    let mut message = MSG::default();
    while GetMessageW(&mut message, None, 0, 0).as_bool() {
        let _ = TranslateMessage(&message);
        DispatchMessageW(&message);
    }
    let _ = Shell_NotifyIconW(NIM_DELETE, &notification);
    let _ = DestroyIcon(icon);
    Ok(())
}

unsafe extern "system" fn tray_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if TASKBAR_CREATED
        .get()
        .is_some_and(|&m| m != 0 && m == message)
    {
        let data = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const NOTIFYICONDATAW;
        if !data.is_null() {
            let _ = Shell_NotifyIconW(NIM_ADD, data);
        }
        return LRESULT(0);
    }
    match message {
        WM_COMMAND => {
            match wparam.0 & 0xffff {
                1 => send(TrayAction::Show),
                2 => send(TrayAction::Toggle),
                3 => send(TrayAction::Exit),
                _ => (),
            }
            LRESULT(0)
        }
        TRAY_CALLBACK => {
            match lparam.0 as u32 {
                WM_LBUTTONDBLCLK => send(TrayAction::Show),
                WM_RBUTTONUP | WM_CONTEXTMENU => {
                    if let Ok(menu) = CreatePopupMenu() {
                        let locale = cuesor::i18n::Locale::current();
                        let _ = AppendMenuW(menu, MF_STRING, 1, crate::i18n::tray_menu_open_settings(locale));
                        let action = if INDICATOR_ENABLED.load(Ordering::Relaxed) {
                            crate::i18n::tray_menu_pause(locale)
                        } else {
                            crate::i18n::tray_menu_resume(locale)
                        };
                        let _ = AppendMenuW(menu, MF_STRING, 2, action);
                        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
                        let _ = AppendMenuW(menu, MF_STRING, 3, crate::i18n::tray_menu_exit(locale));
                        let mut point = POINT::default();
                        let _ = GetCursorPos(&mut point);
                        let _ = SetForegroundWindow(hwnd);
                        let selected = TrackPopupMenu(
                            menu,
                            TPM_RETURNCMD | TPM_RIGHTBUTTON,
                            point.x,
                            point.y,
                            0,
                            hwnd,
                            None,
                        )
                        .0;
                        match selected {
                            1 => send(TrayAction::Show),
                            2 => send(TrayAction::Toggle),
                            3 => send(TrayAction::Exit),
                            _ => (),
                        }
                        let _ = DestroyMenu(menu);
                        let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
                    }
                }
                _ => (),
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn send(action: TrayAction) {
    if let Some(sender) = TRAY_ACTIONS.get() {
        let _ = sender.send(action);
    }
}

unsafe fn tray_icon() -> AppResult<HICON> {
    let instance = GetModuleHandleW(None)?.into();
    let icon = LoadImageW(
        Some(&instance),
        PCWSTR(std::ptr::without_provenance(1)),
        IMAGE_ICON,
        32,
        32,
        IMAGE_FLAGS::default(),
    )?;
    Ok(HICON(icon.0))
}
