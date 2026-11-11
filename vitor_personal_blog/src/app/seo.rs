//! What the site tells search engines and feed readers beyond `<head>`: structured
//! data about the site, its author and its posts, and the RSS feed.

use std::fmt::Write;

use jiff::fmt::rfc2822::DateTimePrinter;
use serde_json::{Value, json};
use topcoat::{
    Result,
    asset::asset_config,
    context::{Cx, base_url},
    router::href,
    view::{Unescaped, View, component, view},
};

use super::{AUTHOR, DESCRIPTION, GITHUB, SITE_NAME, SOCIAL_IMAGE, blog};
use crate::features::post::Post;

/// The social preview image's absolute URL, which link previews require.
pub fn image_url(cx: &Cx) -> String {
    base_url(cx).join(&asset_config(cx).resolve(SOCIAL_IMAGE))
}

/// Structured data for search engines, as JSON-LD.
#[component]
pub async fn structured_data(data: Value) -> Result<impl View> {
    // A `</script>` inside a string would end the element early, so `<`, `>` and `&`
    // go out as JSON escapes, which read back as the same characters.
    let json = data
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    Ok(view! {
        <script type="application/ld+json">(Unescaped::new_unchecked(json))</script>
    })
}

/// The site and its author, for the home page.
pub fn home(cx: &Cx) -> Value {
    let home = home_url(cx);
    json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "WebSite",
                "@id": format!("{home}#website"),
                "name": SITE_NAME,
                "alternateName": AUTHOR,
                "url": home,
                "description": DESCRIPTION,
                "inLanguage": "en",
                "publisher": { "@id": person_id(cx) },
            },
            {
                "@type": "Person",
                "@id": person_id(cx),
                "name": AUTHOR,
                "url": home,
                "image": image_url(cx),
                "jobTitle": "Software Engineer",
                "worksFor": {
                    "@type": "Organization",
                    "name": "RockawayX",
                    "url": "https://www.rockawayx.com/",
                },
                "knowsAbout": ["Rust", "Cryptocurrency", "Artificial intelligence"],
                "sameAs": [GITHUB],
            },
        ],
    })
}

/// The blog and its posts, for the blog page.
pub fn blog(cx: &Cx, posts: &[Post]) -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": "Blog",
        "@id": format!("{}#blog", blog_url(cx)),
        "name": format!("{SITE_NAME} blog"),
        "url": blog_url(cx),
        "inLanguage": "en",
        "author": author(cx),
        "isPartOf": { "@id": format!("{}#website", home_url(cx)) },
        "blogPost": posts.iter().map(|post| json!({
            "@type": "BlogPosting",
            "headline": post.title,
            "url": post_url(cx, post),
            "datePublished": post.published_at().map(|at| at.to_string()),
        })).collect::<Vec<_>>(),
    })
}

/// The post as an article, and where it sits in the site, for its page.
pub fn post(cx: &Cx, post: &Post) -> Value {
    let url = post_url(cx, post);
    let description = post.description.as_ref().or(post.subtitle.as_ref());
    json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "BlogPosting",
                "@id": format!("{url}#article"),
                "headline": post.title,
                "description": description,
                "url": url,
                "mainEntityOfPage": url,
                "datePublished": post.published_at().map(|at| at.to_string()),
                "dateModified": post.modified_at().map(|at| at.to_string()),
                "author": author(cx),
                "publisher": author(cx),
                "image": image_url(cx),
                "keywords": post.tags,
                "inLanguage": "en",
                "isPartOf": { "@id": format!("{}#blog", blog_url(cx)) },
            },
            {
                "@type": "BreadcrumbList",
                "itemListElement": [
                    { "@type": "ListItem", "position": 1, "name": "Home", "item": home_url(cx) },
                    { "@type": "ListItem", "position": 2, "name": "Blog", "item": blog_url(cx) },
                    { "@type": "ListItem", "position": 3, "name": post.title, "item": url },
                ],
            },
        ],
    })
}

/// The published posts as an RSS 2.0 feed, newest first.
pub fn rss(cx: &Cx, posts: &[Post]) -> String {
    let home = home_url(cx);
    let feed = href!(super::feed).absolute().resolve(cx);
    let date = |post: &Post| {
        post.published_at()
            .and_then(|at| DateTimePrinter::new().timestamp_to_rfc9110_string(&at).ok())
    };

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n",
    );
    let _ = writeln!(xml, "<title>{}</title>", escape(SITE_NAME));
    let _ = writeln!(xml, "<link>{}</link>", escape(&home));
    let _ = writeln!(xml, "<description>{}</description>", escape(DESCRIPTION));
    xml.push_str("<language>en</language>\n");
    let _ = writeln!(
        xml,
        "<atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\"/>",
        escape(&feed)
    );
    if let Some(latest) = posts.first().and_then(date) {
        let _ = writeln!(xml, "<lastBuildDate>{latest}</lastBuildDate>");
    }

    for post in posts {
        let url = escape(&post_url(cx, post));
        xml.push_str("<item>\n");
        let _ = writeln!(xml, "<title>{}</title>", escape(&post.title));
        let _ = writeln!(xml, "<link>{url}</link>");
        let _ = writeln!(xml, "<guid isPermaLink=\"true\">{url}</guid>");
        if let Some(published) = date(post) {
            let _ = writeln!(xml, "<pubDate>{published}</pubDate>");
        }
        if let Some(description) = post.description.as_ref().or(post.subtitle.as_ref()) {
            let _ = writeln!(xml, "<description>{}</description>", escape(description));
        }
        for tag in &post.tags {
            let _ = writeln!(xml, "<category>{}</category>", escape(tag));
        }
        xml.push_str("</item>\n");
    }

    xml.push_str("</channel>\n</rss>\n");
    xml
}

/// The author, as the posts credit them.
fn author(cx: &Cx) -> Value {
    json!({
        "@type": "Person",
        "@id": person_id(cx),
        "name": AUTHOR,
        "url": home_url(cx),
    })
}

fn person_id(cx: &Cx) -> String {
    format!("{}#person", home_url(cx))
}

fn home_url(cx: &Cx) -> String {
    href!(super::home).absolute().resolve(cx)
}

fn blog_url(cx: &Cx) -> String {
    href!(blog::page).absolute().resolve(cx)
}

/// The post's absolute URL, such as `https://vitor.ws/blog/my_post`.
pub fn post_url(cx: &Cx, post: &Post) -> String {
    href!(blog::slug::page, blog::slug::Slug(&post.slug))
        .absolute()
        .resolve(cx)
}

/// Escapes text for an XML element or a double-quoted attribute.
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn escape_covers_markup_and_quotes() {
        assert_eq!(
            escape(r#"Axum & "HTMX" <3 it's"#),
            "Axum &amp; &quot;HTMX&quot; &lt;3 it&apos;s"
        );
    }
}
