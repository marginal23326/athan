pub static DATE_FMT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[weekday], [month repr:long] [day], [year]");

pub fn format_duration(d: time::Duration) -> String {
    let secs = d.whole_seconds().max(0);
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

pub fn format_time(t: time::Time, use_24h: bool) -> String {
    let (h, m, _) = t.as_hms();
    if use_24h {
        format!("{:02}:{:02}", h, m)
    } else {
        let ampm = if h < 12 { "AM" } else { "PM" };
        let h12 = if h == 0 {
            12
        } else if h > 12 {
            h - 12
        } else {
            h
        };
        format!("{}:{:02} {}", h12, m, ampm)
    }
}
