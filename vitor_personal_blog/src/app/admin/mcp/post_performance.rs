//! The `post_performance` tool: how each published post did in a period, which tells
//! the topics and the shares that bring readers.

use std::collections::HashMap;

use serde_json::{Value, json};
use topcoat::{context::Cx, router::href};

use super::common::{PeriodArguments, ToolError, parse, period, period_schema};
use crate::{
    app::{blog, seo},
    db::db,
    features::{
        analytics::{self, PageStats},
        post::Post,
        post_view::PostView,
    },
};

/// The name agents call the tool by.
pub const NAME: &str = "post_performance";

/// What the tool does, for the agent deciding whether to call it.
const DESCRIPTION: &str = "Reports how each published post of the blog, \
    https://vitor.ws, did in a period: the last days, today included, or the days from \
    one date to another. Every post is listed, those nobody read too, the most read \
    first, with its title, address, publication date, tags, and in the period: views; \
    visitors, people who opened it, each counted once a day; entries, the visitors whose \
    first page it was, whom it brought to the site; the sources of those entries, the \
    other sites they came from, or direct; and the campaigns of the links that led \
    readers to it, by their utm_source, such as linkedin. all_time_views is the count \
    the post shows its readers, kept since before the visits were counted and not tied \
    to the period. site_visits gives the site's totals; read_post a post's text.";

/// The tool's name, description and arguments, for `tools/list`.
pub fn definition() -> Value {
    json!({
        "name": NAME,
        "title": "Report each post's readers",
        "description": DESCRIPTION,
        "inputSchema": period_schema(),
        "annotations": { "readOnlyHint": true, "openWorldHint": false },
    })
}

/// Reports each post's visits in the period the arguments ask for.
pub async fn call(cx: &Cx, arguments: Value) -> Result<Value, ToolError> {
    let arguments: PeriodArguments = parse(arguments)?;
    let (first, last) = period(&arguments, analytics::today(), analytics::earliest_day())?;
    let pages = PageStats::between(&db(cx), first, last).await?;
    let all_time = PostView::by_slug(cx).await?;
    let posts = Post::list().await?;
    let posts: Vec<_> = posts
        .iter()
        .map(|post| {
            let path = href!(blog::slug::page, blog::slug::Slug(&post.slug)).resolve(cx);
            Performance {
                post,
                url: seo::post_url(cx, post),
                stats: pages.get(&path),
            }
        })
        .collect();
    Ok(json!({
        "from": first.to_string(),
        "to": last.to_string(),
        "days": (last - first).get_days() + 1,
        "posts": performances(posts, &all_time),
    }))
}

/// A post and its visits in the period, if it had any.
struct Performance<'a> {
    post: &'a Post,
    url: String,
    stats: Option<&'a PageStats>,
}

/// The posts as the tool gives them, the most read first, and the newest first among
/// those read alike. `posts` come newest first.
fn performances(mut posts: Vec<Performance>, all_time: &HashMap<String, u64>) -> Vec<Value> {
    let none = PageStats::default();
    posts.sort_by_key(|performance| {
        let stats = performance.stats.unwrap_or(&none);
        std::cmp::Reverse((stats.visitors, stats.views))
    });
    posts
        .iter()
        .map(|Performance { post, url, stats }| {
            let stats = stats.unwrap_or(&none);
            let sources: Vec<_> = stats
                .sources
                .iter()
                .map(|(host, visitors)| json!({ "source": host.as_deref().unwrap_or("direct"), "visitors": visitors }))
                .collect();
            let campaigns: Vec<_> = stats
                .campaigns
                .iter()
                .map(|(campaign, visitors)| json!({ "utm_source": campaign, "visitors": visitors }))
                .collect();
            json!({
                "slug": post.slug,
                "title": post.title,
                "url": url,
                "published": post.date_iso(),
                "updated": post.updated_iso(),
                "tags": post.tags,
                "views": stats.views,
                "visitors": stats.visitors,
                "entries": stats.entries,
                "sources": sources,
                "campaigns": campaigns,
                "all_time_views": all_time.get(&post.slug).copied().unwrap_or(0),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_every_post_the_most_read_first() {
        let posts = Post::list().await.unwrap();
        assert!(posts.len() >= 3, "the tests read the blog's own posts");
        let (newest, middle, oldest) = (&posts[0], &posts[1], &posts[2]);
        let read = PageStats {
            views: 9,
            visitors: 4,
            entries: 3,
            sources: vec![(Some("news.ycombinator.com".to_owned()), 2), (None, 1)],
            campaigns: vec![("linkedin".to_owned(), 1)],
        };
        let less = PageStats {
            views: 2,
            visitors: 1,
            ..PageStats::default()
        };
        let performance = |post, stats| Performance {
            post,
            url: format!("https://vitor.ws/blog/{}", post.slug),
            stats,
        };
        let all_time = HashMap::from([(oldest.slug.clone(), 120)]);
        let listed = performances(
            vec![
                performance(newest, None),
                performance(middle, Some(&less)),
                performance(oldest, Some(&read)),
            ],
            &all_time,
        );
        let slugs: Vec<_> = listed
            .iter()
            .map(|post| post["slug"].as_str().unwrap())
            .collect();
        assert_eq!(slugs, [&oldest.slug, &middle.slug, &newest.slug]);
        let top = &listed[0];
        assert_eq!(top["entries"], 3);
        assert_eq!(top["sources"][0]["source"], "news.ycombinator.com");
        assert_eq!(top["sources"][1]["source"], "direct");
        assert_eq!(top["campaigns"][0]["utm_source"], "linkedin");
        assert_eq!(top["all_time_views"], 120);
        assert_eq!(top["published"], oldest.date_iso());
        assert_eq!(listed[2]["visitors"], 0);
        assert_eq!(listed[2]["all_time_views"], 0);
    }
}
