#![windows_subsystem = "windows"]

mod native;
mod radix_theme;
mod ui;

extern crate gpui_kit as gpui;
use cuesor::{
    agent::{self, RuntimeStatus},
    platform::{AppResult, OwnedHandle},
    settings::{self, Appearance, Settings, WindowSize},
};
use gpui::{prelude::*, *};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};
use windows::Win32::{
    Foundation::HWND,
    System::Threading::WaitForSingleObject,
    UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE},
};

struct SettingsView {
    options: Settings,
    directory: PathBuf,
    runtime: RuntimeStatus,
    notice: String,
    notice_is_error: bool,
    controller_stopped: bool,
    agent: OwnedHandle,
    instance: Rc<native::Instance>,
    receiver: Receiver<native::TrayAction>,
    hwnd: usize,
    elevated: bool,
    color_dialog_open: bool,
    elevating: bool,
    window_size: WindowSize,
    size_save_at: Option<Instant>,
}

impl SettingsView {
    fn new(
        directory: PathBuf,
        options: Settings,
        agent: OwnedHandle,
        instance: Rc<native::Instance>,
        receiver: Receiver<native::TrayAction>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let hwnd = native_hwnd(window).0 as usize;
        window.on_window_should_close(cx, move |_, _| {
            unsafe {
                let _ = ShowWindow(HWND(hwnd as *mut _), SW_HIDE);
            }
            false
        });
        cx.observe_window_appearance(window, |view, window, cx| {
            if view.options.appearance == Appearance::System {
                view.apply_appearance(window, cx);
            }
        })
        .detach();
        cx.observe_window_bounds(window, |view, window, _| {
            if let WindowBounds::Windowed(bounds) = window.window_bounds() {
                let size = WindowSize {
                    width: f32::from(bounds.size.width).round() as u32,
                    height: f32::from(bounds.size.height).round() as u32,
                };
                if size.width >= WindowSize::MIN.width
                    && size.height >= WindowSize::MIN.height
                    && size != view.window_size
                {
                    view.window_size = size;
                    view.size_save_at = Some(Instant::now() + Duration::from_millis(350));
                }
            }
        })
        .detach();
        cx.on_app_quit(|view, _| {
            let _ = view.window_size.save(&view.directory);
            async {}
        })
        .detach();
        cx.spawn(async move |view, cx| {
            let mut ticks = 0;
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
                let result = view.update(cx, |view, cx| {
                    if view
                        .size_save_at
                        .is_some_and(|deadline| Instant::now() >= deadline)
                    {
                        view.size_save_at = None;
                        if let Err(error) = view.window_size.save(&view.directory) {
                            view.notice = format!("窗口尺寸保存失败：{error}");
                            view.notice_is_error = true;
                            cx.notify();
                        }
                    }
                    if view.instance.show_requested() {
                        view.show(cx);
                    }
                    while let Ok(action) = view.receiver.try_recv() {
                        match action {
                            native::TrayAction::Show => view.show(cx),
                            native::TrayAction::Toggle => {
                                let mut next = view.options.clone();
                                next.enabled = !next.enabled;
                                view.persist(next, cx);
                            }
                            native::TrayAction::Exit => cx.quit(),
                            native::TrayAction::ColorChosen(chinese, color) => {
                                view.color_dialog_open = false;
                                if let Some(color) = color {
                                    view.change_color(chinese, color, cx);
                                }
                                cx.notify();
                            }
                            native::TrayAction::ElevationFinished(result) => {
                                view.elevating = false;
                                match result {
                                    Ok(()) => cx.quit(),
                                    Err(error) => {
                                        view.notice = error;
                                        view.notice_is_error = true;
                                        cx.notify();
                                    }
                                }
                            }
                        }
                    }
                    ticks += 1;
                    if ticks % 10 == 0 {
                        let current = std::fs::read(view.directory.join("runtime.json"))
                            .ok()
                            .and_then(|bytes| serde_json::from_slice::<RuntimeStatus>(&bytes).ok())
                            .filter(|r| r.controller_pid == std::process::id());
                        if let Some(runtime) = current {
                            let changed = serde_json::to_vec(&runtime).ok()
                                != serde_json::to_vec(&view.runtime).ok();
                            view.runtime = runtime;
                            if changed {
                                cx.notify();
                            }
                        }
                        if unsafe { WaitForSingleObject(view.agent.0, 0) }
                            == windows::Win32::Foundation::WAIT_OBJECT_0
                        {
                            view.notice =
                                "后台已停止，请退出后重新打开；可查看运行记录排查。".into();
                            view.notice_is_error = true;
                            view.controller_stopped = true;
                            view.runtime = RuntimeStatus::default();
                            cx.notify();
                        }
                    }
                });
                if result.is_err() {
                    break;
                }
            }
        })
        .detach();
        let view = Self {
            options,
            directory,
            runtime: RuntimeStatus::default(),
            notice: "设置自动保存".into(),
            notice_is_error: false,
            controller_stopped: false,
            agent,
            instance,
            receiver,
            hwnd,
            elevated: native::is_elevated(),
            color_dialog_open: false,
            elevating: false,
            window_size: WindowSize {
                width: f32::from(window.bounds().size.width).round() as u32,
                height: f32::from(window.bounds().size.height).round() as u32,
            },
            size_save_at: None,
        };
        view.apply_appearance(window, cx);
        view
    }

    fn show(&self, cx: &mut App) {
        unsafe {
            let _ = ShowWindow(HWND(self.hwnd as *mut _), SW_RESTORE);
            let _ = SetForegroundWindow(HWND(self.hwnd as *mut _));
        }
        cx.activate(true);
    }

    fn persist(&mut self, next: Settings, cx: &mut Context<Self>) {
        let result = (|| -> AppResult<()> {
            if next.auto_start != self.options.auto_start {
                native::set_auto_start(next.auto_start, &std::env::current_exe()?)?;
            }
            if let Err(error) = next.save(&self.directory) {
                if next.auto_start != self.options.auto_start {
                    let _ =
                        native::set_auto_start(self.options.auto_start, &std::env::current_exe()?);
                }
                return Err(error);
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.options = next;
                native::set_indicator_enabled(self.options.enabled);
                self.notice = "设置已保存".into();
                self.notice_is_error = false;
            }
            Err(error) => {
                self.notice = format!("保存失败：{error}");
                self.notice_is_error = true;
            }
        }
        cx.notify();
    }

    fn change_color(&mut self, chinese: bool, color: u32, cx: &mut Context<Self>) {
        let mut next = self.options.clone();
        next.customize_color(chinese, color);
        self.persist(next, cx);
    }

    fn elevate(&mut self, cx: &mut Context<Self>) {
        if !self.elevating {
            self.elevating = true;
            self.notice = "请处理 Windows 的管理员授权提示。".into();
            self.notice_is_error = false;
            native::request_elevation();
            cx.notify();
        }
    }
}

