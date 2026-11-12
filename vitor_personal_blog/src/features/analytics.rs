//! Visits to the site: each page someone opens, counted without cookies and without
//! keeping anyone's address. A visitor is a hash of their address and browser, made with
//! a secret key and the day, so the same person counts once a day and cannot be
//! followed from one day to the next.
//!
//! Someone who accepts the site's cookie also gets a random ID in it, which tells when
//! they come back on another day. Without their yes, nothing is stored in their
//! browser for this.

use std::collections::{BTreeMap, HashMap, HashSet};

use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan, Zoned};
use sha2::{Digest, Sha256};
use toasty::Db;
use topcoat::context::{Cx, app_context};
use topcoat::cookie::{Cookie, Cookies, SameSite, cookies, time::Duration};
use topcoat::router::Method;
use topcoat::router::request::{headers, method, original_headers, remote_addr, uri};

/// One page someone opened.
#[derive(Debug, toasty::Model)]
pub struct PageView {
    #[key]
    #[auto]
    pub id: u64,

    /// When, in seconds since the Unix epoch.
    #[index]
    pub at: i64,

    /// The page's path, such as `/blog/how_use_axum_askama_htmx`.
    pub path: String,

    /// Who opened it, as a hash that changes every day.
    pub visitor: String,

    /// The site the visitor came from, such as `github.com`, when it was another one, or
    /// the Android app that opened the link, as `app:` and its package, such as
    /// `app:com.slack`.
    pub referrer: Option<String>,

    /// The kind of device: `Mobile`, `Tablet` or `Desktop`.
    pub device: Option<String>,

    /// The browser's family, such as `Firefox`.
    pub browser: Option<String>,

    /// The campaign in the link the visitor followed, its `utm_source`, such as
    /// `newsletter`.
    pub campaign: Option<String>,

    /// The link's `utm_medium`, such as `bio`, kept like [`campaign`](Self::campaign).
    pub utm_medium: Option<String>,

    /// The link's `utm_campaign`, such as `oxploy-launch`.
    pub utm_campaign: Option<String>,

    /// The link's `utm_content`, such as `thread-1`, which tells posts apart.
    pub utm_content: Option<String>,

    /// A hash of the ID in the visitor's cookie, when they accepted it, which stays
    /// the same from one day to the next.
    pub browser_id: Option<String>,
}

/// The secret the visitors' hashes are made with. There is one, made the first time
/// the site runs.
#[derive(Debug, toasty::Model)]
pub struct AnalyticsKey {
    #[key]
    #[auto]
    pub id: u64,

    pub key: Vec<u8>,
}

/// How long visits are kept.
pub const KEPT_DAYS: i64 = 400;

/// The time zone the visits' days and hours are in: the admin's, Brasília's.
const TIME_ZONE: &str = "America/Sao_Paulo";

/// The cookie with the visitor's choice: `yes` or `no`. It only remembers the choice, so
/// it needs no consent of its own.
const CHOICE_COOKIE: &str = "cookies";

/// The cookie with the random ID of a visitor who accepted it.
const ID_COOKIE: &str = "visitor";

/// How long the choice and the ID last, after which the site asks again: 13 months.
const COOKIE_DAYS: i64 = 395;

/// What a browser's user agent says when a program, not a person, opens the page.
const BOTS: [&str; 16] = [
    "bot", "crawl", "spider", "slurp", "curl", "wget", "python", "go-http", "headless",
    "preview", "facebookexternalhit", "monitor", "uptime", "lighthouse", "scan", "fetch",
];

/// The site's visits, shared by every request.
#[derive(Clone)]
pub struct Analytics {
    db: Db,
    key: Vec<u8>,
}

impl Analytics {
    /// Takes the key, making it the first time, and forgets the visits older than
    /// [`KEPT_DAYS`].
    pub async fn start(db: Db) -> toasty::Result<Analytics> {
        let key = match AnalyticsKey::all().first().exec(&mut db.clone()).await? {
            Some(stored) => stored.key,
            None => {
                let mut key = vec![0; 32];
                getrandom::fill(&mut key).expect("the system provides random bytes");
                toasty::create!(AnalyticsKey { key: key.clone() })
                    .exec(&mut db.clone())
                    .await?;
                key
            }
        };
        let cutoff = Timestamp::now().as_second() - KEPT_DAYS * 24 * 60 * 60;
        PageView::filter(PageView::fields().at().lt(cutoff))
            .delete()
            .exec(&mut db.clone())
            .await?;
        Ok(Analytics { db, key })
    }

    /// Returns the analytics registered on the router.
    pub fn get(cx: &Cx) -> &Analytics {
        app_context::<Analytics>(cx)
    }

