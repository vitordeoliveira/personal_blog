//! How many times each post was read, stored in the database.

use std::collections::HashMap;

use toasty::stmt;
use topcoat::{
    Result,
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    router::error::RouterErrorExt,
    runtime::{connected, connected_untracked, shard},
    view::{View, attributes, component, view},
};

use crate::db::db;
use crate::features::post::Post;

/// The number of times a post was read.
#[derive(Debug, toasty::Model)]
pub struct PostView {
    #[key]
    pub slug: String,

    #[default(0)]
    pub views: u64,
}

impl PostView {
    /// Returns the view count of every post read at least once, by slug.
    pub async fn by_slug(cx: &Cx) -> Result<HashMap<String, u64>> {
        let rows = PostView::all().exec(&mut db(cx)).await?;
        Ok(rows.into_iter().map(|row| (row.slug, row.views)).collect())
    }

    /// Returns how many times the post was read.
    pub async fn count(cx: &Cx, slug: &str) -> Result<u64> {
        let row = PostView::filter_by_slug(slug)
            .first()
            .exec(&mut db(cx))
            .await?;
        Ok(row.map_or(0, |row| row.views))
    }

    /// Counts one more read of the post and returns the new total.
    pub async fn record(cx: &Cx, slug: &str) -> Result<u64> {
        // One statement that creates the row or increments it, so concurrent reads
        // are not lost.
        let row = PostView::upsert_by_slug(slug)
            .views(stmt::increment())
            .exec(&mut db(cx))
            .await?;
        Ok(row.views)
    }
}

/// The post's view count, counting the current read.
///
/// The page's HTTP render shows the stored count and asks the browser to connect.
/// Only a browser running the page connects, so crawlers and link previews are not
/// counted. The connected render counts the read and shows the new total.
#[shard]
pub async fn view_count(cx: &Cx, slug: String) -> Result<impl View> {
    // `connected_untracked` does not ask for another connection, so the browser leaves
    // after this render and a reconnect cannot count the same read twice.
    let views = if connected_untracked(cx) {
        // The slug comes from the browser. Only published posts are counted.
        Post::get(&slug).await?.ok_or_not_found()?;
        PostView::record(cx, &slug).await?
    } else {
        let _ = connected(cx);
        PostView::count(cx, &slug).await?
    };

    Ok(view! { views_label(views: views) })
}

/// A view count with its icon, such as `12 views`.
#[component]
pub async fn views_label(views: u64) -> Result<impl View> {
    Ok(view! {
        <span class="inline-flex items-center gap-1">
            icon(
                data: iconify_icon!("lucide:eye"),
                attrs: attributes! { class="size-3.5" }
            )
            (views)
            if views == 1 {
                " view"
            } else {
                " views"
            }
        </span>
    })
}
