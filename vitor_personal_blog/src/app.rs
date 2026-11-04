mod admin;
mod blog;
mod privacy;

use topcoat::{
    Result,
    asset::{Asset, AssetBundle, RouterBuilderAssetExt, asset},
    context::{Cx, base_url, try_request_context},
    cookie::RouterBuilderCookieExt,
    font::{Font, fontsource::fontsource_font},
    icon::{icon, iconify::iconify_icon},
    router::{
        RouterBuilder, RouterBuilderDirectoryExt, RouterBuilderDiscoverExt, Slot, StatusCode,
        content::sitemap::{ChangeFrequency, Sitemap, SitemapUrl},
        error::NotFoundError,
        href, layout, module_router, not_found, page,
        request::uri,
        route,
    },
    runtime::RouterBuilderRuntimeExt,
    session::{RouterBuilderSessionExt, SessionConfig},
    tailwind,
    view::{View, attributes, class, component, error_boundary, view},
};

use crate::components::button::{ButtonSize, ButtonVariant, button_variants};
use crate::features::analytics::{self, Analytics};
use crate::features::post::Post;
use crate::features::post_view::PostView;
use crate::features::theme::{Theme, theme_script, theme_toggle};

/// The Geist font family, loaded from Fontsource.
const GEIST: Font = fontsource_font!(GEIST, weight: [400, 500, 600, 700], style: Normal);

const FAVICON: Asset = asset!("assets/icon.svg");

/// The home screen icon for iOS, which ignores SVG icons. It is the favicon as a
/// full-bleed 180px square, since iOS rounds the corners itself.
const APPLE_TOUCH_ICON: Asset = asset!("assets/apple-touch-icon.png");

/// How many posts the home page lists.
const RECENT_POSTS: usize = 5;

/// Where the site is published, for canonical links and the sitemap. `SITE_URL`
/// overrides it.
const SITE_URL: &str = "https://vitor.ws";

/// The description of pages that do not have their own.
const DESCRIPTION: &str =
    "Blog by Vitor de Oliveira - JavaScript, Rust & AI specialist with 10+ years of experience";

/// What `<head>` says about the current page.
///
/// A layer registers it in the request context for pages with their own title or
/// description. Other pages get [`PageMeta::default`].
#[derive(Clone, Debug)]
pub struct PageMeta {
    /// The page's name, which `<title>` follows with ` | vitor.ws`.
    pub title: String,
    pub description: String,
    /// For a post, its publication date, which marks the page as an article.
    pub published: Option<String>,
}

impl Default for PageMeta {
    fn default() -> Self {
        Self {
            title: "Vitor de Oliveira".to_owned(),
            description: DESCRIPTION.to_owned(),
            published: None,
        }
    }
}

// Unmatched URLs render the not-found page inside the root layout.
not_found!();

/// Builds the application's router.
pub fn router() -> RouterBuilder {
    // The pages and layouts of this module and its submodules, with paths derived
    // from the module tree.
    module_router!()
        // The runtime's procedures and shards, which have no place in the module tree.
        .discover()
        .assets(AssetBundle::load().unwrap())
        // The images the posts link to, at the fixed URLs written in their markdown.
        .serve_dir("/blog_assets/{*file}", "./public/blog_assets")
        // Turns paths into absolute URLs for canonical links and the sitemap.
        .base_url(std::env::var("SITE_URL").unwrap_or_else(|_| SITE_URL.to_owned()))
        // Lets the theme and the analytics read and remember the visitor's choices.
        .cookies()
        // The admin's session, carried in a cookie.
        .sessions(SessionConfig::default())
        // Lets the browser runtime rerun pages; register it after other layers.
        .runtime()
}

#[layout]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let dark = Theme::from_cookie(cx) == Some(Theme::Dark);
    let meta = try_request_context::<PageMeta>(cx)
        .cloned()
        .unwrap_or_default();
    let path = uri(cx).path();
    let url = base_url(cx).join(path);
    let admin = path == "/admin" || path.starts_with("/admin/");
    // This layout also runs for a missing page, so only the site's pages count as visits.
    if counts_as_visit(path).await {
        Analytics::get(cx).record(cx);
    }
    // The privacy page asks the same itself, and the admin is not a visitor.
    let ask = !admin && path != "/privacy" && analytics::choice(cx).is_none();

    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" class=(dark.then_some("dark"))>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>
                    (&meta.title)
                    " | vitor.ws"
                </title>
                <meta name="description" content=(&meta.description)>
                if admin {
                    <meta name="robots" content="noindex">
                }
                <link rel="canonical" href=(&url)>
                // Open Graph and Twitter tags shape the preview when a link is shared.
                <meta property="og:site_name" content="vitor.ws">
                <meta
                    property="og:type"
                    content=(if meta.published.is_some() {
                        "article"
                    } else {
                        "website"
                    })
                >
                <meta property="og:title" content=(&meta.title)>
                <meta property="og:description" content=(&meta.description)>
                <meta property="og:url" content=(&url)>
                if let Some(published) = &meta.published {
                    <meta property="article:published_time" content=(published)>
                }
                <meta name="twitter:card" content="summary">
                <link rel="icon" type="image/svg+xml" href=(FAVICON)>
                <link rel="apple-touch-icon" href=(APPLE_TOUCH_ICON)>
                // Before the stylesheet, so the first paint already has the right scheme.
                theme_script()
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::font::link(font: GEIST)
                topcoat::runtime::script()
                topcoat::dev::script()
            </head>
            <body class="flex min-h-screen flex-col antialiased">
                site_header()
                <main class="flex-1">
                    error_boundary(
                        fallback: |error| {
                            if error.downcast_ref::<NotFoundError>().is_none() {
                                return Err(error);
                            }

                            Ok(
                                view! {
                                    (StatusCode::NOT_FOUND)
                                    not_found_message()
                                },
                            )
                        },
                        (slot)
                    )
                </main>
                site_footer()
                if ask {
                    privacy::cookie_banner()
                }
            </body>
        </html>
    })
}

