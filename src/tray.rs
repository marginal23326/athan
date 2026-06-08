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
fn make_icon() -> ksni::Icon {
    let size = 64;
    let rgba = generate_icon(size);
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

    // Charcoal: #18181b
    let charcoal_r = 24.0;
    let charcoal_g = 24.0;
    let charcoal_b = 27.0;

    // Gold: #eab308
    let gold_r = 234.0;
    let gold_g = 179.0;
    let gold_b = 8.0;

    // Rounded box geometry
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
    icon: Vec<ksni::Icon>,
    tx: tokio_mpsc::UnboundedSender<TrayEvent>,
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
                    let tray = AthanTray {
                        tooltip,
                        icon: vec![make_icon()],
                        tx,
                    };
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

        let size = 32;
        let rgba = generate_icon(size);
        let icon = tray_icon::Icon::from_rgba(rgba, size, size).ok()?;

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