fn native_hwnd(window: &Window) -> HWND {
    match HasWindowHandle::window_handle(window)
        .expect("Windows window handle")
        .as_raw()
    {
        RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as *mut _),
        _ => unreachable!("Windows only"),
    }
}

fn run() -> AppResult<()> {
    let arguments: Vec<_> = std::env::args().collect();
    if let Some(index) = arguments.iter().position(|arg| arg == "--wait-for") {
        if let Some(pid) = arguments.get(index + 1).and_then(|s| s.parse::<u32>().ok()) {
            if pid != std::process::id() {
                unsafe {
                    if let Ok(handle) = windows::Win32::System::Threading::OpenProcess(
                        windows::Win32::System::Threading::PROCESS_SYNCHRONIZE,
                        false,
                        pid,
                    ) {
                        let old = OwnedHandle(handle);
                        WaitForSingleObject(old.0, 5000);
                    }
                }
            }
        }
    }
    let Some(instance) = native::Instance::acquire()? else {
        return Ok(());
    };
    let directory = settings::data_directory()?;
    let options = Settings::load(&directory)?;
    options.save(&directory)?;
    native::set_indicator_enabled(options.enabled);
    let background = std::env::args().any(|arg| arg == "--background");
    let (sender, receiver) = std::sync::mpsc::channel();
    let _tray = native::Tray::start(sender)?;
    let agent = agent::start_controller(std::process::id(), &directory)?;
    let startup_error = Rc::new(RefCell::new(None));
    let window_error = startup_error.clone();
    gpui_kit::application()
        .with_assets(ui::AppAssets)
        .run(move |cx: &mut App| {
            gpui_kit::init(cx);
            radix_theme::install(cx);
            let stored_size = WindowSize::load(&directory);
            let restored_size = cx
                .primary_display()
                .map(|display| {
                    let available = display.visible_bounds().size;
                    stored_size.constrained(
                        f32::from(available.width) as u32,
                        f32::from(available.height) as u32,
                    )
                })
                .unwrap_or(stored_size);
            let bounds = Bounds::centered(
                None,
                size(
                    px(restored_size.width as f32),
                    px(restored_size.height as f32),
                ),
                cx,
            );
            let result = gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    // Platform window constraints use logical pixels; content uses rem-based helpers.
                    window_min_size: Some(size(
                        px(WindowSize::MIN.width as f32),
                        px(WindowSize::MIN.height as f32),
                    )),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Cuesor 设置".into()),
                        ..Default::default()
                    }),
                    show: !background,
                    focus: !background,
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    cx.new(|cx| {
                        SettingsView::new(
                            directory,
                            options,
                            agent,
                            Rc::new(instance),
                            receiver,
                            window,
                            cx,
                        )
                    })
                },
            );
            if let Err(error) = result {
                *window_error.borrow_mut() = Some(error.to_string());
                cx.quit();
            }
            if !background {
                cx.activate(true);
            }
        });
    if let Some(error) = startup_error.borrow_mut().take() {
        return Err(error.into());
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        native::error_message(&error.to_string());
        std::process::exit(1);
    }
}
