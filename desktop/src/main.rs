#![windows_subsystem = "windows"]

mod native;

use gpui::{prelude::*, *};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::mpsc::Receiver, time::Duration};
use terminal_ime::{
    agent::{self, RuntimeStatus},
    model::Color,
    platform::{AppResult, OwnedHandle},
    settings::{self, Settings},
};
use windows::Win32::{
    Foundation::HWND,
    System::Threading::WaitForSingleObject,
    UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindow, SW_HIDE, SW_RESTORE},
};

const INK: u32 = 0x1c2638;
const MUTED: u32 = 0x7b8495;
const ACCENT: u32 = 0x5865d8;

struct SettingsView {
    options: Settings,
    directory: PathBuf,
    runtime: RuntimeStatus,
    notice: String,
    agent: OwnedHandle,
    instance: Rc<native::Instance>,
    receiver: Receiver<native::TrayAction>,
    hwnd: usize,
    elevated: bool,
    color_dialog_open: bool,
    elevating: bool,
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
        cx.spawn(async move |view, cx| {
            let mut ticks = 0;
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
                let result = view.update(cx, |view, cx| {
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
                            }
                            native::TrayAction::ElevationFinished(result) => {
                                view.elevating = false;
                                match result {
                                    Ok(()) => cx.quit(),
                                    Err(error) => {
                                        view.notice = error;
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
        Self {
            options,
            directory,
            runtime: RuntimeStatus::default(),
            notice: "设置自动保存，并应用到已接入的终端。".into(),
            agent,
            instance,
            receiver,
            hwnd,
            elevated: native::is_elevated(),
            color_dialog_open: false,
            elevating: false,
        }
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
                self.notice = "已保存，终端会自动应用新设置。".into();
            }
            Err(error) => self.notice = format!("保存失败：{error}"),
        }
        cx.notify();
    }

    fn change_color(&mut self, chinese: bool, color: u32, cx: &mut Context<Self>) {
        let mut next = self.options.clone();
        if chinese {
            next.chinese_color = format!("#{color:06X}");
        } else {
            next.english_color = format!("#{color:06X}");
        }
        self.persist(next, cx);
    }

    fn elevate(&mut self, cx: &mut Context<Self>) {
        if !self.elevating {
            self.elevating = true;
            self.notice = "请处理 Windows 的管理员授权提示。".into();
            native::request_elevation();
            cx.notify();
        }
    }

    fn color_card(&self, chinese: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let value = if chinese {
            &self.options.chinese_color
        } else {
            &self.options.english_color
        };
        let color = Color::parse(value).unwrap().0;
        let label = if chinese {
            "中文输入"
        } else {
            "英文输入"
        };
        let key = if chinese { "cn" } else { "en" };
        let palette = [0xff7800, 0xf05c72, 0xe4b443, 0x20a78b, 0x0077ff, 0x8163de];
        div()
            .flex_1()
            .p_5()
            .rounded_xl()
            .bg(rgb(0xffffff))
            .border_1()
            .border_color(rgb(0xe5e8ef))
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(label))
                    .child(div().text_sm().text_color(rgb(MUTED)).child(if chinese {
                        "中"
                    } else {
                        "EN"
                    })),
            )
            .child(
                div()
                    .h(px(90.))
                    .rounded_lg()
                    .bg(rgb(0x182130))
                    .p_4()
                    .flex()
                    .items_center()
                    .gap_2()
                    .font_family("Consolas")
                    .text_color(rgb(0xc7d1e2))
                    .child(if chinese {
                        "> 输入一些文字"
                    } else {
                        "> echo hello"
                    })
                    .child(div().w(px(10.)).h(px(22.)).bg(rgb(color))),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().size_4().rounded_sm().bg(rgb(color)))
                            .child(value.clone()),
                    )
                    .child(
                        div()
                            .id((key, 1u32))
                            .cursor_pointer()
                            .text_sm()
                            .text_color(rgb(ACCENT))
                            .child("自定义颜色")
                            .on_click(cx.listener(move |view, _, window, _| {
                                if !view.color_dialog_open {
                                    view.color_dialog_open = true;
                                    native::choose_color(chinese, color, native_hwnd(window));
                                }
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .children(palette.into_iter().enumerate().map(move |(i, choice)| {
                        div()
                            .id((if chinese { "cn-swatch" } else { "en-swatch" }, i))
                            .size_6()
                            .rounded_full()
                            .bg(rgb(choice))
                            .cursor_pointer()
                            .border_2()
                            .border_color(rgb(if color == choice { INK } else { 0xffffff }))
                            .on_click(cx.listener(move |view, _, _, cx| {
                                view.change_color(chinese, choice, cx)
                            }))
                    })),
            )
    }

    fn switch(
        &self,
        id: &'static str,
        enabled: bool,
        action: impl Fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .w(px(42.))
            .h(px(24.))
            .rounded_full()
            .px(px(3.))
            .flex()
            .items_center()
            .when(enabled, |d| d.justify_end())
            .bg(rgb(if enabled { ACCENT } else { 0xcbd0dc }))
            .cursor_pointer()
            .child(div().size(px(18.)).rounded_full().bg(rgb(0xffffff)))
            .on_click(cx.listener(action))
    }
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.runtime.sessions.len();
        let cn = self.color_card(true, cx).into_any_element();
        let en = self.color_card(false, cx).into_any_element();
        div()
            .size_full()
            .bg(rgb(0xf5f6fa))
            .text_color(rgb(INK))
            .font_family("Microsoft YaHei UI")
            .text_sm()
            .flex()
            .flex_col()
            .child(
                div()
                    .px_8()
                    .py_5()
                    .bg(rgb(0xffffff))
                    .border_b_1()
                    .border_color(rgb(0xe5e8ef))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size_9()
                                    .rounded_lg()
                                    .bg(rgb(ACCENT))
                                    .text_color(rgb(0xffffff))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .child("I"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Terminal IME"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(MUTED))
                                            .child("输入状态，一眼可见"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_full()
                            .bg(rgb(if self.options.enabled {
                                0xe5f5ed
                            } else {
                                0xe9ebf1
                            }))
                            .text_color(rgb(if self.options.enabled {
                                0x23845b
                            } else {
                                MUTED
                            }))
                            .child(if self.options.enabled {
                                "● 指示已启用"
                            } else {
                                "○ 已暂停"
                            }),
                    ),
            )
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_8()
                    .py_6()
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("让光标告诉你输入状态"),
                            )
                            .child(
                                div()
                                    .text_color(rgb(MUTED))
                                    .child("自动跟随中英文切换，适用于 Windows Terminal。"),
                            ),
                    )
                    .child(div().flex_shrink_0().flex().gap_4().child(cn).child(en))
                    .child(
                        div()
                            .flex_shrink_0()
                            .rounded_xl()
                            .bg(rgb(0xffffff))
                            .border_1()
                            .border_color(rgb(0xe5e8ef))
                            .px_5()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .py_4()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child("启用光标颜色指示")
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(MUTED))
                                                    .child("暂停时恢复终端默认光标颜色"),
                                            ),
                                    )
                                    .child(self.switch(
                                        "enable",
                                        self.options.enabled,
                                        |view, _, _, cx| {
                                            let mut next = view.options.clone();
                                            next.enabled = !next.enabled;
                                            view.persist(next, cx);
                                        },
                                        cx,
                                    )),
                            )
                            .child(
                                div()
                                    .py_4()
                                    .border_t_1()
                                    .border_color(rgb(0xeff0f5))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child("登录 Windows 时启动")
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(MUTED))
                                                    .child("启动后驻留托盘，自动接入终端"),
                                            ),
                                    )
                                    .child(self.switch(
                                        "autostart",
                                        self.options.auto_start,
                                        |view, _, _, cx| {
                                            let mut next = view.options.clone();
                                            next.auto_start = !next.auto_start;
                                            view.persist(next, cx);
                                        },
                                        cx,
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .rounded_xl()
                            .p_4()
                            .bg(rgb(0xeceefa))
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().font_weight(FontWeight::MEDIUM).child(
                                if !self.runtime.elevated_terminal_pids.is_empty() {
                                    if count == 0 {
                                        "检测到管理员终端，需要切换管理员模式".to_string()
                                    } else {
                                        format!("已接入 {count} 个会话；另有管理员终端需要授权")
                                    }
                                } else if !self.runtime.errors.is_empty() {
                                    "部分终端未能接入，请检查权限或重新打开程序".to_string()
                                } else if count == 0 {
                                    "正在等待 Windows Terminal 会话".to_string()
                                } else {
                                    format!("已发现 {count} 个终端会话 · 自动接入")
                                },
                            ))
                            .child(div().text_xs().text_color(rgb(0x667092)).child(
                                if !self.runtime.elevated_terminal_pids.is_empty() {
                                    "软件与终端权限不一致。请点击下方按钮，在 Windows 授权提示中选择“是”。"
                                } else {
                                    "打开终端即可使用，无需执行 Nushell 命令；新会话数秒内接入。"
                                },
                            ))
                            .when(!self.runtime.elevated_terminal_pids.is_empty(), |d| {
                                d.child(
                                    div()
                                        .id("terminal-access-admin")
                                        .text_sm()
                                        .text_color(rgb(ACCENT))
                                        .cursor_pointer()
                                        .child(if self.elevating { "正在请求授权…" } else { "以管理员身份重新启动" })
                                        .on_click(cx.listener(|view, _, _, cx| view.elevate(cx))),
                                )
                            })
                            .children(self.runtime.sessions.iter().take(3).map(|s| {
                                div().text_xs().text_color(rgb(0x667092)).child(format!(
                                    "{} · 已接入",
                                    match s.shell.as_str() {
                                        "nu.exe" => "Nushell",
                                        "powershell.exe" | "pwsh.exe" => "PowerShell",
                                        "cmd.exe" => "命令提示符",
                                        other => other,
                                    }
                                ))
                            })),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(MUTED))
                                    .child(if self.elevated {
                                        "管理员模式 · 关闭窗口后继续在托盘运行。"
                                    } else {
                                        "关闭窗口后继续在托盘运行。管理员终端需要管理员模式。"
                                    }),
                            )
                            .when(!self.elevated, |d| {
                                d.child(
                                    div()
                                        .id("admin-mode")
                                        .text_xs()
                                        .text_color(rgb(ACCENT))
                                        .cursor_pointer()
                                        .child("切换管理员模式")
                                        .on_click(cx.listener(|view, _, _, cx| view.elevate(cx))),
                                )
                            }),
                    ),
            )
            .child(
                div()
                    .px_8()
                    .py_4()
                    .bg(rgb(0xffffff))
                    .border_t_1()
                    .border_color(rgb(0xe5e8ef))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(self.notice.clone()),
                    )
                    .child(
                        div()
                            .id("reset-defaults")
                            .cursor_pointer()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(0xdfe3ec))
                            .child("恢复默认颜色")
                            .on_click(cx.listener(|view, _, _, cx| {
                                let mut next = view.options.clone();
                                next.chinese_color = "#FF7800".into();
                                next.english_color = "#0077FF".into();
                                view.persist(next, cx);
                            })),
                    ),
            )
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
    let background = std::env::args().any(|arg| arg == "--background");
    let (sender, receiver) = std::sync::mpsc::channel();
    let _tray = native::Tray::start(sender)?;
    let agent = agent::start_controller(std::process::id(), &directory)?;
    let startup_error = Rc::new(RefCell::new(None));
    let window_error = startup_error.clone();
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.), px(740.)), cx);
        let result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(780.), px(620.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Terminal IME · 设置".into()),
                    ..Default::default()
                }),
                show: !background,
                focus: !background,
                ..Default::default()
            },
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
