use icu_calendar::Date;
use icu_calendar::cal::Hijri;

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

    pub fn from_gregorian(date: time::Date) -> Self {
        let iso = match Date::try_new_iso(date.year(), date.month() as u8, date.day()) {
            Ok(d) => d,
            Err(_) => {
                return HijriDate {
                    year: 0,
                    month: 1,
                    day: 1,
                };
            }
        };
        let hijri_cal = Hijri::new_umm_al_qura();
        let hijri = iso.to_calendar(hijri_cal);
        HijriDate {
            year: hijri.era_year().year,
            month: hijri.month().ordinal,
            day: hijri.day_of_month().0,
        }
    }

    pub fn display(&self) -> String {
        format!("{} {}, {} AH", self.month_name(), self.day, self.year)
    }

    pub fn arabic_display(&self) -> String {
        format!("{} {} {} هـ", self.day, self.arabic_month_name(), self.year)
    }
}

pub fn is_ramadan(date: time::Date) -> bool {
    let h = HijriDate::from_gregorian(date);
    h.month == 9
}

#[cfg(test)]
mod tests;
