//! The admin's page, at `/admin`: a form to sign in, and once signed in, the site's
//! visits of a period: today, yesterday, the last days, or the days the admin picks.
//! `/admin/mcp` gives the visits to AI agents.

pub mod mcp;

use jiff::{Timestamp, ToSpan, civil::Date};
use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    router::{
        Body, Next,
        content::Form,
        error::{SeeOther, see_other},
        layer, page, query_params,
        response::Response,
        route,
    },
    view::{View, attributes, component, view},
};

use super::PageMeta;

use crate::components::button::{ButtonSize, ButtonVariant, button};
use crate::components::input::input;
use crate::db::db;
use crate::features::admin;
use crate::features::analytics::{self, Accepted, Campaign, Report, Source, VISITS_LISTED, Visit};

/// A period the page offers as a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Preset {
    Today,
    Yesterday,
    /// The last days, today included.
    Days(i64),
}

/// The periods the page offers as links, in their order.
const PRESETS: [Preset; 5] = [
    Preset::Today,
    Preset::Yesterday,
    Preset::Days(7),
    Preset::Days(30),
    Preset::Days(90),
];

/// The period shown when none is asked for.
const DEFAULT_PRESET: Preset = Preset::Days(30);

impl Preset {
    /// The value of the query's `period`.
    fn code(self) -> String {
        match self {
            Preset::Today => "today".to_owned(),
            Preset::Yesterday => "yesterday".to_owned(),
            Preset::Days(days) => days.to_string(),
        }
    }

    fn label(self) -> String {
        match self {
            Preset::Today => "Today".to_owned(),
            Preset::Yesterday => "Yesterday".to_owned(),
            Preset::Days(days) => format!("{days} days"),
        }
    }

    /// The period's first and last days, counted from `today`.
    fn range(self, today: Date) -> (Date, Date) {
        let back = |days: i64| today.checked_sub(days.days()).unwrap_or(today);
        match self {
            Preset::Today => (today, today),
            Preset::Yesterday => (back(1), back(1)),
            Preset::Days(days) => (back(days - 1), today),
        }
    }
}

/// The period the page shows.
struct Selection {
    first: Date,
    last: Date,
    /// The link it came from, if any.
    preset: Option<Preset>,
    /// Why the days the admin typed were not shown as typed.
    note: Option<&'static str>,
}

impl Selection {
    /// The period a query asks for: a [`Preset`] by its code, or the days `from` and `to`,
    /// as `2026-11-01`. Days of the admin's own are kept between the oldest visit kept,
    /// `earliest`, and `today`; without them, it is the preset asked for, or the default
    /// one.
    fn from_query(
        period: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
        today: Date,
        earliest: Date,
    ) -> Selection {
        fn typed(value: Option<&str>) -> Option<&str> {
            value.filter(|value| !value.is_empty())
        }
        let (period, from, to) = (typed(period), typed(from), typed(to));
        let preset = |preset: Preset, note: Option<&'static str>| {
            let (first, last) = preset.range(today);
            Selection { first, last, preset: Some(preset), note }
        };
        if from.is_none() && to.is_none() {
            let asked = period.and_then(|code| PRESETS.into_iter().find(|preset| preset.code() == code));
            return preset(asked.unwrap_or(DEFAULT_PRESET), None);
        }
        let parse = |value: Option<&str>| value.map(str::parse::<Date>);
        let (from, to) = match (parse(from), parse(to)) {
            (Some(Ok(from)), Some(Ok(to))) => (from, to),
            (Some(Ok(day)), None) | (None, Some(Ok(day))) => (day, day),
            _ => return preset(DEFAULT_PRESET, Some("Those dates could not be read.")),
        };
        if from > to {
            return preset(DEFAULT_PRESET, Some("The first day comes after the last one."));
        }
        let first = from.max(earliest);
        let last = to.min(today);
        if first > last {
            return preset(DEFAULT_PRESET, Some("No visits are kept on those days."));
        }
        let note = (first != from || last != to)
            .then_some("Visits are kept for 400 days and go up to today, so the period was adjusted.");
        let preset = PRESETS.into_iter().find(|preset| preset.range(today) == (first, last));
        Selection { first, last, preset, note }
    }
}

