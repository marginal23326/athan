use futures_channel::mpsc;
use std::sync::OnceLock;

pub fn cached_icon_rgba_64() -> &'static [u8] {
    static ICON: OnceLock<Vec<u8>> = OnceLock::new();
    ICON.get_or_init(|| generate_icon(64))
}

#[derive(Debug, Clone, Copy)]
pub enum TrayEvent {
    ToggleWindow,
    Exit,
    StopAdhan,
}

pub struct TrayHandle {
    #[cfg(target_os = "linux")]
    update_tx: mpsc::UnboundedSender<(String, bool, bool)>,
    #[cfg(target_os = "windows")]
    tray_icon: tray_icon::TrayIcon,
    #[cfg(target_os = "windows")]
    stop_item: tray_icon::menu::MenuItem,
    #[cfg(target_os = "windows")]
    toggle_item: tray_icon::menu::MenuItem,
}

#[cfg(target_os = "linux")]
fn make_icon() -> ksni::Icon {
    let size = 64;
    let rgba = cached_icon_rgba_64().to_vec();
    let mut argb = Vec::with_capacity(rgba.len());
    for chunk in rgba.chunks_exact(4) {
        argb.push(chunk[3]); // Alpha
        argb.push(chunk[0]); // Red
        argb.push(chunk[1]); // Green
        argb.push(chunk[2]); // Blue
    }
    ksni::Icon {
        width: size as i32,
        height: size as i32,
        data: argb,
    }
}

