pub mod slug;

use std::collections::HashMap;

use topcoat::{
    Result,
    context::Cx,
    router::{Body, Next, href, layer, page, response::Response},
    view::{View, component, view},
};

use super::{PageMeta, seo};

use crate::components::badge::{BadgeVariant, badge};
use crate::features::post::Post;
use crate::features::post_view::{PostView, views_label};

/// Names the blog's pages in `<head>`. The post page's layer replaces it with the post.
#[layer]
async fn blog_meta(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let cx = cx.with(PageMeta {
        title: "Blog: Rust, Topcoat and AI".to_owned(),
        description: "Posts by Vitor de Oliveira on Rust, Topcoat, self-hosting and AI: the \
                      projects I build, and why I pick the technologies behind them"
            .to_owned(),
        ..PageMeta::default()
    });
    next.run(&cx, body).await
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let posts = Post::list().await?;
    let views = PostView::by_slug(cx).await?;

    Ok(view! {
        seo::structured_data(data: seo::blog(cx, &posts))
        <section class="mx-auto max-w-3xl px-4 py-16 sm:px-6">
            <h1 class="text-3xl font-bold tracking-tight md:text-4xl">"Blog"</h1>
            if posts.is_empty() {
                <p class="mt-6 text-muted-foreground">
                    "No posts yet. Check back soon!"
                </p>
            } else {
                post_list(posts: posts, views: views, heading: "h2")
            }
        </section>
    })
}

/// Posts as links with their date, view count, description, and tags.
///
/// `views` holds the counts by slug, from [`PostView::by_slug`]. `heading` is the
/// element for each title, so the list fits under the page's outline.
#[component]
pub async fn post_list(
    posts: Vec<Post>,
    views: HashMap<String, u64>,
    heading: &'static str,
) -> Result<impl View> {
    Ok(view! {
        <ul class="mt-8 flex flex-col gap-2">
            for post in posts {
                <li>
                    <a
                        href=(href!(slug::page, slug::Slug(&post.slug)))
                        class="group -mx-4 block rounded-xl px-4 py-5 transition-colors hover:bg-foreground/5"
                    >
                        <p
                            class="flex items-center gap-2 text-sm text-muted-foreground"
                        >
                            <time datetime=(post.date_iso())>(post.date_long())</time>
                            <span aria-hidden="true">"•"</span>
                            views_label(
                                views: views.get(&post.slug).copied().unwrap_or(0)
                            )
                        </p>
                        <(heading)
                            class="mt-1 text-xl font-semibold tracking-tight transition-colors group-hover:text-primary"
                        >
                            (&post.title)
                        </(heading)>
                        if let Some(description) = &post.description {
                            <p class="mt-2 leading-relaxed text-muted-foreground">
                                (description)
                            </p>
                        }
                        if !post.tags.is_empty() {
                            <div class="mt-3 flex flex-wrap gap-1.5">
                                for tag in &post.tags {
                                    badge(variant: BadgeVariant::Secondary, (tag))
                                }
                            </div>
                        }
                    </a>
                </li>
            }
        </ul>
    })
}
