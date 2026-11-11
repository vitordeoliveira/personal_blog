//! The `read_post` tool: a published post's text, for an agent to write about it.

use serde::Deserialize;
use serde_json::{Value, json};
use topcoat::context::{Cx, base_url};

use super::common::{ToolError, parse, refuse};
use crate::{app::seo, features::post::Post};

/// The name agents call the tool by.
pub const NAME: &str = "read_post";

/// What the tool does, for the agent deciding whether to call it.
const DESCRIPTION: &str = "Gives a published post of the blog, https://vitor.ws: its \
    title, subtitle, the description search results and link previews show, address, \
    publication and update dates, tags, and its whole text in Markdown, with its links \
    and images made absolute. A post's slug is the end of its address, such as \
    how_use_axum_askama_htmx in https://vitor.ws/blog/how_use_axum_askama_htmx; \
    post_performance lists every post with its slug.";

/// The tool's name, description and arguments, for `tools/list`.
pub fn definition() -> Value {
    json!({
        "name": NAME,
        "title": "Read a post",
        "description": DESCRIPTION,
        "inputSchema": {
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "The post's slug, such as how_use_axum_askama_htmx.",
                },
            },
            "required": ["slug"],
            "additionalProperties": false,
        },
        "annotations": { "readOnlyHint": true, "openWorldHint": false },
    })
}

/// The tool's arguments, as the agent sends them.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Arguments {
    slug: String,
}

/// Gives the post the arguments name.
pub async fn call(cx: &Cx, arguments: Value) -> Result<Value, ToolError> {
    let Arguments { slug } = parse(arguments)?;
    let Some(post) = Post::get(&slug).await? else {
        return Err(refuse(format!(
            "There is no published post {slug}. post_performance lists every post with its slug."
        )));
    };
    Ok(describe(
        &post,
        seo::post_url(cx, &post),
        base_url(cx).as_str(),
    ))
}

/// `post` as the tool gives it, at `url`, with its links made absolute on `origin`.
fn describe(post: &Post, url: String, origin: &str) -> Value {
    json!({
        "slug": post.slug,
        "title": post.title,
        "subtitle": post.subtitle,
        "description": post.description,
        "url": url,
        "published": post.date_iso(),
        "updated": post.updated_iso(),
        "tags": post.tags,
        "markdown": absolute_links(post.markdown(), origin),
    })
}

/// `markdown` with the links and images that point into the site, such as
/// `/blog_assets/chart.png`, made absolute on `origin`, so they work wherever the text
/// goes.
fn absolute_links(markdown: &str, origin: &str) -> String {
    ["](/", "src=\"/", "href=\"/"]
        .into_iter()
        .fold(markdown.to_owned(), |text, start| {
            let absolute = format!("{}{origin}/", &start[..start.len() - 1]);
            text.replace(start, &absolute)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_the_sites_links_absolute() {
        let markdown = "![chart](/blog_assets/chart.png) [home](/) [rust](https://www.rust-lang.org/) \
            <img src=\"/blog_assets/a.png\"> <a href=\"/blog\">blog</a> // comment";
        assert_eq!(
            absolute_links(markdown, "https://vitor.ws"),
            "![chart](https://vitor.ws/blog_assets/chart.png) [home](https://vitor.ws/) \
            [rust](https://www.rust-lang.org/) <img src=\"https://vitor.ws/blog_assets/a.png\"> \
            <a href=\"https://vitor.ws/blog\">blog</a> // comment"
        );
    }

    #[tokio::test]
    async fn gives_a_published_post_without_its_front_matter() {
        let newest = &Post::list().await.unwrap()[0];
        let url = format!("https://vitor.ws/blog/{}", newest.slug);
        let read = describe(newest, url.clone(), "https://vitor.ws");
        assert_eq!(read["title"], newest.title);
        assert_eq!(read["url"], url);
        let markdown = read["markdown"].as_str().unwrap();
        assert!(!markdown.is_empty());
        assert!(!markdown.contains("](/"), "{markdown}");
        assert!(!markdown.contains("ready:"), "{markdown}");
    }
}