/// Names the admin's page in `<head>`. The root layout also keeps it out of search
/// engines.
#[layer]
async fn admin_meta(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let cx = cx.with(PageMeta {
        title: "Admin".to_owned(),
        ..PageMeta::default()
    });
    next.run(&cx, body).await
}

/// What `/admin` was asked for: the period, and whether the last sign-in was wrong.
#[query_params(error = bad_request)]
struct Asked {
    period: Option<String>,
    from: Option<String>,
    to: Option<String>,
    wrong: Option<String>,
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let asked = query_params::<Asked>(cx)?;
    let today = analytics::today();
    let earliest = analytics::earliest_day();
    let selection = Selection::from_query(
        asked.period.as_deref(),
        asked.from.as_deref(),
        asked.to.as_deref(),
        today,
        earliest,
    );
    let report = if admin::signed_in(cx).await? {
        Some(Report::between(&db(cx), selection.first, selection.last).await?)
    } else {
        None
    };

    Ok(view! {
        <section class="mx-auto w-full max-w-6xl px-4 py-10 sm:px-6">
            if let Some(report) = &report {
                dashboard(report: report, selection: &selection, today: today, earliest: earliest)
            } else {
                sign_in_form(set_up: admin::is_set_up(), wrong: asked.wrong.is_some())
            }
        </section>
    })
}

/// The form that signs the admin in, or what to set up first.
#[component]
async fn sign_in_form(set_up: bool, wrong: bool) -> Result<impl View> {
    Ok(view! {
        <div class="mx-auto flex max-w-sm flex-col gap-6 py-10">
            <div class="flex flex-col gap-2">
                <h1 class="text-2xl font-semibold tracking-tight">"Admin"</h1>
                <p class="text-sm text-muted-foreground">"The site's visits, for its admin."</p>
            </div>
            if set_up {
                <form method="post" action="/admin/sign-in" class="flex flex-col gap-4">
                    <label class="flex flex-col gap-2 text-sm font-medium">
                        "Name"
                        input(attrs: attributes! { name="name" autocomplete="username" required="" })
                    </label>
                    <label class="flex flex-col gap-2 text-sm font-medium">
                        "Password"
                        input(attrs: attributes! { name="password" type="password" autocomplete="current-password" required="" })
                    </label>
                    if wrong {
                        <p class="flex items-center gap-2 text-sm text-destructive">
                            icon(data: iconify_icon!("lucide:circle-alert"), size: 16, attrs: attributes! { class="shrink-0" })
                            "Wrong name or password."
                        </p>
                    }
                    button(attrs: attributes! { type="submit" }, "Sign in")
                </form>
            } else {
                <p class="rounded-lg border border-border bg-card p-4 text-sm text-muted-foreground">
                    "Set " <code class="font-mono text-foreground">"ADMIN_USERNAME"</code> " and "
                    <code class="font-mono text-foreground">"ADMIN_PASSWORD"</code>
                    " in the site's environment, such as its service's Environment tab in oxploy, to sign in here."
                </p>
            }
        </div>
    })
}