    /// Counts the page this request opens, unless a program opened it, the browser is
    /// only prefetching it, or the browser runtime asked for it again. It is saved on a
    /// task of its own, so the page does not wait for it.
    pub fn record(&self, cx: &Cx) {
        if method(cx) != Method::GET {
            return;
        }
        let header = |name: &str| {
            headers(cx)
                .get(name)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("")
                .trim()
                .to_owned()
        };
        let prefetch = [header("sec-purpose"), header("purpose")]
            .iter()
            .any(|purpose| purpose.contains("prefetch"));
        if prefetch || original_headers(cx).contains_key("x-topcoat-runtime") {
            return;
        }
        let agent = header("user-agent");
        if is_bot(&agent) {
            return;
        }
        // Behind Cloudflare, it says who the visitor is; Caddy always says who
        // connected to it.
        let address = [header("cf-connecting-ip"), header("x-forwarded-for")]
            .into_iter()
            .map(|value| value.split(',').next().unwrap_or("").trim().to_owned())
            .find(|value| !value.is_empty())
            .or_else(|| remote_addr(cx).map(|address| address.ip().to_string()))
            .unwrap_or_default();
        let referrer = referrer_host(&header("referer"), &header("host"));
        let now = Zoned::now().with_time_zone(time_zone());
        let visitor = visitor_hash(&self.key, now.date(), &address, &agent);
        let browser_id = (choice(cx) == Some(true))
            .then(|| cookies(cx).get(ID_COOKIE))
            .flatten()
            .map(|cookie| cookie.value().to_owned())
            .filter(|id| id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .map(|id| keyed_hash(&self.key, &["cookie", &id]));
        let query = uri(cx).query().unwrap_or("");
        let view = PageView {
            id: 0,
            at: now.timestamp().as_second(),
            path: uri(cx).path().to_owned(),
            visitor,
            referrer,
            device: Some(device(&agent).to_owned()),
            browser: Some(browser(&agent).to_owned()),
            campaign: utm(query, "utm_source"),
            utm_medium: utm(query, "utm_medium"),
            utm_campaign: utm(query, "utm_campaign"),
            utm_content: utm(query, "utm_content"),
            browser_id,
        };
        let mut db = self.db.clone();
        tokio::spawn(async move {
            let saved = toasty::create!(PageView {
                at: view.at,
                path: view.path,
                visitor: view.visitor,
                referrer: view.referrer,
                device: view.device,
                browser: view.browser,
                campaign: view.campaign,
                utm_medium: view.utm_medium,
                utm_campaign: view.utm_campaign,
                utm_content: view.utm_content,
                browser_id: view.browser_id,
            })
            .exec(&mut db)
            .await;
            if let Err(error) = saved {
                eprintln!("vitor_personal_blog: a visit could not be saved: {error}");
            }
        });
    }
}

/// The visitor's choice about the site's cookie: `Some(true)` if they accepted it,
/// `Some(false)` if they did not, `None` if they have not chosen.
pub fn choice(cx: &Cx) -> Option<bool> {
    match cookies(cx).get(CHOICE_COOKIE)?.value() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Records the visitor's choice in their browser, with a new random ID when they
/// accept, or without the ID they had when they do not. Only a route can call it,
/// before its answer starts.
pub fn choose(cx: &Cx, accept: bool) {
    let cookie = |name: &'static str, value: String| {
        Cookie::build((name, value))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::days(COOKIE_DAYS))
            .build()
    };
    let jar = cookies(cx);
    jar.add(cookie(CHOICE_COOKIE, (if accept { "yes" } else { "no" }).to_owned()));
    if accept {
        let mut id = [0; 16];
        getrandom::fill(&mut id).expect("the system provides random bytes");
        jar.add(cookie(ID_COOKIE, id.iter().map(|byte| format!("{byte:02x}")).collect()));
    } else {
        jar.remove(Cookie::build(ID_COOKIE).path("/").build());
    }
}

/// The kind of device a user agent is on.
fn device(agent: &str) -> &'static str {
    let agent = agent.to_ascii_lowercase();
    let android = agent.contains("android");
    if agent.contains("ipad") || agent.contains("tablet") || (android && !agent.contains("mobile")) {
        "Tablet"
    } else if android || agent.contains("mobi") || agent.contains("iphone") {
        "Mobile"
    } else {
        "Desktop"
    }
}

/// The family of the browser a user agent is, such as `Firefox`. The order matters:
/// Edge and Opera also say Chrome, and Chrome also says Safari.
fn browser(agent: &str) -> &'static str {
    let agent = agent.to_ascii_lowercase();
    [
        ("edg/", "Edge"),
        ("opr/", "Opera"),
        ("samsungbrowser", "Samsung Internet"),
        ("firefox/", "Firefox"),
        ("fxios", "Firefox"),
        ("chrome/", "Chrome"),
        ("crios", "Chrome"),
        ("safari/", "Safari"),
    ]
    .into_iter()
    .find(|(mark, _)| agent.contains(mark))
    .map_or("Other", |(_, name)| name)
}

/// The value of `key` in a query, such as `newsletter` for `utm_source`: decoded, in
/// lowercase, with spaces as `-`, and kept to letters, digits, `.`, `-` and `_`, up to 50
/// of them. The rest of the query is not kept, as it may hold anything.
fn utm(query: &str, key: &str) -> Option<String> {
    let value = query
        .split('&')
        .find_map(|pair| pair.strip_prefix(key)?.strip_prefix('='))?;
    let kept: String = percent_decode(value)
        .to_lowercase()
        .chars()
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .take(50)
        .collect();
    let kept = kept.trim_matches('-');
    (!kept.is_empty()).then(|| kept.to_owned())
}

