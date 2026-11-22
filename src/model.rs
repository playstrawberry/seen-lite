//! What `/seen` answers: online now, last seen some time ago, or never seen; and the time as words.

/// The longest name the command looks up; a longer one is no player's name.
pub const MAX_NAME: usize = 32;

const MINUTE_MS: u64 = 60_000;
const MINUTES_PER_HOUR: u64 = 60;
const MINUTES_PER_DAY: u64 = 24 * MINUTES_PER_HOUR;

/// The name typed, or `None` when it cannot be a name (empty, too long, a space or a control character).
pub fn name(raw: &str) -> Option<&str> {
    let name = raw.trim();
    let fits = !name.is_empty()
        && name.chars().count() <= MAX_NAME
        && !name.chars().any(|c| c.is_whitespace() || c.is_control());
    fits.then_some(name)
}

/// What the server knows of the player: his name as he last joined and when he last joined or left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub last_seen_ms: u64,
}

/// The answer to `/seen <name>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// Online now and the reader may see him; `for_ms` since he joined.
    Online { name: String, for_ms: u64 },
    /// Not online (or hidden from the reader); `ago_ms` since he was last seen.
    Offline { name: String, ago_ms: u64 },
    /// No player of that name ever joined this server.
    Never,
}

/// The answer from the server's record of the player and whether the reader sees him online now. A player
/// online but hidden from the reader (a staff member hidden by a staff script) is answered as offline, as
/// EssentialsX does. A clock that went back answers zero, never a time in the future.
pub fn answer(record: Option<Record>, online_and_visible: bool, now_ms: u64) -> Answer {
    let Some(record) = record else {
        return Answer::Never;
    };
    let since = now_ms.saturating_sub(record.last_seen_ms);
    if online_and_visible {
        Answer::Online {
            name: record.name,
            for_ms: since,
        }
    } else {
        Answer::Offline {
            name: record.name,
            ago_ms: since,
        }
    }
}

/// One part of a time in words: the lang key (`day`, `days`, `hour`, ...) and its number.
pub type Part = (&'static str, u64);

fn part(count: u64, one: &'static str, many: &'static str) -> Part {
    (if count == 1 { one } else { many }, count)
}

/// A time span as at most its two largest units (days and hours, hours and minutes, or minutes), the
/// smaller one left out when it is zero; under a minute is `moment`.
pub fn span(ms: u64) -> Vec<Part> {
    let minutes = ms / MINUTE_MS;
    let days = minutes / MINUTES_PER_DAY;
    let hours = (minutes / MINUTES_PER_HOUR) % 24;
    let rest = minutes % MINUTES_PER_HOUR;
    let mut parts = Vec::with_capacity(2);
    if days > 0 {
        parts.push(part(days, "day", "days"));
        if hours > 0 {
            parts.push(part(hours, "hour", "hours"));
        }
    } else if hours > 0 {
        parts.push(part(hours, "hour", "hours"));
        if rest > 0 {
            parts.push(part(rest, "minute", "minutes"));
        }
    } else if rest > 0 {
        parts.push(part(rest, "minute", "minutes"));
    } else {
        parts.push(("moment", 0));
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: u64 = MINUTE_MS;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;

    #[test]
    fn a_name_is_trimmed_and_what_cannot_be_a_name_is_refused() {
        assert_eq!(name("  Steve "), Some("Steve"));
        assert_eq!(name("Alex_2"), Some("Alex_2"));
        assert_eq!(name(""), None);
        assert_eq!(name("   "), None);
        assert_eq!(name("two words"), None);
        assert_eq!(name("bad\u{7}"), None);
        assert_eq!(
            name(&"a".repeat(MAX_NAME)),
            Some("a".repeat(MAX_NAME).as_str())
        );
        assert_eq!(name(&"a".repeat(MAX_NAME + 1)), None);
    }

    #[test]
    fn a_time_is_said_in_its_two_largest_units() {
        assert_eq!(span(0), vec![("moment", 0)]);
        assert_eq!(span(59 * 1000), vec![("moment", 0)]);
        assert_eq!(span(MIN), vec![("minute", 1)]);
        assert_eq!(span(42 * MIN + 30 * 1000), vec![("minutes", 42)]);
        assert_eq!(span(HOUR), vec![("hour", 1)]);
        assert_eq!(span(3 * HOUR + MIN), vec![("hours", 3), ("minute", 1)]);
        assert_eq!(span(DAY), vec![("day", 1)]);
        assert_eq!(
            span(2 * DAY + 5 * HOUR + 59 * MIN),
            vec![("days", 2), ("hours", 5)]
        );
        assert_eq!(span(400 * DAY + 59 * MIN), vec![("days", 400)]);
    }

    fn steve(last_seen_ms: u64) -> Option<Record> {
        Some(Record {
            name: "Steve".into(),
            last_seen_ms,
        })
    }

    #[test]
    fn an_online_player_is_online_since_he_joined() {
        assert_eq!(
            answer(steve(1_000), true, 1_000 + 2 * HOUR),
            Answer::Online {
                name: "Steve".into(),
                for_ms: 2 * HOUR
            }
        );
    }

    #[test]
    fn an_offline_or_hidden_player_is_last_seen_and_an_unknown_name_never() {
        let offline = Answer::Offline {
            name: "Steve".into(),
            ago_ms: 3 * DAY,
        };
        assert_eq!(answer(steve(5), false, 5 + 3 * DAY), offline);
        assert_eq!(answer(None, false, 10), Answer::Never);
        assert_eq!(answer(None, true, 10), Answer::Never);
    }

    #[test]
    fn a_clock_that_went_back_answers_zero() {
        assert_eq!(
            answer(steve(10_000), false, 5_000),
            Answer::Offline {
                name: "Steve".into(),
                ago_ms: 0
            }
        );
    }
}
