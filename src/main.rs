mod app;
mod ui;

use app::{App, Message, new, update};
use iced::widget::{center, mouse_area, opaque, stack};
use iced::{Element, Subscription};

fn view(app: &App) -> Element<'_, Message> {
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

fn subscription(_app: &App) -> Subscription<Message> {
    let tick =
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick(time::OffsetDateTime::now_utc()));

    let keyboard_listen = iced::event::listen_with(|event, status, _id| {
        if status == iced::event::Status::Ignored
            && let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event
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
        } else {
            None
        }
    });

    Subscription::batch(vec![tick, keyboard_listen])
}

fn theme(_app: &App) -> iced::Theme {
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

fn main() {
    if std::env::args().len() > 1 {
        if let Err(e) = athan::cli::run() {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    } else {
        launch_gui().expect("GUI error");
    }
}

fn launch_gui() -> iced::Result {
    iced::application(new, update, view)
        .subscription(subscription)
        .theme(theme)
        .window_size(iced::Size::new(450.0, 640.0))
        .centered()
        .run()
}