/// Whether `path` is one of the site's pages, which count as visits: the home page, the
/// blog, and each published post. A new page goes here too.
async fn counts_as_visit(path: &str) -> bool {
    match path {
        "/" | "/blog" => true,
        _ => match path.strip_prefix("/blog/") {
            Some(slug) => matches!(Post::get(slug).await, Ok(Some(_))),
            None => false,
        },
    }
}

/// Tells crawlers they may index everything and where the sitemap is.
#[route(GET "./robots.txt")]
async fn robots(cx: &Cx) -> Result<String> {
    let sitemap_url = href!(sitemap).absolute().resolve(cx);
    Ok(format!(
        "User-Agent: *\nAllow: /\n\nSitemap: {sitemap_url}\n"
    ))
}

/// Lists the home page, the blog, and every published post for crawlers.
#[route(GET "./sitemap.xml")]
async fn sitemap(cx: &Cx) -> Result<Sitemap> {
    let posts = Post::list().await?;
    // The site changes when a post is published.
    let latest = posts.first().and_then(Post::published_at);

    let mut home_url = SitemapUrl::new(href!(home).resolve(cx))
        .change_frequency(ChangeFrequency::Weekly)
        .priority(1.0);
    let mut blog_url = SitemapUrl::new(href!(blog::page).resolve(cx))
        .change_frequency(ChangeFrequency::Weekly)
        .priority(0.8);
    if let Some(latest) = latest {
        home_url = home_url.last_modified(latest);
        blog_url = blog_url.last_modified(latest);
    }

    Ok(Sitemap::new()
        .url(home_url)
        .url(blog_url)
        .urls(posts.iter().map(|post| {
            let url =
                SitemapUrl::new(href!(blog::slug::page, blog::slug::Slug(&post.slug)).resolve(cx))
                    .change_frequency(ChangeFrequency::Monthly)
                    .priority(0.6);
            match post.published_at() {
                Some(published) => url.last_modified(published),
                None => url,
            }
        })))
}

#[component]
async fn site_header(cx: &Cx) -> Result<impl View> {
    let blog_link = href!(blog::page);
    let on_blog = blog_link.is_current(cx);

    Ok(view! {
        <header
            class="sticky top-0 z-50 border-b border-border bg-background/80 backdrop-blur-lg"
        >
            <div
                class="mx-auto flex h-16 max-w-3xl items-center justify-between px-4 sm:px-6"
            >
                <a
                    href=(href!(home))
                    class="text-xl font-bold tracking-tight text-primary"
                >
                    "vitor.ws"
                </a>
                <nav class="flex items-center gap-2">
                    <a
                        href=(blog_link)
                        aria-current=(on_blog.then_some("page"))
                        class=(class!(
                            button_variants(ButtonVariant::Ghost, ButtonSize::Sm),
                            "aria-[current=page]:text-primary",
                        ))
                    >
                        "Blog"
                    </a>
                    theme_toggle()
                </nav>
            </div>
        </header>
    })
}

#[component]
async fn site_footer() -> Result<impl View> {
    let year = jiff::Zoned::now().year();

    Ok(view! {
        <footer class="border-t border-border">
            <div
                class="mx-auto flex max-w-3xl flex-col items-center gap-2 px-4 py-8 text-sm text-muted-foreground sm:flex-row sm:justify-between sm:px-6"
            >
                <p>
                    "© "
                    (year)
                    " Vitor de Oliveira"
                </p>
                <div class="flex items-center gap-4">
                    <a
                        href=(href!(privacy::page))
                        class="transition-colors hover:text-primary"
                    >
                        "Privacy"
                    </a>
                    <a
                        href="mailto:vitordeoliveira50@gmail.com"
                        class="transition-colors hover:text-primary"
                    >
                        "vitordeoliveira50@gmail.com"
                    </a>
                </div>
            </div>
        </footer>
    })
}

