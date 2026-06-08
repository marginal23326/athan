use tokio::sync::mpsc as tokio_mpsc;

#[derive(Debug, Clone, Copy)]
pub enum TrayEvent {
    Clicked,
    Exit,
}

pub struct TrayHandle {
    #[cfg(target_os = "linux")]
    update_tx: tokio_mpsc::UnboundedSender<String>,
    #[cfg(target_os = "windows")]
    tray_icon: tray_icon::TrayIcon,
}

#[cfg(target_os = "linux")]
struct AthanTray {
    tooltip: String,
    tx: tokio_mpsc::UnboundedSender<TrayEvent>,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for AthanTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }
    fn icon_name(&self) -> String {
        "dialog-information".into()
    }
    fn title(&self) -> String {
        "Athan".into()
    }
    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "Athan".into(),
            description: self.tooltip.clone(),
            ..Default::default()
        }
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayEvent::Clicked);
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Open".into(),
                activate: Box::new(|this: &mut AthanTray| {
                    let _ = this.tx.send(TrayEvent::Clicked);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(|this: &mut AthanTray| {
                    let _ = this.tx.send(TrayEvent::Exit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn spawn(initial_tooltip: &str) -> Option<(TrayHandle, tokio_mpsc::UnboundedReceiver<TrayEvent>)> {
    let (tx, rx) = tokio_mpsc::unbounded_channel();

    #[cfg(target_os = "linux")]
    {
        let (update_tx, mut update_rx) = tokio_mpsc::unbounded_channel::<String>();
        let tooltip = initial_tooltip.to_string();

        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                rt.block_on(async move {
                    use ksni::TrayMethods;
                    let tray = AthanTray { tooltip, tx };
                    if let Ok(handle) = tray.spawn().await {
                        while let Some(new_tooltip) = update_rx.recv().await {
                            let _ = handle.update(|tray: &mut AthanTray| tray.tooltip = new_tooltip).await;
                        }
                    }
                });
            }
        });

        return Some((TrayHandle { update_tx }, rx));
    }

    #[cfg(target_os = "windows")]
    {
        use tray_icon::TrayIconBuilder;
        use tray_icon::menu::{Menu, MenuItem};

        let menu = Menu::new();
        let open_item = MenuItem::new("Open", true, None);
        let exit_item = MenuItem::new("Exit", true, None);
        let _ = menu.append(&open_item);
        let _ = menu.append(&exit_item);

        let width = 32;
        let height = 32;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            rgba.extend_from_slice(&[0, 128, 255, 255]);
        }
        let icon = tray_icon::Icon::from_rgba(rgba, width, height).ok()?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(initial_tooltip)
            .with_icon(icon)
            .build()
            .ok()?;

        let open_id = open_item.id().clone();
        let exit_id = exit_item.id().clone();

        let tx_click = tx.clone();
        tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
            if let tray_icon::TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = event
            {
                let _ = tx_click.send(TrayEvent::Clicked);
            }
        }));

        let tx_menu = tx.clone();
        tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
            if event.id == open_id {
                let _ = tx_menu.send(TrayEvent::Clicked);
            } else if event.id == exit_id {
                let _ = tx_menu.send(TrayEvent::Exit);
            }
        }));

        return Some((TrayHandle { tray_icon }, rx));
    }

    #[allow(unreachable_code)]
    None
}

impl TrayHandle {
    pub fn update_tooltip(&self, text: &str) {
        #[cfg(target_os = "linux")]
        {
            let _ = self.update_tx.send(text.to_string());
        }
        #[cfg(target_os = "windows")]
        {
            let _ = self.tray_icon.set_tooltip(Some(text));
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = text;
        }
    }
}
