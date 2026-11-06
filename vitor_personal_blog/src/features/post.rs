//! The blog posts, read from the markdown files in `posts/`.

use std::io::ErrorKind;
use std::path::Path;

use std::collections::HashMap;

use jiff::civil::{Date, DateTime};
use jiff::{Timestamp, tz::TimeZone};
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};
use serde::Deserialize;
use topcoat::Result;

/// The directory with one markdown file per post, named after the post's slug.
const POSTS_DIR: &str = "posts";

/// A published post.
#[derive(Debug)]
pub struct Post {
    pub slug: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    /// The date as written in the front matter, such as `2024-04-03t17:52:00`.
    pub date: String,
    /// When the post last changed in a way worth telling readers, written like `date`.
    pub updated: Option<String>,
    pub tags: Vec<String>,
    markdown: String,
}

/// The YAML block at the top of a post file.
#[derive(Deserialize)]
struct FrontMatter {
    title: String,
    subtitle: Option<String>,
    description: Option<String>,
    date: String,
    updated: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    /// Posts stay hidden until they are marked ready.
    #[serde(default)]
    ready: bool,
}

impl Post {
    /// Returns every published post, newest first.
    pub async fn list() -> Result<Vec<Post>> {
        let mut entries = tokio::fs::read_dir(POSTS_DIR).await?;
        let mut posts = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_none_or(|extension| extension != "md") {
                continue;
            }
            let Some(slug) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let source = tokio::fs::read_to_string(&path).await?;
            if let Some(post) = parse(slug, &source)? {
                posts.push(post);
            }
        }

