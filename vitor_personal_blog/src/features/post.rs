//! The blog posts, read from the markdown files in `posts/`.

use std::io::ErrorKind;
use std::path::Path;

use jiff::civil::{Date, DateTime};
use jiff::{Timestamp, tz::TimeZone};
use pulldown_cmark::{Options, Parser, html};
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
        match self.date_iso().parse::<Date>() {
            Ok(date) => date.strftime("%B %-d, %Y").to_string(),
            Err(_) => self.date.clone(),
        }
    }

    /// When the post was published, reading the front matter date as UTC.
    pub fn published_at(&self) -> Option<Timestamp> {
        let date_time = self.date.parse::<DateTime>().ok()?;
        Some(date_time.to_zoned(TimeZone::UTC).ok()?.timestamp())
    }

    /// The body rendered to HTML.
    ///
    /// The markdown is written by the blog's author, so its inline HTML is trusted.
    pub fn html(&self) -> String {
        let options =
            Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
        let mut body = String::new();
        html::push_html(&mut body, Parser::new_ext(&self.markdown, options));
        body
    }
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

    Ok(Some(Post {
        slug: slug.to_owned(),
        title: front_matter.title,
        subtitle: front_matter.subtitle,
        description: front_matter.description,
        date: front_matter.date,
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
