//! What the tools share: how a call fails, how its arguments are read, and the period a
//! report covers.

use jiff::{ToSpan, civil::Date};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::features::analytics::KEPT_DAYS;

/// Why a tool call did not go through.
#[derive(Debug)]
pub enum ToolError {
    /// The call asks for something the tool cannot do, such as a post that does not
    /// exist. The agent reads why and can correct the call.
    Refused(String),
    /// The database or the posts' files failed.
    Failed(topcoat::Error),
}

impl From<topcoat::Error> for ToolError {
    fn from(error: topcoat::Error) -> Self {
        ToolError::Failed(error)
    }
}

impl From<toasty::Error> for ToolError {
    fn from(error: toasty::Error) -> Self {
        ToolError::Failed(error.into())
    }
}

/// A refusal that says why.
pub fn refuse(why: impl Into<String>) -> ToolError {
    ToolError::Refused(why.into())
}

/// Reads a tool's arguments. They refuse unknown fields, so a misspelled one is never
/// silently ignored.
pub fn parse<T: DeserializeOwned>(arguments: Value) -> Result<T, ToolError> {
    serde_json::from_value(arguments)
        .map_err(|error| refuse(format!("Invalid arguments: {error}.")))
}

/// The days a period covers when the agent asks for none.
const DEFAULT_DAYS: i64 = 30;

/// The arguments of a tool that reports a period: the last days, or from one day to
/// another.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeriodArguments {
    days: Option<i64>,
    from: Option<String>,
    to: Option<String>,
}

/// The schema of [`PeriodArguments`], for `tools/list`.
pub fn period_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "days": {
                "type": "integer",
                "minimum": 1,
                "maximum": KEPT_DAYS,
                "default": DEFAULT_DAYS,
                "description": "How many days to report, today included. Leave it out \
                    when giving from.",
            },
            "from": {
                "type": "string",
                "description": "The first day to report, such as 2026-11-01, in place \
                    of days.",
            },
            "to": {
                "type": "string",
                "description": "The last day to report, such as 2026-11-07, with from. \
                    Defaults to from, for a single day.",
            },
        },
        "additionalProperties": false,
    })
}

/// The first and last days the arguments ask for, or why they cannot be reported. A
/// period that starts before `earliest`, the oldest day whose visits are kept, starts
/// there.
pub fn period(
    arguments: &PeriodArguments,
    today: Date,
    earliest: Date,
) -> Result<(Date, Date), ToolError> {
    let day = |name: &str, text: &str| {
        text.parse::<Date>().map_err(|_| {
            refuse(format!(
                "{name} must be a day such as 2026-11-05, not {text}."
            ))
        })
    };
    let (first, last) = match (
        arguments.days,
        arguments.from.as_deref(),
        arguments.to.as_deref(),
    ) {
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            return Err(refuse("Give days, or from and to, not both."));
        }
        (None, None, Some(_)) => return Err(refuse("Give from along with to.")),
        (days, None, None) => {
            let days = days.unwrap_or(DEFAULT_DAYS);
            if !(1..=KEPT_DAYS).contains(&days) {
                return Err(refuse(format!(
                    "days must be from 1 to {KEPT_DAYS}, not {days}."
                )));
            }
            (today.checked_sub((days - 1).days()).unwrap_or(today), today)
        }
        (None, Some(from), to) => {
            let first = day("from", from)?;
            let last = to.map(|to| day("to", to)).transpose()?.unwrap_or(first);
            if first > last {
                return Err(refuse(format!("from, {first}, comes after to, {last}.")));
            }
            (first, last)
        }
    };
    if last > today {
        return Err(refuse(format!(
            "The period cannot end after today, {today}."
        )));
    }
    Ok((first.max(earliest), last))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periods_are_the_last_days_or_from_one_day_to_another() {
        let today: Date = "2026-11-10".parse().unwrap();
        let earliest: Date = "2025-10-07".parse().unwrap();
        let ask = |value| period(&parse(value).unwrap(), today, earliest);
        let day = |text: &str| text.parse::<Date>().unwrap();

        assert_eq!(ask(json!({})).unwrap(), (day("2026-10-12"), today));
        assert_eq!(ask(json!({ "days": 1 })).unwrap(), (today, today));
        assert_eq!(
            ask(json!({ "from": "2026-11-03" })).unwrap(),
            (day("2026-11-03"), day("2026-11-03"))
        );
        assert_eq!(
            ask(json!({ "from": "2026-11-01", "to": "2026-11-07" })).unwrap(),
            (day("2026-11-01"), day("2026-11-07"))
        );
        assert_eq!(
            ask(json!({ "from": "2020-01-01", "to": "2026-01-01" }))
                .unwrap()
                .0,
            earliest
        );
        assert!(ask(json!({ "days": 7, "from": "2026-11-01" })).is_err());
        assert!(ask(json!({ "to": "2026-11-01" })).is_err());
        assert!(ask(json!({ "from": "2026-11-07", "to": "2026-11-01" })).is_err());
        assert!(ask(json!({ "from": "2026-11-11" })).is_err());
        assert!(ask(json!({ "from": "yesterday" })).is_err());
        assert!(ask(json!({ "days": 0 })).is_err());
        assert!(ask(json!({ "days": 401 })).is_err());
    }
}