        posts.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(posts)
    }

    /// Returns the published post with the given slug, if there is one.
    pub async fn get(slug: &str) -> Result<Option<Post>> {
        // The slug comes from the URL. Accepting only file name characters keeps the
        // lookup inside `posts/`.
        let is_file_name = slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if slug.is_empty() || !is_file_name {
            return Ok(None);
        }

        let path = Path::new(POSTS_DIR).join(format!("{slug}.md"));
        match tokio::fs::read_to_string(&path).await {
            Ok(source) => parse(slug, &source),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// The date part of [`Post::date`], such as `2024-04-03`, for `<time datetime>`.
    pub fn date_iso(&self) -> &str {
        self.date.get(..10).unwrap_or(&self.date)
    }

    /// The date for readers, such as `April 3, 2024`.
    pub fn date_long(&self) -> String {
        long_date(self.date_iso())
    }

    /// The date part of [`Post::updated`], such as `2024-04-03`, if the post has one.
    pub fn updated_iso(&self) -> Option<&str> {
        let updated = self.updated.as_deref()?;
        Some(updated.get(..10).unwrap_or(updated))
    }

    /// When the post was published, reading the front matter date as UTC.
    pub fn published_at(&self) -> Option<Timestamp> {
        timestamp(&self.date)
    }

    /// When the post last changed: its `updated` date, or else its publication.
    pub fn modified_at(&self) -> Option<Timestamp> {
        self.updated
            .as_deref()
            .and_then(timestamp)
            .or_else(|| self.published_at())
    }

    /// The body rendered to HTML.
    ///
    /// Each heading gets an `id` from its text, so a link can point at a section and
    /// search results can jump to it. Images load lazily, since most sit below the
    /// fold. The markdown is written by the blog's author, so its inline HTML is
    /// trusted.
    pub fn html(&self) -> String {
        let options =
            Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
        let events = with_heading_ids(Parser::new_ext(&self.markdown, options).collect());
        let mut body = String::new();
        html::push_html(&mut body, events.into_iter());
        body.replace("<img ", r#"<img loading="lazy" decoding="async" "#)
    }
}

/// A `2024-04-03` date for readers, such as `April 3, 2024`.
pub fn long_date(iso: &str) -> String {
    match iso.parse::<Date>() {
        Ok(date) => date.strftime("%B %-d, %Y").to_string(),
        Err(_) => iso.to_owned(),
    }
}

/// Reads a front matter date, such as `2024-04-03t17:52:00` or `2024-04-03`, as UTC.
fn timestamp(date: &str) -> Option<Timestamp> {
    let date_time = date
        .parse::<DateTime>()
        .or_else(|_| date.parse::<Date>().map(|date| date.at(0, 0, 0, 0)))
        .ok()?;
    Some(date_time.to_zoned(TimeZone::UTC).ok()?.timestamp())
}

/// Gives every heading without an `id` one made from its text, unique in the post.
fn with_heading_ids(mut events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut used = HashMap::<String, usize>::new();
    let mut start = None;
    let mut text = String::new();

    for index in 0..events.len() {
        match &events[index] {
            Event::Start(Tag::Heading { id: None, .. }) => {
                start = Some(index);
                text.clear();
            }
            Event::Text(part) | Event::Code(part) if start.is_some() => text.push_str(part),
            Event::End(TagEnd::Heading(_)) => {
                let Some(start) = start.take() else { continue };
                let base = slugify(&text);
                if base.is_empty() {
                    continue;
                }
                let count = used.entry(base.clone()).or_insert(0);
                *count += 1;
                let id = match *count {
                    1 => base,
                    n => format!("{base}-{n}"),
                };
                if let Event::Start(Tag::Heading { id: slot, .. }) = &mut events[start] {
                    *slot = Some(CowStr::from(id));
                }
            }
            _ => {}
        }
    }
    events
}

/// Lowercases `text` and joins its words with `-`, such as `what-is-htmx`.
fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_owned()
}

/// Builds a post from its file contents, or returns `None` if it is not ready.
fn parse(slug: &str, source: &str) -> Result<Option<Post>> {
    // Without front matter there is no `ready: true`, so the file is not a post yet.
    let Some((front_matter, markdown)) = split_front_matter(source) else {
        return Ok(None);
    };
    let front_matter: FrontMatter = serde_yaml_ng::from_str(front_matter)?;

    if !front_matter.ready {
        return Ok(None);
    }

    // A multi-line YAML string can end in a space, which would show in `<head>`.
    let trimmed = |text: Option<String>| text.map(|text| text.trim().to_owned());
    Ok(Some(Post {
        slug: slug.to_owned(),
        title: front_matter.title,
        subtitle: trimmed(front_matter.subtitle),
        description: trimmed(front_matter.description),
        date: front_matter.date,
        updated: front_matter.updated,
        tags: front_matter.tags,
        markdown: markdown.to_owned(),
    }))
}

/// Splits a file into the YAML between its leading `---` lines and the markdown after.
fn split_front_matter(source: &str) -> Option<(&str, &str)> {
    let rest = source.strip_prefix("---")?;
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))?;
    let end = rest.find("\n---")?;
    let markdown = rest[end + 4..]
        .split_once('\n')
        .map_or("", |(_, markdown)| markdown);
    Some((&rest[..end], markdown))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(markdown: &str) -> Post {
        Post {
            slug: "post".to_owned(),
            title: "Post".to_owned(),
            subtitle: None,
            description: None,
            date: "2024-04-03t17:52:00".to_owned(),
            updated: None,
            tags: Vec::new(),
            markdown: markdown.to_owned(),
        }
    }

    #[test]
    fn headings_get_ids_from_their_text() {
        let html = post("## Why use `axum`?\n\n### Why use `axum`?\n\n## \u{1F680}\n").html();
        assert!(html.contains(r#"<h2 id="why-use-axum">"#), "{html}");
        assert!(html.contains(r#"<h3 id="why-use-axum-2">"#), "{html}");
        // A heading without words keeps no id.
        assert!(html.contains("<h2>"), "{html}");
    }

    #[test]
    fn headings_with_inline_html_use_only_their_text() {
        let html = post("## <a name=\"what-is-htmx\"></a> What is HTMX?\n").html();
        assert!(html.contains(r#"<h2 id="what-is-htmx">"#), "{html}");
    }

    #[test]
    fn images_load_lazily() {
        let html = post("![a diagram](/blog_assets/diagram.png)\n").html();
        assert!(
            html.contains(r#"<img loading="lazy" decoding="async" src="/blog_assets/diagram.png""#),
            "{html}"
        );
    }

    #[test]
    fn dates_read_with_or_without_a_time() {
        assert_eq!(
            timestamp("2024-04-03t17:52:00").unwrap().to_string(),
            "2024-04-03T17:52:00Z"
        );
        assert_eq!(
            timestamp("2024-04-03").unwrap().to_string(),
            "2024-04-03T00:00:00Z"
        );
        assert_eq!(timestamp("soon"), None);
    }

    #[test]
    fn modified_falls_back_to_the_publication() {
        let mut post = post("");
        assert_eq!(post.modified_at(), post.published_at());
        assert_eq!(post.updated_iso(), None);

        post.updated = Some("2024-05-10t09:30:00".to_owned());
        assert_eq!(
            post.modified_at().unwrap().to_string(),
            "2024-05-10T09:30:00Z"
        );
        assert_eq!(post.updated_iso(), Some("2024-05-10"));
        assert_eq!(long_date("2024-05-10"), "May 10, 2024");
    }
}
