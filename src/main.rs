mod app;
mod config;
mod tray;
mod ui;

use app::{App, Message, update};
use iced::widget::{center, mouse_area, opaque, stack};
use iced::{Element, Subscription, Task};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct TrayReceiver(Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<crate::tray::TrayEvent>>>);

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

        if app.adjustments_open {
            stack![
                base,
                settings_layer,
                opaque(
                    mouse_area(center(opaque(ui::modals::adjustments_modal(app))).style(|_| {
                        iced::widget::container::Style {
                            background: Some(ui::styles::MODAL_BACKDROP.into()),
                            ..Default::default()
                        }
                    }))
                    .on_press(Message::ToggleAdjustments)
                )
            ]
            .into()
        } else {
            stack![base, settings_layer].into()
        }
    } else {
        base
    }
}

fn subscription(app: &App) -> Subscription<Message> {
    let tick =
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick(time::OffsetDateTime::now_utc()));

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
        let tray_sub = iced::Subscription::run_with(TrayReceiver(rx_arc.clone()), |receiver| {
            let rx = receiver.0.clone();
            iced::futures::stream::unfold(rx, |rx| async move {
                let mut lock = rx.lock().await;
                let event = lock.recv().await?;
                Some((Message::TrayEvent(event), rx.clone()))
            })
        });
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

    if let Some(cfg) = config::load() {
        app.apply_config(cfg);
    }

    if let Some((tray, tray_rx)) = tray::spawn("Athan") {
        app.tray = Some(tray);
        app.tray_rx = Some(Arc::new(Mutex::new(tray_rx)));
    }

    let (id, task) = iced::window::open(app::App::main_window_settings());
    app.window_id = Some(id);

    (app, task.map(|_| Message::Tick(time::OffsetDateTime::now_utc())))
}

fn main() {
    #[cfg(feature = "cli")]
    if std::env::args().len() > 1 {
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
