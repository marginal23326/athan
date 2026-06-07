#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HijriDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

const ISLAMIC_MONTH_NAMES: &[&str; 12] = &[
    "Muharram",
    "Safar",
    "Rabi' I",
    "Rabi' II",
    "Jumada I",
    "Jumada II",
    "Rajab",
    "Sha'ban",
    "Ramadan",
    "Shawwal",
    "Dhu al-Qi'dah",
    "Dhu al-Hijjah",
];

const ARABIC_MONTH_NAMES: &[&str; 12] = &[
    "محرم",
    "صفر",
    "ربيع الأول",
    "ربيع الآخر",
    "جمادى الأولى",
    "جمادى الآخرة",
    "رجب",
    "شعبان",
    "رمضان",
    "شوال",
    "ذو القعدة",
    "ذو الحجة",
];

impl HijriDate {
    pub fn month_name(&self) -> &'static str {
        self.month
            .checked_sub(1)
            .and_then(|month| ISLAMIC_MONTH_NAMES.get(month as usize))
            .unwrap_or(&"Unknown")
    }

    pub fn arabic_month_name(&self) -> &'static str {
        self.month
            .checked_sub(1)
            .and_then(|month| ARABIC_MONTH_NAMES.get(month as usize))
            .unwrap_or(&"")
    }

    #[cfg(feature = "hijri")]
    pub fn from_gregorian(date: time::Date) -> Option<Self> {
        use icu_calendar::Date;
        use icu_calendar::cal::Hijri;

        let iso = Date::try_new_iso(date.year(), date.month() as u8, date.day()).ok()?;
        let hijri_cal = Hijri::new_umm_al_qura();
        let hijri = iso.to_calendar(hijri_cal);
        Some(HijriDate {
            year: hijri.era_year().year,
            month: hijri.month().ordinal,
            day: hijri.day_of_month().0,
        })
    }

    pub fn display(&self) -> String {
        format!("{} {}, {} AH", self.month_name(), self.day, self.year)
    }

    pub fn arabic_display(&self) -> String {
        format!("{} {} {} هـ", self.day, self.arabic_month_name(), self.year)
    }
}

#[cfg(feature = "hijri")]
pub fn is_ramadan(date: time::Date) -> bool {
    HijriDate::from_gregorian(date).map(|h| h.month == 9).unwrap_or(false)
}

#[cfg(not(feature = "hijri"))]
pub fn is_ramadan(date: time::Date) -> bool {
    let (y, m, d) = (date.year(), date.month() as i32, date.day() as i32);

    let a = (m <= 2) as i32;
    let y4 = y + 4800 - a;
    let m4 = m + 12 * a - 3;
    let jd = d + (153 * m4 + 2) / 5 + 365 * y4 + y4 / 4 - y4 / 100 + y4 / 400 - 32045;

    let cycle = jd - 1937808;
    let n = (cycle - 1) / 10631;
    let cycle = cycle - 10631 * n + 354;

    let year_in_cycle = (30 * cycle - 4) / 10631;
    let ramadan_start = (10631 * year_in_cycle + 7113) / 30;

    cycle >= ramadan_start && cycle < ramadan_start + 30
}

#[cfg(test)]
mod tests;