pub fn generate_icon(size: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let scale = 1.2_f64;
    let px_size = 2.0 / (size as f64) / scale;

    let charcoal_r = 24.0;
    let charcoal_g = 24.0;
    let charcoal_b = 27.0;

    let gold_r = 234.0;
    let gold_g = 179.0;
    let gold_b = 8.0;

    let box_w = 0.65;
    let box_h = 0.65;
    let r = 0.16;

    let c1 = -0.22;
    let h1 = 0.075;
    let c2 = -0.01;
    let h2 = 0.025;

    for y in 0..size {
        for x in 0..size {
            let u = (((x as f64) + 0.5) / (size as f64) * 2.0 - 1.0) / scale;
            let v = (((y as f64) + 0.5) / (size as f64) * 2.0 - 1.0) / scale;

            let dx = u.abs() - (box_w - r);
            let dy = v.abs() - (box_h - r);

            let mx = dx.max(0.0);
            let my = dy.max(0.0);
            let length = mx.hypot(my);
            let inside_dist = dx.max(dy).min(0.0);
            let d_box = length + inside_dist - r;

            let alpha = if d_box < -px_size {
                1.0
            } else if d_box > px_size {
                0.0
            } else {
                0.5 - 0.5 * (d_box / px_size)
            };

            if alpha > 0.0 {
                let d_band1 = (v - c1).abs() - h1;
                let d_band2 = (v - c2).abs() - h2;
                let d_gold = d_band1.min(d_band2);

                let gold_factor = if d_gold < -px_size {
                    1.0
                } else if d_gold > px_size {
                    0.0
                } else {
                    0.5 - 0.5 * (d_gold / px_size)
                };

                let blended_r = charcoal_r * (1.0 - gold_factor) + gold_r * gold_factor;
                let blended_g = charcoal_g * (1.0 - gold_factor) + gold_g * gold_factor;
                let blended_b = charcoal_b * (1.0 - gold_factor) + gold_b * gold_factor;

                rgba.push(blended_r.round() as u8);
                rgba.push(blended_g.round() as u8);
                rgba.push(blended_b.round() as u8);
                rgba.push((alpha * 255.0).round() as u8);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    rgba
}

#[cfg(target_os = "linux")]
struct AthanTray {
    tooltip: String,
    playing: bool,
    is_window_open: bool,
    icon: Vec<ksni::Icon>,
    tx: mpsc::UnboundedSender<TrayEvent>,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for AthanTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
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
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.icon.clone()
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.unbounded_send(TrayEvent::ToggleWindow);
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let toggle_label = if self.is_window_open { "Hide" } else { "Show" };
        let mut items = vec![
            StandardItem {
                label: toggle_label.into(),
                activate: Box::new(|this: &mut AthanTray| {
                    let _ = this.tx.unbounded_send(TrayEvent::ToggleWindow);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
        ];

        if self.playing {
            items.push(
                StandardItem {
                    label: "Stop Adhan".into(),
                    activate: Box::new(|this: &mut AthanTray| {
                        let _ = this.tx.unbounded_send(TrayEvent::StopAdhan);
                    }),
                    ..Default::default()
                }
                .into(),
            );
            items.push(MenuItem::Separator);
        }

        items.push(
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(|this: &mut AthanTray| {
                    let _ = this.tx.unbounded_send(TrayEvent::Exit);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

pub fn spawn(initial_tooltip: &str) -> Option<(TrayHandle, mpsc::UnboundedReceiver<TrayEvent>)> {
    let (tx, rx) = mpsc::unbounded();

    #[cfg(target_os = "linux")]
    {
        let (update_tx, mut update_rx) = mpsc::unbounded::<(String, bool, bool)>();
        let tooltip = initial_tooltip.to_string();

        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                rt.block_on(async move {
                    use iced::futures::StreamExt;
                    use ksni::TrayMethods;
                    let tray = AthanTray {
                        tooltip,
                        playing: false,
                        is_window_open: false,
                        icon: vec![make_icon()],
                        tx,
                    };
                    if let Ok(handle) = tray.spawn().await {
                        while let Some((new_tooltip, playing, is_window_open)) = update_rx.next().await {
                            let _ = handle
                                .update(|tray: &mut AthanTray| {
                                    tray.tooltip = new_tooltip;
                                    tray.playing = playing;
                                    tray.is_window_open = is_window_open;
                                })
                                .await;
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
        use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};

        let menu = Menu::new();
        let toggle_item = MenuItem::new("Show", true, None);
        let stop_item = MenuItem::new("Stop Adhan", false, None);
        let exit_item = MenuItem::new("Exit", true, None);
        let _ = menu.append(&toggle_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&stop_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&exit_item);

        let size = 64;
        let rgba = cached_icon_rgba_64().to_vec();
        let icon = tray_icon::Icon::from_rgba(rgba, size, size).ok()?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip(initial_tooltip)
            .with_icon(icon)
            .build()
            .ok()?;

        let toggle_id = toggle_item.id().clone();
        let stop_id = stop_item.id().clone();
        let exit_id = exit_item.id().clone();

        let tx_click = tx.clone();
        tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
            if let tray_icon::TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = event
            {
                let _ = tx_click.unbounded_send(TrayEvent::ToggleWindow);
            }
        }));

        let tx_menu = tx.clone();
        tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
            if event.id == toggle_id {
                let _ = tx_menu.unbounded_send(TrayEvent::ToggleWindow);
            } else if event.id == stop_id {
                let _ = tx_menu.unbounded_send(TrayEvent::StopAdhan);
            } else if event.id == exit_id {
                let _ = tx_menu.unbounded_send(TrayEvent::Exit);
            }
        }));

        return Some((TrayHandle { tray_icon, stop_item, toggle_item }, rx));
    }

    #[allow(unreachable_code)]
    None
}

impl TrayHandle {
    pub fn update(&self, text: &str, playing: bool, is_window_open: bool) {
        #[cfg(target_os = "linux")]
        {
            let _ = self.update_tx.unbounded_send((text.to_string(), playing, is_window_open));
        }
        #[cfg(target_os = "windows")]
        {
            let _ = self.tray_icon.set_tooltip(Some(text));
            self.stop_item.set_enabled(playing);
            self.toggle_item.set_text(if is_window_open { "Hide" } else { "Show" });
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = text;
            let _ = playing;
            let _ = is_window_open;
        }
    }
}