#[component]
async fn not_found_message() -> Result<impl View> {
    Ok(view! {
        <section
            class="mx-auto flex max-w-3xl flex-col items-center px-4 py-24 text-center sm:px-6"
        >
            <p class="text-sm font-medium text-primary">"404"</p>
            <h1 class="mt-2 text-3xl font-bold tracking-tight">"Page not found"</h1>
            <p class="mt-3 text-muted-foreground">
                "The page you're looking for doesn't exist."
            </p>
            <a
                href=(href!(blog::page))
                class=(class!(
                    button_variants(ButtonVariant::Outline, ButtonSize::Md),
                    "mt-8",
                ))
            >
                icon(
                    data: iconify_icon!("lucide:arrow-left"),
                    attrs: attributes! { class="size-4" }
                )
                "Back to the blog"
            </a>
        </section>
    })
}

#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    let mut posts = Post::list().await?;
    posts.truncate(RECENT_POSTS);
    let views = PostView::by_slug(cx).await?;

    let products = [
        (
            iconify_icon!("lucide:users"),
            "Sales Agent Platform",
            "AI-powered sales automation",
        ),
        (
            iconify_icon!("lucide:book-open"),
            "Histories Books Generator",
            "Create personalized stories",
        ),
        (
            iconify_icon!("lucide:headphones"),
            "Support Agents",
            "Intelligent customer support",
        ),
    ];
    let roles = ["Founder & Partner", "Software Engineer", "AI Enthusiast"];

    Ok(view! {
        <section class="mx-auto max-w-3xl px-4 pt-16 pb-12 sm:px-6 md:pt-24">
            <p class="text-sm font-medium text-primary">
                "10+ Years Experience • JavaScript • Rust • AI"
            </p>
            <h1 class="mt-4 text-4xl font-bold tracking-tight md:text-5xl lg:text-6xl">
                "Hi, I'm "
                <span class="text-primary">"Vitor de Oliveira"</span>
            </h1>
            <p class="mt-6 text-lg leading-relaxed text-muted-foreground md:text-xl">
                "With over 10+ years of experience in the tech industry, I've had the privilege of "
                "working with leading companies like "
                <span class="font-medium text-foreground">"Volkswagen"</span>
                ", where I've contributed to innovative projects and cutting-edge solutions."
            </p>
            <p class="mt-4 leading-relaxed text-muted-foreground">
                "I specialize in JavaScript, Rust, and AI, combining these technologies to build "
                "robust, performant, and intelligent applications. Beyond my technical expertise, "
                "I'm also an entrepreneur—founder of "
                <span class="font-medium text-foreground">"3 companies"</span>
                " and partner in another."
            </p>

            <div class="mt-10">
                <p class="flex items-center gap-2 text-sm text-muted-foreground">
                    icon(
                        data: iconify_icon!("lucide:building-2"),
                        attrs: attributes! { class="size-4 text-primary" }
                    )
                    "Founder of"
                    <a
                        href="https://invisible-matrix.com"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="inline-flex items-center gap-1 font-semibold text-primary hover:underline"
                    >
                        "Invisible Matrix"
                        icon(
                            data: iconify_icon!("lucide:external-link"),
                            attrs: attributes! { class="size-3" }
                        )
                    </a>
                </p>
                <ul class="mt-4 grid gap-3 sm:grid-cols-3">
                    for (product_icon, name, description) in products {
                        <li
                            class="flex items-center gap-3 rounded-xl border border-border bg-card p-4 text-card-foreground shadow-sm"
                        >
                            <span
                                class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary"
                            >
                                icon(
                                    data: product_icon,
                                    attrs: attributes! { class="size-5" }
                                )
                            </span>
                            <div>
                                <p class="text-sm font-semibold">(name)</p>
                                <p class="text-xs text-muted-foreground">(description)</p>
                            </div>
                        </li>
                    }
                </ul>
            </div>

            <ul class="mt-8 flex flex-wrap gap-2">
                for role in roles {
                    <li
                        class="rounded-full bg-primary/10 px-3 py-1 text-sm font-medium text-primary"
                    >
                        (role)
                    </li>
                }
            </ul>
        </section>

        if !posts.is_empty() {
            <section class="mx-auto max-w-3xl px-4 pt-8 pb-16 sm:px-6">
                <div class="flex items-baseline justify-between gap-4">
                    <h2 class="text-2xl font-semibold tracking-tight">
                        "Recent posts"
                    </h2>
                    <a
                        href=(href!(blog::page))
                        class="inline-flex items-center gap-1 text-sm font-medium text-primary hover:underline"
                    >
                        "All posts"
                        icon(
                            data: iconify_icon!("lucide:arrow-right"),
                            attrs: attributes! { class="size-4" }
                        )
                    </a>
                </div>
                blog::post_list(posts: posts, views: views, heading: "h3")
            </section>
        }
    })
}
