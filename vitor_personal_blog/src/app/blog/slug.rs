use topcoat::{
    Result,
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    router::{
        Body, Next, error::RouterErrorExt, href, layer, module_param, page, path_param,
        response::Response,
    },
    view::{Unescaped, View, attributes, class, view},
};

use super::super::{Article, PageMeta, seo};
use super::post_list;

use crate::components::badge::{BadgeVariant, badge};
use crate::components::button::{ButtonSize, ButtonVariant, button_variants};
use crate::features::post::{Post, long_date};
use crate::features::post_view::{PostView, view_count};

module_param!(pub slug);

/// How many other posts the end of a post suggests.
const MORE_POSTS: usize = 3;

/// Gives the post page the post's title, description and dates in `<head>`.
#[layer]
async fn post_meta(cx: &Cx, body: Body, next: Next<'_>) -> Result<Response> {
    let Some(post) = Post::get(path_param::<Slug>(cx)).await? else {
        // The page answers with the not-found page.
        return next.run(cx, body).await;
    };

    let description = post.description.clone().or_else(|| post.subtitle.clone());
    let cx = cx.with(PageMeta {
        description: description.unwrap_or_else(|| PageMeta::default().description),
        article: Some(Article {
            published: post.published_at(),
            modified: post.modified_at(),
            tags: post.tags,
        }),
        title: post.title,
    });
    next.run(&cx, body).await
}

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let post = Post::get(path_param::<Slug>(cx)).await?.ok_or_not_found()?;
    // Posts are markdown files from this repository, so their HTML is trusted.
    let body = Unescaped::new_unchecked(post.html());
    // The newest other posts, so readers and crawlers find more than this one.
    let more: Vec<Post> = Post::list()
        .await?
        .into_iter()
        .filter(|other| other.slug != post.slug)
        .take(MORE_POSTS)
        .collect();
    let views = PostView::by_slug(cx).await?;

    Ok(view! {
        seo::structured_data(data: seo::post(cx, &post))
        <article class="mx-auto max-w-3xl px-4 py-12 sm:px-6 md:py-16">
            <a
                href=(href!(super::page))
                class=(class!(
                    button_variants(ButtonVariant::Ghost, ButtonSize::Sm),
                    "-ml-3",
                ))
            >
                icon(
                    data: iconify_icon!("lucide:arrow-left"),
                    attrs: attributes! { class="size-4" }
                )
                "All posts"
            </a>
            <header class="mt-8">
                <p
                    class="flex flex-wrap items-center gap-x-2 text-sm text-muted-foreground"
                >
                    <time datetime=(post.date_iso())>(post.date_long())</time>
                    if let Some(updated) = post.updated_iso() {
                        <span aria-hidden="true">"•"</span>
                        <span>
                            "Updated "
                            <time datetime=(updated)>(long_date(updated))</time>
                        </span>
                    }
                    <span aria-hidden="true">"•"</span>
                    view_count(slug: post.slug.clone())
                </p>
                <h1 class="mt-2 text-3xl font-bold tracking-tight md:text-4xl">
                    (&post.title)
                </h1>
                if let Some(subtitle) = &post.subtitle {
                    <p class="mt-3 text-lg leading-relaxed text-muted-foreground">
                        (subtitle)
                    </p>
                }
                if !post.tags.is_empty() {
                    <div class="mt-5 flex flex-wrap gap-1.5">
                        for tag in &post.tags {
                            badge(variant: BadgeVariant::Secondary, (tag))
                        }
                    </div>
                }
            </header>
            <hr class="my-10 border-border">
            <div class="prose prose-theme max-w-none">(body)</div>
        </article>
        if !more.is_empty() {
            <section class="mx-auto max-w-3xl px-4 pb-16 sm:px-6">
                <h2
                    class="border-t border-border pt-10 text-2xl font-semibold tracking-tight"
                >
                    "Keep reading"
                </h2>
                post_list(posts: more, views: views, heading: "h3")
            </section>
        }
    })
}