/// The visits of the period the admin picked.
#[component]
async fn dashboard(report: &Report, selection: &Selection, today: Date, earliest: Date) -> Result<impl View> {
    let single_day = report.first == report.last;
    let per_visitor = if report.visitors == 0 {
        "0".to_owned()
    } else {
        format!("{:.1}", report.views as f64 / report.visitors as f64)
    };
    let daily = daily_bars(report);
    let hourly = hourly_bars(report);

    Ok(view! {
        <div class="flex flex-col gap-6">
            <div class="flex flex-wrap items-end justify-between gap-4">
                <div class="flex flex-col gap-1">
                    <h1 class="text-2xl font-semibold tracking-tight">"Visits"</h1>
                    <p class="text-sm font-medium">(period_label(report.first, report.last, today))</p>
                    <p class="max-w-2xl text-sm text-muted-foreground">
                        "The people who opened the site's pages, each counted once a day, without keeping their address, whether they accepted the cookie or not. Programs, such as search engines, are left out. Days and times are in "
                        (&report.time_zone) "."
                    </p>
                </div>
                <div class="flex flex-col items-start gap-3 lg:items-end">
                    <div class="flex flex-wrap items-center gap-2">
                        <nav class="flex flex-wrap rounded-lg border border-border p-0.5 text-sm" aria-label="Period">
                            #[key(preset.code())]
                            for preset in PRESETS {
                                <a
                                    href=(format!("/admin?period={}", preset.code()))
                                    class=(if selection.preset == Some(preset) { "rounded-md bg-foreground/10 px-3 py-1.5 font-medium" } else { "rounded-md px-3 py-1.5 text-muted-foreground hover:text-foreground" })
                                    aria-current=((selection.preset == Some(preset)).then_some("page"))
                                >
                                    (preset.label())
                                </a>
                            }
                        </nav>
                        <form method="post" action="/admin/sign-out">
                            button(variant: ButtonVariant::Ghost, size: ButtonSize::Sm, attrs: attributes! { type="submit" }, "Sign out")
                        </form>
                    </div>
                    <form method="get" action="/admin" class="flex flex-wrap items-end gap-2 text-sm">
                        <label class="flex flex-col gap-1">
                            <span class="text-xs text-muted-foreground">"From"</span>
                            <input
                                type="date"
                                name="from"
                                value=(report.first.to_string())
                                min=(earliest.to_string())
                                max=(today.to_string())
                                class="h-8 rounded-md border border-border bg-transparent px-2"
                            >
                        </label>
                        <label class="flex flex-col gap-1">
                            <span class="text-xs text-muted-foreground">"To"</span>
                            <input
                                type="date"
                                name="to"
                                value=(report.last.to_string())
                                min=(earliest.to_string())
                                max=(today.to_string())
                                class="h-8 rounded-md border border-border bg-transparent px-2"
                            >
                        </label>
                        button(variant: ButtonVariant::Outline, size: ButtonSize::Sm, attrs: attributes! { type="submit" }, "Show")
                    </form>
                </div>
            </div>
            if let Some(note) = selection.note {
                <p role="status" class="rounded-lg border border-border bg-card px-4 py-3 text-sm">(note)</p>
            }

            <div class="grid gap-4 sm:grid-cols-3">
                stat(label: "Visitors", value: grouped(report.visitors), note: "People, once a day each")
                stat(label: "Page views", value: grouped(report.views), note: "Pages opened")
                stat(label: "Pages per visitor", value: per_visitor.clone(), note: "Page views for each visitor")
            </div>

            if single_day {
                bar_chart(title: "Each hour", note: "", unit: "Hour", bars: &hourly, newest_first: false)
            } else {
                bar_chart(title: "Each day", note: "", unit: "Day", bars: &daily, newest_first: true)
                bar_chart(
                    title: "Hour of the day",
                    note: "Each hour, summed over the period's days.",
                    unit: "Hour",
                    bars: &hourly,
                    newest_first: false
                )
            }

            <div class="grid gap-4 lg:grid-cols-2">
                <section class="flex flex-col gap-3 rounded-xl border border-border bg-card p-5">
                    <h2 class="font-semibold">"Pages"</h2>
                    if report.pages.is_empty() {
                        <p class="text-sm text-muted-foreground">"No page was opened in these days."</p>
                    } else {
                        <div class="flex justify-between px-3 text-xs text-muted-foreground">
                            <span>"Page"</span>
                            <span>"Views · visitors"</span>
                        </div>
                        <ol class="flex flex-col gap-1">
                            #[key(row.path.clone())]
                            for row in &report.pages {
                                share_row(
                                    label: &row.path,
                                    value: format!("{} · {}", grouped(row.views), grouped(row.visitors)),
                                    share: share(row.views, report.pages[0].views),
                                    monospace: true
                                )
                            }
                        </ol>
                    }
                </section>
                sources_card(sources: &report.sources)
            </div>

            campaigns_card(campaigns: &report.campaigns)

            visits_card(visits: &report.visits, visitors: report.visitors, single_day: single_day)

            <div class="grid gap-4 lg:grid-cols-2">
                tally_card(title: "Devices", column: "Device", rows: &report.devices, empty: "No one came in these days.")
                tally_card(title: "Browsers", column: "Browser", rows: &report.browsers, empty: "No one came in these days.")
            </div>

            accepted_card(accepted: &report.accepted, visitors: report.visitors)
        </div>
    })
}

