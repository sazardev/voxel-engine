//! The world list as text: name, last played time (UTC) and size, one world per line (FR-24).

use super::worlds::WorldSummary;

const SECONDS_PER_DAY: u64 = 86_400;

/// One line per world, the most recent first, or a hint when there are none.
#[must_use]
pub fn format_world_list(worlds: &[WorldSummary]) -> String {
    if worlds.is_empty() {
        return "No worlds yet. Start the game to create one.\n".to_owned();
    }
    let width = worlds
        .iter()
        .map(|world| world.name.chars().count())
        .max()
        .unwrap_or(0);
    worlds
        .iter()
        .map(|world| {
            format!(
                "{:<width$}  {}  {}\n",
                world.name,
                last_played(world.last_played_secs),
                format_size(world.size_bytes),
            )
        })
        .collect::<Vec<String>>()
        .concat()
}

fn last_played(secs: Option<u64>) -> String {
    secs.map_or_else(|| "never played".to_owned(), format_utc)
}

/// `YYYY-MM-DD HH:MM UTC` for seconds since the Unix epoch.
fn format_utc(secs: u64) -> String {
    let (year, month, day) = civil_from_days(secs / SECONDS_PER_DAY);
    let minutes = secs % SECONDS_PER_DAY / 60;
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        minutes / 60,
        minutes % 60
    )
}

/// Year, month and day for a count of days since 1970-01-01 (proleptic Gregorian calendar).
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    if bytes >= MIB {
        format!("{}.{} MB", bytes / MIB, bytes % MIB * 10 / MIB)
    } else if bytes >= KIB {
        format!("{}.{} KB", bytes / KIB, bytes % KIB * 10 / KIB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::world_id::WorldId;

    fn summary(name: &str, last_played_secs: Option<u64>, size_bytes: u64) -> WorldSummary {
        WorldSummary {
            id: WorldId::from_name(name).expect("a valid name"),
            name: name.to_owned(),
            last_played_secs,
            size_bytes,
        }
    }

    #[test]
    fn dates_are_calendar_correct_across_leap_days_and_year_ends() {
        assert_eq!(format_utc(0), "1970-01-01 00:00 UTC");
        assert_eq!(
            format_utc(951_782_400 + 3_600 + 120),
            "2000-02-29 01:02 UTC"
        );
        assert_eq!(format_utc(1_798_761_599), "2026-12-31 23:59 UTC");
        assert_eq!(format_utc(1_798_761_600), "2027-01-01 00:00 UTC");
    }

    #[test]
    fn sizes_use_the_largest_fitting_unit() {
        assert_eq!(format_size(86), "86 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(33 * 1024 * 1024 + 512 * 1024), "33.5 MB");
    }

    #[test]
    fn each_world_is_one_line_with_aligned_columns() {
        let text =
            format_world_list(&[summary("Island", Some(0), 86), summary("Home", None, 2048)]);

        assert_eq!(
            text,
            "Island  1970-01-01 00:00 UTC  86 B\nHome    never played  2.0 KB\n"
        );
    }

    #[test]
    fn no_worlds_says_how_to_make_one() {
        assert_eq!(
            format_world_list(&[]),
            "No worlds yet. Start the game to create one.\n"
        );
    }
}
