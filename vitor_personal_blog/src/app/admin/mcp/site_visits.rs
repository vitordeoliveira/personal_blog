//! The `site_visits` tool: the site's visits in a period, as the admin shows them, next
//! to the period before it.

use jiff::{Timestamp, ToSpan, civil::Date};
use serde_json::{Value, json};
use topcoat::context::Cx;

use super::common::{PeriodArguments, ToolError, parse, period, period_schema};
use crate::{
    db::db,
    features::analytics::{self, Report},
};

/// The name agents call the tool by.
pub const NAME: &str = "site_visits";

/// How many visits the tool lists, the latest ones.
const VISITS_LISTED: usize = 50;

/// What the tool does, for the agent deciding whether to call it.
const DESCRIPTION: &str = "Reports the visits to the blog, https://vitor.ws, in a \
    period, as its admin shows them: the last days, today included, or the days from \
    one date to another. The site counts them itself, without cookies: a visitor is a \
    person on one day, so someone who comes on two days counts twice, and programs such \
    as search engines are left out. It gives the totals and those of the period of the \
    same length just before, to tell whether the blog grows; each day; each hour of the \
    day, summed over the period; the 10 pages most opened; the 10 sources, devices and \
    browsers with the most visitors; the 20 campaigns, by the utm_source, utm_medium, \
    utm_campaign and utm_content of the first tagged link a visitor followed, such as \
    x, bio; and the 50 latest visits, each with when it started and ended, its source, \
    campaign tags, device, browser and pages in order. A source is who sent the \
    visitor, by their first page, such as X (Twitter), LinkedIn, Hacker News or another \
    site's host; when the browser did not say, the utm_source of that page's link; and \
    otherwise direct, for a typed address, a bookmark or an app that does not say. The \
    cookie part counts only the visitors who accepted the cookie: how many browsers, \
    and how many of them came back on another day. counting_since is the day of the \
    oldest visit kept: there are none before it, so a period that starts earlier is not \
    counted in full. Visits before 2026-11-12 kept only their utm_source. Days and \
    times are in Brasília time, given as time_zone. Visits are kept for 400 days.";

/// The tool's name, description and arguments, for `tools/list`.
pub fn definition() -> Value {
    json!({
        "name": NAME,
        "title": "Report the blog's visits",
        "description": DESCRIPTION,
        "inputSchema": period_schema(),
        "annotations": { "readOnlyHint": true, "openWorldHint": false },
    })
}

/// The visitors and page views of the period before the one reported.
struct Previous {
    first: Date,
    last: Date,
    visitors: usize,
    views: usize,
}

/// Reports the visits of the period the arguments ask for.
pub async fn call(cx: &Cx, arguments: Value) -> Result<Value, ToolError> {
    let arguments: PeriodArguments = parse(arguments)?;
    let (first, last) = period(&arguments, analytics::today(), analytics::earliest_day())?;
    let db = db(cx);
    let report = Report::between(&db, first, last).await?;
    let length = report.days.len() as i64;
    let before = first.yesterday().unwrap_or(first);
    let start = before.checked_sub((length - 1).days()).unwrap_or(before);
    let (visitors, views) = analytics::totals_between(&db, start, before).await?;
    let previous = Previous {
        first: start,
        last: before,
        visitors,
        views,
    };
    let since = analytics::counting_since(&db).await?;
    Ok(visits(&report, &previous, since))
}