/// Who sent the visitors.
#[component]
async fn sources_card(sources: &[Source]) -> Result<impl View> {
    Ok(view! {
        <section class="flex flex-col gap-3 rounded-xl border border-border bg-card p-5">
            <div class="flex flex-col gap-1">
                <h2 class="font-semibold">"Sources"</h2>
                <p class="text-sm text-muted-foreground">
                    "Where each visitor came from, by the first page they opened. When the browser does not say, the link's utm_source does."
                </p>
            </div>
            if sources.is_empty() {
                <p class="text-sm text-muted-foreground">"No one came in these days."</p>
            } else {
                <div class="flex justify-between px-3 text-xs text-muted-foreground">
                    <span>"Source"</span>
                    <span>"Visitors"</span>
                </div>
                <ol class="flex flex-col gap-1">
                    #[key(index)]
                    for (index, source) in sources.iter().enumerate() {
                        share_row(
                            label: source.name.as_deref().unwrap_or("Direct, a bookmark or an app"),
                            value: grouped(source.visitors),
                            share: share(source.visitors, sources[0].visitors),
                            monospace: false
                        )
                    }
                </ol>
            }
        </section>
    })
}

/// The visitors each campaign brought, by the tags of the first link they followed.
#[component]
async fn campaigns_card(campaigns: &[Campaign]) -> Result<impl View> {
    let tag = |value: &Option<String>| value.clone().unwrap_or_else(|| "–".to_owned());

    Ok(view! {
        <section class="flex flex-col gap-3 rounded-xl border border-border bg-card p-5">
            <div class="flex flex-col gap-1">
                <h2 class="font-semibold">"Campaigns"</h2>
                <p class="text-sm text-muted-foreground">
                    "The visitors who came by a link with a utm_source, by the tags of the first such link they followed. Tag each link you share, such as "
                    <code class="rounded bg-muted px-1 py-0.5 font-mono text-xs break-all">
                        "?utm_source=linkedin&utm_medium=post&utm_campaign=oxploy-launch&utm_content=video-1"
                    </code>
                    "."
                </p>
            </div>
            if campaigns.is_empty() {
                <p class="text-sm text-muted-foreground">"No one followed a link with a utm_source in these days."</p>
            } else {
                <div class="overflow-x-auto">
                    <table class="w-full text-left text-sm tabular-nums">
                        <thead class="text-xs text-muted-foreground">
                            <tr>
                                <th class="py-1.5 pr-4 font-medium">"utm_source"</th>
                                <th class="py-1.5 pr-4 font-medium">"utm_medium"</th>
                                <th class="py-1.5 pr-4 font-medium">"utm_campaign"</th>
                                <th class="py-1.5 pr-4 font-medium">"utm_content"</th>
                                <th class="py-1.5 text-right font-medium">"Visitors"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            #[key(index)]
                            for (index, campaign) in campaigns.iter().enumerate() {
                                <tr>
                                    <td class="py-1.5 pr-4 font-medium">(&campaign.source)</td>
                                    <td class="py-1.5 pr-4">(tag(&campaign.medium))</td>
                                    <td class="py-1.5 pr-4">(tag(&campaign.name))</td>
                                    <td class="py-1.5 pr-4">(tag(&campaign.content))</td>
                                    <td class="py-1.5 text-right">(grouped(campaign.visitors))</td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>
            }
        </section>
    })
}

/// Each visitor's day: when they came, from where, on what, and the pages they opened.
#[component]
async fn visits_card(visits: &[Visit], visitors: usize, single_day: bool) -> Result<impl View> {
    Ok(view! {
        <section class="flex flex-col gap-3 rounded-xl border border-border bg-card p-5">
            <div class="flex flex-col gap-1">
                <h2 class="font-semibold">"Visits"</h2>
                <p class="text-sm text-muted-foreground">
                    "Each visitor on each day: when they came, from where, and the pages they opened, in order."
                    if visitors > visits.len() {
                        " These are the latest " (VISITS_LISTED.to_string()) " of " (grouped(visitors)) "."
                    }
                </p>
            </div>
            if visits.is_empty() {
                <p class="text-sm text-muted-foreground">"No one came in these days."</p>
            } else {
                <div class="overflow-x-auto">
                    <table class="w-full text-left text-sm">
                        <thead class="text-xs text-muted-foreground">
                            <tr>
                                <th class="py-1.5 pr-4 font-medium">"Came"</th>
                                <th class="py-1.5 pr-4 font-medium">"Source"</th>
                                <th class="py-1.5 pr-4 font-medium">"Device"</th>
                                <th class="py-1.5 font-medium">"Pages"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            #[key(index)]
                            for (index, visit) in visits.iter().enumerate() {
                                <tr class="align-top">
                                    <td class="py-2 pr-4 whitespace-nowrap tabular-nums">
                                        (arrival(visit.start, single_day))
                                        if let Some(stay) = stay(visit) {
                                            <div class="text-xs text-muted-foreground">(stay)</div>
                                        }
                                    </td>
                                    <td class="py-2 pr-4">
                                        (visit.source.as_deref().unwrap_or("Direct, a bookmark or an app"))
                                        if let Some(campaign) = &visit.campaign {
                                            <div class="text-xs text-muted-foreground">(campaign)</div>
                                        }
                                    </td>
                                    <td class="py-2 pr-4 whitespace-nowrap">
                                        (visit.device.as_deref().unwrap_or("–"))
                                        <div class="text-xs text-muted-foreground">(visit.browser.as_deref().unwrap_or("–"))</div>
                                    </td>
                                    <td class="py-2 font-mono text-[13px] break-words">(visit.pages.join(" → "))</td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>
            }
        </section>
    })
}

/// A list of how many visitors each value has, such as each browser.
#[component]
async fn tally_card(title: &str, column: &str, rows: &[(String, usize)], empty: &str) -> Result<impl View> {
    Ok(view! {
        <section class="flex flex-col gap-3 rounded-xl border border-border bg-card p-5">
            <h2 class="font-semibold">(title)</h2>
            if rows.is_empty() {
                <p class="text-sm text-muted-foreground">(empty)</p>
            } else {
                <div class="flex justify-between px-3 text-xs text-muted-foreground">
                    <span>(column)</span>
                    <span>"Visitors"</span>
                </div>
                <ol class="flex flex-col gap-1">
                    #[key(value.clone())]
                    for (value, visitors) in rows {
                        share_row(
                            label: value,
                            value: grouped(*visitors),
                            share: share(*visitors, rows[0].1),
                            monospace: false
                        )
                    }
                </ol>
            }
        </section>
    })
}

/// What the cookie tells, among the visitors who accepted it.
#[component]
async fn accepted_card(accepted: &Accepted, visitors: usize) -> Result<impl View> {
    Ok(view! {
        <section class="flex flex-col gap-4 rounded-xl border border-border bg-card p-5">
            <div class="flex flex-col gap-1">
                <h2 class="font-semibold">"Visitors who accepted the cookie"</h2>
                <p class="text-sm text-muted-foreground">
                    "The cookie tells when the same browser comes back on another day. These numbers count only the visitors who accepted it, so they say less the fewer accept."
                </p>
            </div>
            <div class="grid gap-4 sm:grid-cols-3">
                measure(
                    label: "Accepted",
                    value: percent(accepted.visitors, visitors),
                    note: format!("{} of the {} visitors", grouped(accepted.visitors), grouped(visitors))
                )
                measure(
                    label: "Browsers",
                    value: grouped(accepted.browsers),
                    note: "Each counted once in these days".to_owned()
                )
                measure(
                    label: "Came back",
                    value: percent(accepted.returning, accepted.browsers),
                    note: format!("{} came on two days or more", grouped(accepted.returning))
                )
            </div>
        </section>
    })
}

/// A number inside a card, with what it counts.
#[component]
async fn measure(label: &str, value: String, note: String) -> Result<impl View> {
    Ok(view! {
        <div class="flex flex-col gap-1">
            <span class="text-sm text-muted-foreground">(label)</span>
            <span class="text-2xl font-semibold tracking-tight tabular-nums">(value)</span>
            <span class="text-xs text-muted-foreground">(note)</span>
        </div>
    })
}

/// One number of the period, with what it counts.
#[component]
async fn stat(label: &str, value: String, note: &str) -> Result<impl View> {
    Ok(view! {
        <div class="flex flex-col gap-1 rounded-xl border border-border bg-card p-5">
            <span class="text-sm text-muted-foreground">(label)</span>
            <span class="text-3xl font-semibold tracking-tight tabular-nums">(value)</span>
            <span class="text-xs text-muted-foreground">(note)</span>
        </div>
    })
}

/// A line of a list, with a bar behind it as long as its share of the first line's.
#[component]
async fn share_row(label: &str, value: String, share: f64, monospace: bool) -> Result<impl View> {
    Ok(view! {
        <li class="relative flex items-center justify-between gap-3 overflow-hidden rounded-md px-3 py-2 text-sm">
            <span
                class="absolute inset-y-0 left-0 rounded-md bg-chart-1/15"
                style=(format!("width: {:.1}%", share * 100.0))
                aria-hidden="true"
            ></span>
            <span class=(if monospace { "relative truncate font-mono text-[13px]" } else { "relative truncate" })>(label)</span>
            <span class="relative shrink-0 tabular-nums text-muted-foreground">(value)</span>
        </li>
    })
}

/// One bar pair of a chart: the views and the visitors of a day or an hour.
struct Bar {
    /// What the bar stands for, in the tooltip and the table, such as `Wed, Nov 4`.
    name: String,
    /// What the axis says under the bar, for the bars that name it.
    axis: Option<String>,
    views: usize,
    visitors: usize,
}

/// The period's days as bars. The first, middle and last days name the axis.
fn daily_bars(report: &Report) -> Vec<Bar> {
    let last = report.days.len().saturating_sub(1);
    report
        .days
        .iter()
        .enumerate()
        .map(|(index, day)| Bar {
            name: long_date(day.date),
            axis: (index == 0 || index == last || index == last / 2).then(|| short_date(day.date)),
            views: day.views,
            visitors: day.visitors,
        })
        .collect()
}

/// The hours of the day as bars, every third one naming the axis.
fn hourly_bars(report: &Report) -> Vec<Bar> {
    report
        .hours
        .iter()
        .map(|hour| Bar {
            name: format!("{:02}:00 to {:02}:00", hour.hour, hour.hour + 1),
            axis: (hour.hour % 3 == 0).then(|| format!("{:02}h", hour.hour)),
            views: hour.views,
            visitors: hour.visitors,
        })
        .collect()
}

/// Bars of views and visitors side by side, with the numbers in a table under them.
#[component]
async fn bar_chart(
    title: &str,
    /// A line under the title, or empty for none.
    note: &str,
    /// What the table's first column holds, such as `Day`.
    unit: &str,
    bars: &[Bar],
    /// Whether the table lists the last bar first.
    newest_first: bool,
) -> Result<impl View> {
    let peak = bars.iter().map(|bar| bar.views).max().unwrap_or(0);
    let top = nice_ceiling(peak);
    let height = move |count: usize| format!("height: {:.1}%", share(count, top) * 100.0);
    let rows: Vec<&Bar> = if newest_first { bars.iter().rev().collect() } else { bars.iter().collect() };

    Ok(view! {
        <section class="flex flex-col gap-4 rounded-xl border border-border bg-card p-5">
            <div class="flex flex-wrap items-center justify-between gap-3">
                <div class="flex flex-col gap-1">
                    <h2 class="font-semibold">(title)</h2>
                    if !note.is_empty() {
                        <p class="text-sm text-muted-foreground">(note)</p>
                    }
                </div>
                <div class="flex items-center gap-4 text-xs text-muted-foreground">
                    <span class="flex items-center gap-1.5"><span class="size-2.5 rounded-sm bg-chart-1"></span>"Page views"</span>
                    <span class="flex items-center gap-1.5"><span class="size-2.5 rounded-sm bg-chart-2"></span>"Visitors"</span>
                </div>
            </div>
            <div class="flex gap-3">
                <div class="flex h-48 flex-col justify-between text-right text-xs text-muted-foreground tabular-nums">
                    <span class="-translate-y-1/2">(grouped(top))</span>
                    <span class="-translate-y-1/2">(grouped(top / 2))</span>
                    <span class="translate-y-1/2">"0"</span>
                </div>
                <div class="flex min-w-0 flex-1 flex-col gap-2">
                    <div class="relative h-48">
                        <div class="absolute inset-x-0 top-0 border-t border-border/70"></div>
                        <div class="absolute inset-x-0 top-1/2 border-t border-border/70"></div>
                        <div class="absolute inset-0 flex items-end gap-0.5 border-b border-border">
                            #[key(bar.name.clone())]
                            for bar in bars {
                                <div
                                    class="group flex h-full flex-1 items-end justify-center gap-px rounded-t hover:bg-foreground/5"
                                    title=(format!("{}: {} page views, {} visitors", bar.name, grouped(bar.views), grouped(bar.visitors)))
                                >
                                    <span class="w-1/2 max-w-3 rounded-t bg-chart-1" style=(height(bar.views))></span>
                                    <span class="w-1/2 max-w-3 rounded-t bg-chart-2" style=(height(bar.visitors))></span>
                                </div>
                            }
                        </div>
                    </div>
                    <div class="flex text-xs text-muted-foreground">
                        #[key(bar.name.clone())]
                        for bar in bars {
                            <span class="flex-1 text-center whitespace-nowrap">
                                if let Some(axis) = &bar.axis { (axis) }
                            </span>
                        }
                    </div>
                </div>
            </div>
            <details class="text-sm">
                <summary class="cursor-pointer text-muted-foreground hover:text-foreground">"Show the numbers"</summary>
                <table class="mt-3 w-full text-left tabular-nums">
                    <thead class="text-xs text-muted-foreground">
                        <tr><th class="py-1.5 font-medium">(unit)</th><th class="py-1.5 text-right font-medium">"Page views"</th><th class="py-1.5 text-right font-medium">"Visitors"</th></tr>
                    </thead>
                    <tbody class="divide-y divide-border">
                        #[key(bar.name.clone())]
                        for bar in rows {
                            <tr><td class="py-1.5">(&bar.name)</td><td class="py-1.5 text-right">(grouped(bar.views))</td><td class="py-1.5 text-right">(grouped(bar.visitors))</td></tr>
                        }
                    </tbody>
                </table>
            </details>
        </section>
    })
}

/// The admin's name and password, as the form sends them.
#[derive(Deserialize)]
struct SignInForm {
    name: String,
    password: String,
}

/// Signs the admin in, or shows the form again saying it was wrong.
#[route(POST "./sign-in")]
async fn sign_in(cx: &Cx, Form(form): Form<SignInForm>) -> Result<SeeOther> {
    let signed = admin::sign_in(cx, &form.name, &form.password).await?;
    Ok(see_other(if signed { "/admin" } else { "/admin?wrong=1" }))
}

/// Signs the admin out.
#[route(POST "./sign-out")]
async fn sign_out(cx: &Cx) -> Result<SeeOther> {
    admin::sign_out(cx).await?;
    Ok(see_other("/admin"))
}

/// `count` as a share of `of`, from 0 to 1.
fn share(count: usize, of: usize) -> f64 {
    if of == 0 { 0.0 } else { count as f64 / of as f64 }
}

/// `count` as a percentage of `of`, such as `40%`, or a dash when `of` is 0.
fn percent(count: usize, of: usize) -> String {
    if of == 0 {
        "–".to_owned()
    } else {
        format!("{:.0}%", share(count, of) * 100.0)
    }
}

/// The smallest of 1, 2 or 5 times a power of ten that is at least `count`, so the
/// axis reads in round numbers.
fn nice_ceiling(count: usize) -> usize {
    let mut power = 1;
    loop {
        for step in [1, 2, 5] {
            if step * power >= count.max(1) {
                return step * power;
            }
        }
        power *= 10;
    }
}

/// `count` with its thousands apart, such as `12,345`.
fn grouped(count: usize) -> String {
    let digits = count.to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// When a visitor came: `14:32`, or with the day when the period has more than one,
/// `Wed, Nov 4, 14:32`.
fn arrival(at: Timestamp, single_day: bool) -> String {
    let at = at.to_zoned(analytics::time_zone());
    let time = at.strftime("%H:%M").to_string();
    if single_day { time } else { format!("{}, {time}", long_date(at.date())) }
}

/// How long a visitor stayed, from their first page to their last, such as `for 8 min`,
/// or `None` when they opened a single page.
fn stay(visit: &Visit) -> Option<String> {
    if visit.pages.len() < 2 {
        return None;
    }
    let minutes = (visit.end.as_second() - visit.start.as_second()) / 60;
    Some(if minutes < 1 { "under 1 min".to_owned() } else { format!("for {minutes} min") })
}

/// The period as a heading: `Today, Thu, Nov 12, 2026`, `Wed, Nov 4, 2026`, or
/// `Nov 4 – Nov 10, 2026`.
fn period_label(first: Date, last: Date, today: Date) -> String {
    let year = |date: Date| format!("{}, {}", short_date(date), date.year());
    if first == last {
        let day = format!("{}, {}", long_date(first), first.year());
        if first == today {
            format!("Today, {day}")
        } else if Some(first) == today.yesterday().ok() {
            format!("Yesterday, {day}")
        } else {
            day
        }
    } else if first.year() == last.year() {
        format!("{} – {}", short_date(first), year(last))
    } else {
        format!("{} – {}", year(first), year(last))
    }
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// A day as `Nov 4`.
fn short_date(date: Date) -> String {
    format!("{} {}", MONTHS[(date.month() - 1) as usize], date.day())
}

/// A day as `Wed, Nov 4`.
fn long_date(date: Date) -> String {
    let weekday = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][date.weekday().to_monday_zero_offset() as usize];
    format!("{weekday}, {}", short_date(date))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_read_in_round_numbers() {
        assert_eq!(nice_ceiling(0), 1);
        assert_eq!(nice_ceiling(3), 5);
        assert_eq!(nice_ceiling(7), 10);
        assert_eq!(nice_ceiling(130), 200);
        assert_eq!(nice_ceiling(1_000), 1_000);
    }

    #[test]
    fn numbers_group_their_thousands() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(12_345), "12,345");
        assert_eq!(grouped(1_234_567), "1,234,567");
    }

    #[test]
    fn periods_read_as_headings() {
        let today = Date::constant(2026, 11, 12);
        let day = Date::constant(2026, 11, 4);
        assert_eq!(period_label(today, today, today), "Today, Thu, Nov 12, 2026");
        assert_eq!(period_label(Date::constant(2026, 11, 11), Date::constant(2026, 11, 11), today), "Yesterday, Wed, Nov 11, 2026");
        assert_eq!(period_label(day, day, today), "Wed, Nov 4, 2026");
        assert_eq!(period_label(day, today, today), "Nov 4 – Nov 12, 2026");
        assert_eq!(period_label(Date::constant(2025, 12, 28), today, today), "Dec 28, 2025 – Nov 12, 2026");
    }

    #[test]
    fn periods_come_from_the_links_or_the_dates() {
        let today = Date::constant(2026, 11, 10);
        let earliest = Date::constant(2025, 10, 7);
        let select = |period, from, to| Selection::from_query(period, from, to, today, earliest);
        let day = |month, day| Date::constant(2026, month, day);

        let default = select(None, None, None);
        assert_eq!((default.first, default.last), (day(10, 12), today));
        assert_eq!(default.preset, Some(DEFAULT_PRESET));

        let yesterday = select(Some("yesterday"), None, None);
        assert_eq!((yesterday.first, yesterday.last), (day(11, 9), day(11, 9)));

        let own = select(None, Some("2026-11-01"), Some("2026-11-05"));
        assert_eq!((own.first, own.last), (day(11, 1), day(11, 5)));
        assert_eq!((own.preset, own.note), (None, None));

        let one_day = select(None, Some("2026-11-03"), Some(""));
        assert_eq!((one_day.first, one_day.last), (day(11, 3), day(11, 3)));

        let today_typed = select(None, Some("2026-11-10"), Some("2026-11-10"));
        assert_eq!(today_typed.preset, Some(Preset::Today));

        let clamped = select(None, Some("2020-01-01"), Some("2030-01-01"));
        assert_eq!((clamped.first, clamped.last), (earliest, today));
        assert!(clamped.note.is_some());

        let backwards = select(None, Some("2026-11-05"), Some("2026-11-01"));
        assert_eq!(backwards.preset, Some(DEFAULT_PRESET));
        assert!(backwards.note.is_some());

        let unreadable = select(None, Some("yesterday"), None);
        assert!(unreadable.note.is_some());
    }
}
