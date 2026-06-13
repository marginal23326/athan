mod app;
mod audio;
mod config;
mod tray;
mod ui;

use app::{App, Message, TrayRxHandle, update};
use iced::widget::{center, mouse_area, opaque, stack};
use iced::{Element, Subscription, Task};
use std::sync::Arc;

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
struct TimerId;

#[derive(Clone)]
struct TrayReceiver(TrayRxHandle);

impl std::hash::Hash for TrayReceiver {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl PartialEq for TrayReceiver {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TrayReceiver {}

fn view(app: &App, id: iced::window::Id) -> Element<'_, Message> {
    if app.window_id != Some(id) {
        return iced::widget::text("").into();
    }

    let base = ui::views::main_view(app);

    if app.settings_open {
        let settings_layer = opaque(
            mouse_area(
                center(opaque(ui::modals::settings_modal(app))).style(|_| iced::widget::container::Style {
                    background: Some(ui::styles::MODAL_BACKDROP.into()),
                    ..Default::default()
                }),
            )
            .on_press(Message::ToggleSettings),
        );

        stack![base, settings_layer].into()
    } else {
        base
    }
}

fn subscription(app: &App) -> Subscription<Message> {
    let tick = iced::Subscription::run_with(TimerId, |_| {
        let (mut tx, rx) = futures_channel::mpsc::channel(1);
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if tx.try_send(()).is_err() && tx.is_closed() {
                    break;
                }
            }
        });
        iced::futures::stream::unfold(rx, |mut rx| async move {
            use iced::futures::StreamExt;
            let _ = rx.next().await?;
            Some((Message::Tick(time::OffsetDateTime::now_utc()), rx))
        })
    });

    let events = iced::event::listen_with(|event, status, id| match event {
        iced::Event::Window(iced::window::Event::CloseRequested) => Some(Message::HideToTray(id)),
        iced::Event::Window(iced::window::Event::Closed) => Some(Message::WindowClosed(id)),
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. })
            if status == iced::event::Status::Ignored =>
        {
            match key {
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab) => Some(if modifiers.shift() {
                    Message::FocusPrevious
                } else {
                    Message::FocusNext
                }),
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::EscapePressed),
                _ => None,
            }
        }
        _ => None,
    });

    let mut subs = vec![tick, events];

    if let Some(rx_arc) = &app.tray_rx {
        let tray_sub = iced::Subscription::run_with(
            TrayReceiver(rx_arc.clone()),
            |receiver| -> std::pin::Pin<Box<dyn iced::futures::Stream<Item = Message> + Send>> {
                if let Ok(mut opt) = receiver.0.lock() {
                    if let Some(rx) = opt.take() {
                        return Box::pin(iced::futures::stream::unfold(rx, |mut rx| async move {
                            use iced::futures::StreamExt;
                            let event = rx.next().await?;
                            Some((Message::TrayEvent(event), rx))
                        }));
                    }
                }
                Box::pin(iced::futures::stream::empty())
            },
        );
        subs.push(tray_sub);
    }

    Subscription::batch(subs)
}

fn theme(_app: &App, _id: iced::window::Id) -> iced::Theme {
    iced::Theme::custom(
        String::from("Athan"),
        iced::theme::Palette {
            background: ui::styles::BG,
            text: ui::styles::TEXT_PRIMARY,
            primary: ui::styles::ACCENT,
            success: ui::styles::ACCENT,
            danger: ui::styles::ERROR,
            warning: ui::styles::ACCENT,
        },
    )
}

fn new() -> (App, Task<Message>) {
    let mut app = App::default();
    let minimized = std::env::args().any(|a| a == "--minimized");

    crate::audio::ensure_audio_files();

    if let Some(cfg) = config::load() {
        app.apply_config(cfg);
    }

    if let Some((tray, tray_rx)) = tray::spawn("Athan") {
        app.tray = Some(tray);
        app.tray_rx = Some(Arc::new(std::sync::Mutex::new(Some(tray_rx))));
    }

    if minimized {
        (app, Task::done(Message::Tick(time::OffsetDateTime::now_utc())))
    } else {
        let (id, task) = iced::window::open(app::App::main_window_settings());
        app.window_id = Some(id);
        (app, task.map(|_| Message::Tick(time::OffsetDateTime::now_utc())))
    }
}

fn main() {
    let _minimized = std::env::args().any(|a| a == "--minimized");

    #[cfg(feature = "cli")]
    if !_minimized && std::env::args().len() > 1 {
        if let Err(e) = athan::cli::run() {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
        return;
    }
    launch_gui().expect("GUI error");
}

fn launch_gui() -> iced::Result {
    iced::daemon(new, update, view)
        .subscription(subscription)
        .theme(theme)
        .run()
}