/// A moment as the tool gives it, in the report's time zone, such as
/// `2026-11-12T14:37:00-03:00`.
fn time(at: Timestamp) -> String {
    at.to_zoned(analytics::time_zone())
        .strftime("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

/// The report as the tool gives it.
fn visits(report: &Report, previous: &Previous, since: Option<Date>) -> Value {
    let each = |rows: &[(String, usize)], key: &str| -> Vec<Value> {
        rows.iter()
            .map(|(value, visitors)| json!({ key: value, "visitors": visitors }))
            .collect()
    };
    let each_day: Vec<_> = report
        .days
        .iter()
        .map(|day| json!({ "date": day.date.to_string(), "views": day.views, "visitors": day.visitors }))
        .collect();
    let each_hour: Vec<_> = report
        .hours
        .iter()
        .map(|hour| json!({ "hour": hour.hour, "views": hour.views, "visitors": hour.visitors }))
        .collect();
    let pages: Vec<_> = report
        .pages
        .iter()
        .map(|page| json!({ "path": page.path, "views": page.views, "visitors": page.visitors }))
        .collect();
    let sources: Vec<_> = report
        .sources
        .iter()
        .map(|source| json!({ "source": source.name.as_deref().unwrap_or("direct"), "visitors": source.visitors }))
        .collect();
    let campaigns: Vec<_> = report
        .campaigns
        .iter()
        .map(|campaign| {
            json!({
                "utm_source": campaign.source,
                "utm_medium": campaign.medium,
                "utm_campaign": campaign.name,
                "utm_content": campaign.content,
                "visitors": campaign.visitors,
            })
        })
        .collect();
    let latest: Vec<_> = report
        .visits
        .iter()
        .take(VISITS_LISTED)
        .map(|visit| {
            json!({
                "started_at": time(visit.start),
                "ended_at": time(visit.end),
                "source": visit.source.as_deref().unwrap_or("direct"),
                "campaign": visit.campaign,
                "device": visit.device,
                "browser": visit.browser,
                "pages": visit.pages,
            })
        })
        .collect();
    let counted_from = |first: Date| since.is_some_and(|since| since <= first);
    json!({
        "from": report.first.to_string(),
        "to": report.last.to_string(),
        "days": report.days.len(),
        "time_zone": report.time_zone,
        "counting_since": since.map(|since| since.to_string()),
        "counted_in_full": counted_from(report.first),
        "visitors": report.visitors,
        "views": report.views,
        "previous_period": {
            "from": previous.first.to_string(),
            "to": previous.last.to_string(),
            "counted_in_full": counted_from(previous.first),
            "visitors": previous.visitors,
            "views": previous.views,
        },
        "each_day": each_day,
        "each_hour": each_hour,
        "pages": pages,
        "sources": sources,
        "devices": each(&report.devices, "device"),
        "browsers": each(&report.browsers, "browser"),
        "campaigns": campaigns,
        "latest_visits": latest,
        "cookie": {
            "visitors_who_accepted": report.accepted.visitors,
            "browsers": report.accepted.browsers,
            "browsers_that_came_back": report.accepted.returning,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::analytics::{Accepted, Campaign, Day, Hour, Row, Source, Visit};

    #[test]
    fn reports_every_part_of_the_admin_and_the_period_before() {
        let day = |text: &str| text.parse::<Date>().unwrap();
        let at: Timestamp = "2026-11-05T17:37:00Z".parse().unwrap();
        let report = Report {
            first: day("2026-11-05"),
            last: day("2026-11-05"),
            days: vec![Day {
                date: day("2026-11-05"),
                views: 12,
                visitors: 5,
            }],
            hours: vec![Hour {
                hour: 14,
                views: 12,
                visitors: 5,
            }],
            visitors: 5,
            views: 12,
            pages: vec![Row {
                path: "/blog/how_use_axum_askama_htmx".to_owned(),
                views: 8,
                visitors: 5,
            }],
            sources: vec![
                Source {
                    name: Some("Hacker News".to_owned()),
                    visitors: 3,
                },
                Source {
                    name: None,
                    visitors: 2,
                },
            ],
            devices: vec![("Desktop".to_owned(), 4)],
            browsers: vec![("Firefox".to_owned(), 3)],
            campaigns: vec![Campaign {
                source: "x".to_owned(),
                medium: Some("bio".to_owned()),
                name: None,
                content: None,
                visitors: 3,
            }],
            visits: vec![Visit {
                start: at,
                end: at,
                pages: vec!["/".to_owned(), "/blog".to_owned()],
                source: Some("X (Twitter)".to_owned()),
                campaign: Some("x · bio".to_owned()),
                device: Some("Mobile".to_owned()),
                browser: Some("Safari".to_owned()),
            }],
            accepted: Accepted {
                browsers: 2,
                returning: 1,
                visitors: 2,
            },
            time_zone: "America/Sao_Paulo".to_owned(),
        };
        let previous = Previous {
            first: day("2026-11-04"),
            last: day("2026-11-04"),
            visitors: 3,
            views: 4,
        };
        let visits = visits(&report, &previous, Some(day("2026-11-05")));
        assert_eq!(visits["from"], "2026-11-05");
        assert_eq!(visits["counting_since"], "2026-11-05");
        assert_eq!(visits["counted_in_full"], true);
        assert_eq!(visits["previous_period"]["to"], "2026-11-04");
        assert_eq!(visits["previous_period"]["visitors"], 3);
        assert_eq!(visits["previous_period"]["counted_in_full"], false);
        assert_eq!(visits["each_day"][0]["date"], "2026-11-05");
        assert_eq!(visits["each_hour"][0]["hour"], 14);
        assert_eq!(visits["pages"][0]["path"], "/blog/how_use_axum_askama_htmx");
        assert_eq!(visits["sources"][0]["source"], "Hacker News");
        assert_eq!(visits["sources"][1]["source"], "direct");
        assert_eq!(visits["campaigns"][0]["utm_source"], "x");
        assert_eq!(visits["campaigns"][0]["utm_medium"], "bio");
        assert_eq!(visits["campaigns"][0]["utm_content"], Value::Null);
        assert_eq!(visits["latest_visits"][0]["started_at"], "2026-11-05T14:37:00-03:00");
        assert_eq!(visits["latest_visits"][0]["campaign"], "x · bio");
        assert_eq!(visits["cookie"]["browsers_that_came_back"], 1);

        let nothing_counted = super::visits(&report, &previous, None);
        assert_eq!(nothing_counted["counting_since"], Value::Null);
        assert_eq!(nothing_counted["counted_in_full"], false);
    }
}