/// A query value with its `%XX` escapes and `+` decoded.
fn percent_decode(value: &str) -> String {
    let digit = |byte: Option<&u8>| char::from(*byte?).to_digit(16);
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match (byte, digit(bytes.get(index + 1)), digit(bytes.get(index + 2))) {
            (b'%', Some(high), Some(low)) => {
                decoded.push((high * 16 + low) as u8);
                index += 3;
            }
            _ => {
                decoded.push(if byte == b'+' { b' ' } else { byte });
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

/// Whether a user agent is a program's, such as a search engine's.
fn is_bot(agent: &str) -> bool {
    let agent = agent.to_ascii_lowercase();
    agent.is_empty() || BOTS.iter().any(|bot| agent.contains(bot))
}

/// The site a visitor came from, from the `Referer` header, when it is not this one:
/// its host without `www.`. An Android app that opened the link says so as
/// `android-app://` and its package, kept as `app:` and the package, such as
/// `app:com.slack`.
fn referrer_host(referer: &str, own_host: &str) -> Option<String> {
    if let Some(app) = referer.strip_prefix("android-app://") {
        let package = app.split(['/', '?', '#']).next()?.to_ascii_lowercase();
        let valid = !package.is_empty()
            && package.len() <= 100
            && package.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_'));
        return valid.then(|| format!("app:{package}"));
    }
    let rest = referer
        .strip_prefix("https://")
        .or_else(|| referer.strip_prefix("http://"))?;
    let host = rest.split(['/', '?', '#']).next()?.split(':').next()?.to_ascii_lowercase();
    let own = own_host.split(':').next().unwrap_or("").to_ascii_lowercase();
    let bare = |host: &str| host.strip_prefix("www.").unwrap_or(host).to_owned();
    (!host.is_empty() && bare(&host) != bare(&own)).then(|| bare(&host))
}

/// Who sent a visitor, as people call it, from the referrer the site keeps: `X (Twitter)`
/// for `t.co`, `Hacker News` for `news.ycombinator.com`, `Google` for `google.com.br`,
/// and so on. Another site keeps its host, and another app its package.
pub fn source_name(referrer: &str) -> String {
    let is = |domain: &str| referrer == domain || referrer.ends_with(&format!(".{domain}"));
    let google = referrer.split('.').next() == Some("google") || is("google.com");
    let name = if is("t.co") || is("x.com") || is("twitter.com") {
        "X (Twitter)"
    } else if is("linkedin.com") || is("lnkd.in") || referrer == "app:com.linkedin.android" {
        "LinkedIn"
    } else if referrer == "news.ycombinator.com" {
        "Hacker News"
    } else if is("reddit.com") || referrer == "app:com.reddit.frontpage" {
        "Reddit"
    } else if is("github.com") {
        "GitHub"
    } else if is("lobste.rs") {
        "Lobsters"
    } else if is("bsky.app") {
        "Bluesky"
    } else if is("slack.com") || referrer == "app:com.slack" {
        "Slack"
    } else if is("discord.com") || referrer == "app:com.discord" {
        "Discord"
    } else if is("instagram.com") || referrer == "app:com.instagram.android" {
        "Instagram"
    } else if is("facebook.com")
        || is("fb.com")
        || is("fb.me")
        || referrer.starts_with("app:com.facebook.")
    {
        "Facebook"
    } else if is("whatsapp.com") || is("wa.me") || referrer.starts_with("app:com.whatsapp") {
        "WhatsApp"
    } else if is("t.me") || is("telegram.org") || referrer == "app:org.telegram.messenger" {
        "Telegram"
    } else if referrer == "mail.google.com" || referrer == "app:com.google.android.gm" {
        "Gmail"
    } else if is("youtube.com") || is("youtu.be") || referrer == "app:com.google.android.youtube" {
        "YouTube"
    } else if google || referrer == "app:com.google.android.googlequicksearchbox" {
        "Google"
    } else if is("bing.com") {
        "Bing"
    } else if is("duckduckgo.com") {
        "DuckDuckGo"
    } else if is("chatgpt.com") || is("chat.openai.com") {
        "ChatGPT"
    } else if is("perplexity.ai") {
        "Perplexity"
    } else {
        return referrer.strip_prefix("app:").unwrap_or(referrer).to_owned();
    };
    name.to_owned()
}

/// Who sent a visitor whose browser did not say, from the `utm_source` of the link they
/// followed: `X (Twitter)` for `x` or `twitter`, and so on, or the tag itself.
pub fn tagged_source_name(source: &str) -> String {
    let name = match source {
        "x" | "twitter" => "X (Twitter)",
        "linkedin" => "LinkedIn",
        "hn" | "hackernews" | "hacker-news" => "Hacker News",
        "reddit" => "Reddit",
        "github" | "gh" => "GitHub",
        "lobsters" => "Lobsters",
        "bluesky" | "bsky" => "Bluesky",
        "slack" => "Slack",
        "discord" => "Discord",
        "instagram" | "ig" => "Instagram",
        "facebook" | "fb" => "Facebook",
        "whatsapp" | "wa" => "WhatsApp",
        "telegram" => "Telegram",
        "youtube" | "yt" => "YouTube",
        "google" => "Google",
        other => return other.to_owned(),
    };
    name.to_owned()
}

/// A visitor's hash for `day`: the same for one address and browser all day, and
/// another the next day.
fn visitor_hash(key: &[u8], day: Date, address: &str, agent: &str) -> String {
    keyed_hash(key, &[&day.to_string(), address, agent])
}

/// A short hash of `parts` with the site's key, which nobody without the key can tie
/// back to them.
fn keyed_hash(key: &[u8], parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key);
    for part in parts {
        hasher.update([0]);
        hasher.update(part);
    }
    hasher.finalize()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The time zone a visit's day and hour are in.
pub fn time_zone() -> TimeZone {
    TimeZone::get(TIME_ZONE).unwrap_or(TimeZone::UTC)
}

/// Today, in [`time_zone`].
pub fn today() -> Date {
    Zoned::now().with_time_zone(time_zone()).date()
}

/// The oldest day whose visits are still kept, [`KEPT_DAYS`] back from today included.
pub fn earliest_day() -> Date {
    let today = today();
    today.checked_sub((KEPT_DAYS - 1).days()).unwrap_or(today)
}

/// The day of the oldest visit kept, or `None` before the first.
pub async fn counting_since(db: &Db) -> toasty::Result<Option<Date>> {
    let oldest = PageView::all()
        .order_by(PageView::fields().at().asc())
        .first()
        .exec(&mut db.clone())
        .await?;
    Ok(oldest
        .and_then(|view| Timestamp::from_second(view.at).ok())
        .map(|at| at.to_zoned(time_zone()).date()))
}

/// The visits from `first` to `last`, both days included, in [`time_zone`], in the order
/// they came.
async fn views_between(db: &Db, first: Date, last: Date) -> toasty::Result<Vec<PageView>> {
    let zone = time_zone();
    let start = |date: Date| {
        date.to_zoned(zone.clone())
            .map_or(0, |start| start.timestamp().as_second())
    };
    let since = start(first);
    let until = last.tomorrow().map_or(i64::MAX, start);
    let mut views = PageView::filter(PageView::fields().at().ge(since))
        .exec(&mut db.clone())
        .await?;
    views.retain(|view| view.at < until);
    views.sort_by_key(|view| (view.at, view.id));
    Ok(views)
}

/// The visitors and the page views from `first` to `last`, both days included.
pub async fn totals_between(db: &Db, first: Date, last: Date) -> toasty::Result<(usize, usize)> {
    let views = views_between(db, first, last).await?;
    let visitors: HashSet<&str> = views.iter().map(|view| view.visitor.as_str()).collect();
    Ok((visitors.len(), views.len()))
}

/// The visits of a period: its days, both ends included, in [`time_zone`].
pub struct Report {
    pub first: Date,
    pub last: Date,
    /// Each day, oldest first, the ones without visits too.
    pub days: Vec<Day>,
    /// Each hour of the day, from 0 to 23, summed over the period's days.
    pub hours: Vec<Hour>,
    /// People, each counted once a day.
    pub visitors: usize,
    pub views: usize,
    /// The pages most opened, most first.
    pub pages: Vec<Row>,
    /// Who sent the visitors, by the first page each opened, most first.
    pub sources: Vec<Source>,
    /// The visitors on each kind of device, most first.
    pub devices: Vec<(String, usize)>,
    /// The visitors on each browser, most first.
    pub browsers: Vec<(String, usize)>,
    /// The visitors who followed a link with a `utm_source`, by its tags, most first.
    pub campaigns: Vec<Campaign>,
    /// Each visitor's pages on each day, the latest first, up to [`VISITS_LISTED`].
    pub visits: Vec<Visit>,
    /// What the cookie tells, among the visitors who accepted it.
    pub accepted: Accepted,
    /// The time zone the days are in, such as `America/Sao_Paulo`.
    pub time_zone: String,
}

/// The visits of the browsers that accepted the cookie.
pub struct Accepted {
    /// Browsers, each counted once in the whole period.
    pub browsers: usize,
    /// Of those, the ones that came on two days or more.
    pub returning: usize,
    /// Visitors, each counted once a day as in [`Report::visitors`], who had accepted.
    pub visitors: usize,
}

/// One day's visits.
pub struct Day {
    pub date: Date,
    pub views: usize,
    pub visitors: usize,
}

/// One hour's visits, such as from 14:00 to 15:00, over the period's days.
pub struct Hour {
    /// The hour it starts at, from 0 to 23.
    pub hour: i8,
    pub views: usize,
    /// The visitors who opened a page in that hour, each counted once a day.
    pub visitors: usize,
}

/// One page's visits.
pub struct Row {
    pub path: String,
    pub views: usize,
    pub visitors: usize,
}

/// The visitors one source sent.
#[derive(Debug, PartialEq)]
pub struct Source {
    /// Who sent them, as [`source_name`] calls it. When their browser did not say, it is
    /// the `utm_source` of their first page, as [`tagged_source_name`] calls it, and
    /// without one, `None`: they typed the address, came from a bookmark, or from an app
    /// that does not say.
    pub name: Option<String>,
    pub visitors: usize,
}

/// The visitors one campaign brought: those whose first link had these tags.
#[derive(Debug, PartialEq)]
pub struct Campaign {
    /// The `utm_source`.
    pub source: String,
    pub medium: Option<String>,
    /// The `utm_campaign`.
    pub name: Option<String>,
    pub content: Option<String>,
    pub visitors: usize,
}

/// One visitor on one day: when they came, from where, and the pages they opened.
pub struct Visit {
    /// When they opened their first page.
    pub start: Timestamp,
    /// When they opened their last page.
    pub end: Timestamp,
    /// The pages, in the order they opened them.
    pub pages: Vec<String>,
    /// Who sent them, as in [`Source::name`].
    pub source: Option<String>,
    /// The tags of the first link with a `utm_source` they followed, such as
    /// `x · bio · profile`.
    pub campaign: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
}

/// How many pages, sources, devices and browsers a report lists.
const LISTED: usize = 10;

/// How many campaigns a report lists.
const CAMPAIGNS_LISTED: usize = 20;

/// How many visits a report lists, the latest ones.
pub const VISITS_LISTED: usize = 100;

/// One visitor's day as it is gathered, before it becomes a [`Visit`].
struct Draft<'a> {
    first: &'a PageView,
    last: &'a PageView,
    pages: Vec<&'a str>,
    /// Their first view whose link had a `utm_source`.
    tagged: Option<&'a PageView>,
}

impl Report {
    /// The visits from `first` to `last`, both days included, in [`time_zone`].
    pub async fn between(db: &Db, first: Date, last: Date) -> toasty::Result<Report> {
        Ok(Report::of(first, last, &views_between(db, first, last).await?))
    }

    /// The report of `views`, the visits from `first` to `last` in the order they came.
    fn of(first: Date, last: Date, views: &[PageView]) -> Report {
        let zone = time_zone();
        let mut by_day: BTreeMap<Date, (usize, HashSet<&str>)> = BTreeMap::new();
        let mut by_hour: [(usize, HashSet<&str>); 24] = Default::default();
        let mut by_page: HashMap<&str, (usize, HashSet<&str>)> = HashMap::new();
        // Each visitor's day, in the order they first came.
        let mut drafts: Vec<Draft> = Vec::new();
        let mut draft_of: HashMap<&str, usize> = HashMap::new();
        // The days each browser that accepted the cookie came on.
        let mut by_browser: HashMap<&str, HashSet<Date>> = HashMap::new();
        let mut accepting = HashSet::new();
        for view in views {
            let Ok(at) = Timestamp::from_second(view.at) else {
                continue;
            };
            let at = at.to_zoned(zone.clone());
            if let Some(browser) = &view.browser_id {
                by_browser.entry(browser).or_default().insert(at.date());
                accepting.insert(view.visitor.as_str());
            }
            let day = by_day.entry(at.date()).or_default();
            day.0 += 1;
            day.1.insert(&view.visitor);
            let hour = &mut by_hour[at.hour().clamp(0, 23) as usize];
            hour.0 += 1;
            hour.1.insert(&view.visitor);
            let page = by_page.entry(&view.path).or_default();
            page.0 += 1;
            page.1.insert(&view.visitor);
            let index = *draft_of.entry(&view.visitor).or_insert_with(|| {
                drafts.push(Draft { first: view, last: view, pages: Vec::new(), tagged: None });
                drafts.len() - 1
            });
            let draft = &mut drafts[index];
            draft.last = view;
            draft.pages.push(&view.path);
            if draft.tagged.is_none() && view.campaign.is_some() {
                draft.tagged = Some(view);
            }
        }

        let mut days = Vec::new();
        let mut date = first;
        while date <= last {
            let (views, visitors) = by_day
                .get(&date)
                .map_or((0, 0), |(views, visitors)| (*views, visitors.len()));
            days.push(Day { date, views, visitors });
            match date.tomorrow() {
                Ok(next) => date = next,
                Err(_) => break,
            }
        }
        let hours = (0..24)
            .zip(&by_hour)
            .map(|(hour, (views, visitors))| Hour { hour, views: *views, visitors: visitors.len() })
            .collect();
        let mut pages: Vec<Row> = by_page
            .into_iter()
            .map(|(path, (views, visitors))| Row {
                path: path.to_owned(),
                views,
                visitors: visitors.len(),
            })
            .collect();
        pages.sort_by(|a, b| b.views.cmp(&a.views).then(a.path.cmp(&b.path)));
        pages.truncate(LISTED);

        let name = |draft: &Draft| {
            draft.first.referrer.as_deref().map(source_name).or_else(|| {
                draft.first.campaign.as_deref().map(tagged_source_name)
            })
        };
        let mut by_source: HashMap<Option<String>, usize> = HashMap::new();
        let mut by_campaign: HashMap<[Option<&str>; 4], usize> = HashMap::new();
        for draft in &drafts {
            *by_source.entry(name(draft)).or_default() += 1;
            if let Some(tagged) = draft.tagged {
                *by_campaign.entry(tags(tagged)).or_default() += 1;
            }
        }
        let sources = most_first(by_source.into_iter())
            .into_iter()
            .map(|(name, visitors)| Source { name, visitors })
            .collect();
        let owned = |tag: Option<&str>| tag.map(str::to_owned);
        let mut campaigns: Vec<Campaign> = by_campaign
            .into_iter()
            .map(|([source, medium, name, content], visitors)| Campaign {
                source: source.unwrap_or_default().to_owned(),
                medium: owned(medium),
                name: owned(name),
                content: owned(content),
                visitors,
            })
            .collect();
        campaigns.sort_by(|a, b| {
            b.visitors.cmp(&a.visitors).then_with(|| {
                (&a.source, &a.medium, &a.name, &a.content)
                    .cmp(&(&b.source, &b.medium, &b.name, &b.content))
            })
        });
        campaigns.truncate(CAMPAIGNS_LISTED);

        // The drafts are in the order the visitors first came, so the latest last.
        let visits: Vec<Visit> = drafts
            .iter()
            .rev()
            .take(VISITS_LISTED)
            .map(|draft| Visit {
                start: Timestamp::from_second(draft.first.at).unwrap_or_default(),
                end: Timestamp::from_second(draft.last.at).unwrap_or_default(),
                pages: draft.pages.iter().map(|page| (*page).to_owned()).collect(),
                source: name(draft),
                campaign: draft.tagged.map(|tagged| {
                    tags(tagged).into_iter().flatten().collect::<Vec<_>>().join(" · ")
                }),
                device: draft.first.device.clone(),
                browser: draft.first.browser.clone(),
            })
            .collect();

        Report {
            first,
            last,
            days,
            hours,
            visitors: drafts.len(),
            views: views.len(),
            pages,
            sources,
            devices: tally(views, |view| view.device.as_deref()),
            browsers: tally(views, |view| view.browser.as_deref()),
            campaigns,
            visits,
            accepted: Accepted {
                browsers: by_browser.len(),
                returning: by_browser.values().filter(|days| days.len() > 1).count(),
                visitors: accepting.len(),
            },
            time_zone: zone.iana_name().unwrap_or("UTC").to_owned(),
        }
    }
}

/// A view's link tags: its `utm_source`, `utm_medium`, `utm_campaign` and `utm_content`.
fn tags(view: &PageView) -> [Option<&str>; 4] {
    [
        view.campaign.as_deref(),
        view.utm_medium.as_deref(),
        view.utm_campaign.as_deref(),
        view.utm_content.as_deref(),
    ]
}

/// How many visitors have each value of `field`, from the first of their views that
/// has one, most first.
fn tally<'a>(views: &'a [PageView], field: impl Fn(&'a PageView) -> Option<&'a str>) -> Vec<(String, usize)> {
    let mut first: HashMap<&str, &str> = HashMap::new();
    for view in views {
        if let Some(value) = field(view) {
            first.entry(&view.visitor).or_insert(value);
        }
    }
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for value in first.into_values() {
        *counts.entry(value).or_default() += 1;
    }
    most_first(counts.into_iter().map(|(value, visitors)| (value.to_owned(), visitors)))
}

/// One page's visits in a period, and how its visitors found it.
#[derive(Debug, Default, PartialEq)]
pub struct PageStats {
    pub views: usize,
    /// People who opened it, each counted once a day.
    pub visitors: usize,
    /// The visitors whose first page was this one, whom it brought to the site.
    pub entries: usize,
    /// Where those visitors came from, most first, with `None` for those who typed the
    /// address or came from a bookmark.
    pub sources: Vec<(Option<String>, usize)>,
    /// The visitors who opened it from a link with each `utm_source`, most first.
    pub campaigns: Vec<(String, usize)>,
}

impl PageStats {
    /// Each page's visits from `first` to `last`, both days included, by path.
    pub async fn between(
        db: &Db,
        first: Date,
        last: Date,
    ) -> toasty::Result<HashMap<String, PageStats>> {
        Ok(page_stats(&views_between(db, first, last).await?))
    }
}

/// Each page's visits among `views`, which are in the order they came, by path.
fn page_stats(views: &[PageView]) -> HashMap<String, PageStats> {
    #[derive(Default)]
    struct Draft<'a> {
        views: usize,
        visitors: HashSet<&'a str>,
        sources: HashMap<Option<&'a str>, usize>,
        campaigns: HashMap<&'a str, HashSet<&'a str>>,
    }
    let mut drafts: HashMap<&str, Draft> = HashMap::new();
    let mut entered = HashSet::new();
    for view in views {
        let draft = drafts.entry(&view.path).or_default();
        draft.views += 1;
        draft.visitors.insert(&view.visitor);
        if entered.insert(view.visitor.as_str()) {
            *draft.sources.entry(view.referrer.as_deref()).or_default() += 1;
        }
        if let Some(campaign) = &view.campaign {
            draft.campaigns.entry(campaign).or_default().insert(&view.visitor);
        }
    }
    drafts
        .into_iter()
        .map(|(path, draft)| {
            let stats = PageStats {
                views: draft.views,
                visitors: draft.visitors.len(),
                entries: draft.sources.values().sum(),
                sources: most_first(
                    draft.sources.into_iter().map(|(source, visitors)| (source.map(str::to_owned), visitors)),
                ),
                campaigns: most_first(
                    draft.campaigns.into_iter().map(|(campaign, visitors)| (campaign.to_owned(), visitors.len())),
                ),
            };
            (path.to_owned(), stats)
        })
        .collect()
}

/// The [`LISTED`] rows with the most visitors, most first, and alike ones by name.
fn most_first<T: Ord>(rows: impl Iterator<Item = (T, usize)>) -> Vec<(T, usize)> {
    let mut rows: Vec<_> = rows.collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    rows.truncate(LISTED);
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(visitor: &str, path: &str, referrer: Option<&str>, campaign: Option<&str>) -> PageView {
        PageView {
            id: 0,
            at: 0,
            path: path.to_owned(),
            visitor: visitor.to_owned(),
            referrer: referrer.map(str::to_owned),
            device: None,
            browser: None,
            campaign: campaign.map(str::to_owned),
            utm_medium: None,
            utm_campaign: None,
            utm_content: None,
            browser_id: None,
        }
    }

    #[test]
    fn pages_tell_whom_they_brought_and_from_where() {
        let views = [
            view("ana", "/blog/rust", Some("news.ycombinator.com"), None),
            view("ana", "/blog", None, None),
            view("bia", "/blog/rust", Some("linkedin.com"), Some("linkedin")),
            view("bia", "/blog/rust", None, None),
            view("caio", "/", None, None),
            view("caio", "/blog/rust", None, Some("newsletter")),
            view("davi", "/blog/rust", Some("news.ycombinator.com"), None),
        ];
        let pages = page_stats(&views);
        let rust = &pages["/blog/rust"];
        assert_eq!((rust.views, rust.visitors, rust.entries), (5, 4, 3));
        assert_eq!(
            rust.sources,
            [(Some("news.ycombinator.com".to_owned()), 2), (Some("linkedin.com".to_owned()), 1)]
        );
        assert_eq!(rust.campaigns, [("linkedin".to_owned(), 1), ("newsletter".to_owned(), 1)]);
        assert_eq!(pages["/blog"].entries, 0);
        assert_eq!(pages["/"].sources, [(None, 1)]);
    }

    #[test]
    fn reports_tell_who_sent_each_visitor_by_hour_and_campaign() {
        let at = |time: &str| {
            format!("2026-11-12T{time}:00-03:00")
                .parse::<Timestamp>()
                .unwrap()
                .as_second()
        };
        let tagged = |mut view: PageView, medium: &str, name: Option<&str>, content: Option<&str>| {
            view.utm_medium = Some(medium.to_owned());
            view.utm_campaign = name.map(str::to_owned);
            view.utm_content = content.map(str::to_owned);
            view
        };
        let at_time = |mut view: PageView, time: &str| {
            view.at = at(time);
            view
        };
        let views = [
            at_time(tagged(view("ana", "/", Some("t.co"), Some("x")), "bio", Some("profile"), Some("link")), "14:00"),
            at_time(view("ana", "/blog/oxploy", None, None), "14:05"),
            at_time(tagged(view("bia", "/blog/rust", None, Some("linkedin")), "post", None, None), "15:10"),
            at_time(view("caio", "/", Some("news.ycombinator.com"), None), "15:20"),
            at_time(view("davi", "/", None, None), "16:00"),
        ];
        let day = Date::constant(2026, 11, 12);
        let report = Report::of(day, day, &views);

        assert_eq!((report.visitors, report.views), (4, 5));
        assert_eq!((report.hours[14].views, report.hours[14].visitors), (2, 1));
        assert_eq!((report.hours[15].views, report.hours[15].visitors), (2, 2));
        let names: Vec<_> = report.sources.iter().map(|source| source.name.as_deref()).collect();
        assert_eq!(names, [None, Some("Hacker News"), Some("LinkedIn"), Some("X (Twitter)")]);
        assert_eq!(
            report.campaigns,
            [
                Campaign {
                    source: "linkedin".to_owned(),
                    medium: Some("post".to_owned()),
                    name: None,
                    content: None,
                    visitors: 1,
                },
                Campaign {
                    source: "x".to_owned(),
                    medium: Some("bio".to_owned()),
                    name: Some("profile".to_owned()),
                    content: Some("link".to_owned()),
                    visitors: 1,
                },
            ]
        );
        let ana = report.visits.last().unwrap();
        assert_eq!(ana.pages, ["/", "/blog/oxploy"]);
        assert_eq!(ana.source.as_deref(), Some("X (Twitter)"));
        assert_eq!(ana.campaign.as_deref(), Some("x · bio · profile · link"));
        assert_eq!(ana.end.as_second() - ana.start.as_second(), 300);
        assert_eq!(report.visits[0].pages, ["/"]);
        assert_eq!(report.visits[0].source, None);
    }

    #[test]
    fn programs_are_not_visitors() {
        assert!(is_bot(""));
        assert!(is_bot("Mozilla/5.0 (compatible; Googlebot/2.1)"));
        assert!(is_bot("curl/8.7.1"));
        assert!(!is_bot("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/131.0 Safari/537.36"));
    }

    #[test]
    fn referrers_are_other_sites_or_apps() {
        assert_eq!(referrer_host("https://github.com/vitordeoliveira/oxploy", "vitor.ws"), Some("github.com".to_owned()));
        assert_eq!(referrer_host("https://www.google.com/", "vitor.ws"), Some("google.com".to_owned()));
        assert_eq!(referrer_host("https://news.ycombinator.com/item?id=1", "vitor.ws"), Some("news.ycombinator.com".to_owned()));
        assert_eq!(referrer_host("https://vitor.ws/blog", "vitor.ws"), None);
        assert_eq!(referrer_host("https://www.vitor.ws/", "vitor.ws"), None);
        assert_eq!(referrer_host("http://127.0.0.1:3000/", "127.0.0.1:3000"), None);
        assert_eq!(referrer_host("android-app://com.slack/", "vitor.ws"), Some("app:com.slack".to_owned()));
        assert_eq!(referrer_host("android-app://com sla<ck>/", "vitor.ws"), None);
        assert_eq!(referrer_host("", "vitor.ws"), None);
    }

    #[test]
    fn sources_read_as_people_call_them() {
        assert_eq!(source_name("t.co"), "X (Twitter)");
        assert_eq!(source_name("news.ycombinator.com"), "Hacker News");
        assert_eq!(source_name("old.reddit.com"), "Reddit");
        assert_eq!(source_name("lnkd.in"), "LinkedIn");
        assert_eq!(source_name("app:com.slack"), "Slack");
        assert_eq!(source_name("google.com.br"), "Google");
        assert_eq!(source_name("mail.google.com"), "Gmail");
        assert_eq!(source_name("blog.example.com"), "blog.example.com");
        assert_eq!(source_name("app:com.example.reader"), "com.example.reader");
        assert_eq!(tagged_source_name("x"), "X (Twitter)");
        assert_eq!(tagged_source_name("hn"), "Hacker News");
        assert_eq!(tagged_source_name("newsletter"), "newsletter");
    }

    #[test]
    fn devices_and_browsers_are_told_apart() {
        let iphone = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1";
        let android = "Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36";
        let tablet = "Mozilla/5.0 (Linux; Android 14; SM-X710) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
        let edge = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0";
        let firefox = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.7; rv:132.0) Gecko/20100101 Firefox/132.0";
        assert_eq!((device(iphone), browser(iphone)), ("Mobile", "Safari"));
        assert_eq!((device(android), browser(android)), ("Mobile", "Chrome"));
        assert_eq!((device(tablet), browser(tablet)), ("Tablet", "Chrome"));
        assert_eq!((device(edge), browser(edge)), ("Desktop", "Edge"));
        assert_eq!((device(firefox), browser(firefox)), ("Desktop", "Firefox"));
    }

    #[test]
    fn campaign_tags_are_kept_clean() {
        let source = |query: &str| utm(query, "utm_source");
        assert_eq!(source("utm_source=Newsletter&utm_medium=email"), Some("newsletter".to_owned()));
        assert_eq!(source("ref=x&utm_source=reddit.com"), Some("reddit.com".to_owned()));
        assert_eq!(source("utm_source=%3Cscript%3E"), Some("script".to_owned()));
        assert_eq!(source(&format!("utm_source={}", "a".repeat(80))).map(|s| s.len()), Some(50));
        assert_eq!(source("utm_source="), None);
        assert_eq!(source("utm_sourcex=x"), None);
        assert_eq!(source("page=2"), None);
        let query = "utm_source=x&utm_medium=bio&utm_campaign=Oxploy+Launch&utm_content=thread%201";
        assert_eq!(utm(query, "utm_medium"), Some("bio".to_owned()));
        assert_eq!(utm(query, "utm_campaign"), Some("oxploy-launch".to_owned()));
        assert_eq!(utm(query, "utm_content"), Some("thread-1".to_owned()));
    }

    #[test]
    fn visitors_change_every_day() {
        let day = Date::constant(2026, 11, 4);
        let one = visitor_hash(b"key", day, "203.0.113.9", "Firefox");
        assert_eq!(one, visitor_hash(b"key", day, "203.0.113.9", "Firefox"));
        assert_ne!(one, visitor_hash(b"key", day.tomorrow().unwrap(), "203.0.113.9", "Firefox"));
        assert_ne!(one, visitor_hash(b"key", day, "203.0.113.10", "Firefox"));
        assert_eq!(one.len(), 16);
    }
}
