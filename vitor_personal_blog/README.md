# vitor_personal_blog

A [Topcoat](https://github.com/tokio-rs/topcoat) application.

## Development

Install the Topcoat CLI:

```sh
cargo install topcoat-cli
```

Create the database tables by applying the migrations:

```sh
cargo run -- toasty migration apply
```

After changing a model, generate a migration from it and apply it:

```sh
cargo run -- toasty migration generate --name <what_changed>
cargo run -- toasty migration apply
```

The database is `DATABASE_URL`, or else `data.db` in `OXPLOY_STORAGE_DIR`, the storage folder
oxploy gives the site and its deploys keep, or else `data.db` where the site runs.

Start the development server, which rebuilds the application when you edit its source:

```sh
topcoat dev
```

Then open <http://127.0.0.1:3000>.

## Releasing

Pushing a tag such as `v0.1.0` that matches the version in `Cargo.toml` builds the blog on
Ubuntu 22.04 and publishes a release with `vitor_personal_blog-x86_64-linux.tar.gz` and its
`.sha256`. The archive has the binary, the `assets` folder it loads, and the `posts`,
`public` and `toasty` folders it reads from its working directory, so run it from the
folder it was extracted to:

```sh
./vitor_personal_blog toasty migration apply
PORT=3000 ./vitor_personal_blog
```

Without `DATABASE_URL` or `OXPLOY_STORAGE_DIR`, the database is `data.db` in the same folder.
Set `HOST` to listen on another address than `127.0.0.1`, and `SITE_URL` to publish somewhere
other than `https://vitor.ws`.

## Posts

Each post is a markdown file in `posts/`, and its file name is its URL: `posts/my_post.md` is
`/blog/my_post`. The YAML at the top describes it:

```yaml
---
title: "My post"
subtitle: "Shown under the title"
description: "One or two sentences for search results and link previews"
tags: ["rust", "topcoat"]
date: "2026-11-02t21:00:00"
updated: "2026-11-04t20:50:00"
ready: true
---
```

A post shows only once `ready: true`. Dates are UTC, with or without the time. Set `updated`
when a post changes in a way worth telling readers: the post then shows the date, and search
engines and the sitemap get it. Keep `description` under about 160 characters, since search
results cut longer ones; without it, the subtitle stands in.

## Search engines and sharing

Every page has its canonical URL, a description, and the Open Graph and X tags that shape a
shared link's preview, with `assets/og.jpg` as the picture. To change the picture, edit
`scripts/social_image.swift` and run it on macOS:

```sh
swift scripts/social_image.swift assets/og.jpg
```

The pages also describe themselves to search engines as JSON-LD (`src/app/seo.rs`): the home
page as the site and its author, `/blog` as the blog, and each post as an article, with its
dates, tags and breadcrumbs. `/sitemap.xml` lists the pages with the date each last changed,
`/robots.txt` points to it, and `/feed.xml` is an RSS feed of the posts. Each post's headings
get an `id` from their text, so links can point at a section.

## Visits and the admin

The site counts the visits to its pages on its own server, with no analytics service, and
without keeping anyone's address. A visitor is a hash of their address and browser, made
with a secret key the site creates on its first start and the day, so the same person
counts once a day and cannot be followed from one day to the next. Each visit also keeps
the page, the time, the host of the site the visitor came from or the Android app that
opened the link, the kind of device, the browser's name and the link's `utm_source`,
`utm_medium`, `utm_campaign` and `utm_content`. Programs, such as search engines, are left
out, and so are prefetches and the browser runtime's own requests. Only the home page, the blog and the
published posts count, as `counts_as_visit` in `src/app.rs` lists them, so a new page goes
there too.

A banner asks each visitor whether the site may keep a cookie. Those who decline are still
counted, with only a `cookies=no` cookie that remembers their answer. Those who accept get a
`visitor` cookie with a random ID for 13 months, which tells when they come back on another
day; the site stores a hash of it, never the ID. The answer goes without reloading the page,
and a plain form sends it when JavaScript is off. `/privacy` explains all of it, the `theme`
cookie too, and lets visitors change their answer.

`/admin` shows the visits of today, yesterday, the last 7, 30 or 90 days, or any days from
one date to another within the 400 days kept: visitors, page views, each day's, each hour's
(a single day is charted by hour, a longer period by day and by hour of the day summed over
its days), the pages most opened, the sources, the campaigns, each visitor's day, their
devices and browsers, and, among those who accepted the cookie, how many came back.
Sources read as people call them, such as X (Twitter) for `t.co` or Hacker News for
`news.ycombinator.com`, and fall back to the link's `utm_source` when the browser says
nothing. Campaigns list the `utm_source`, `utm_medium`, `utm_campaign` and `utm_content` of
the first tagged link each visitor followed. The latest 100 visits are listed too: when each
visitor came, for how long, from where, on what, and the pages in order. Its name and
password are `ADMIN_USERNAME` and `ADMIN_PASSWORD`; without both, nobody can sign in. Once
signed in, the header links to it on every page. Days and times are in Brasília time,
`America/Sao_Paulo`, whatever the server's time zone.

The visits are in the `page_views` table of the site's database, next to the posts' view
counts, and are deleted after 400 days, when the site starts.

## MCP

`/admin/mcp` gives the visits to AI agents, such as the marketing ones, through the
[Model Context Protocol](https://modelcontextprotocol.io), so they can tell how the blog
does and which posts and shares bring readers:

| Tool | What it does |
| --- | --- |
| `site_visits` | The visits of the last 1 to 400 days, or from one day to another, as `/admin` shows them: each day and hour, sources, campaigns by their four tags and the latest 50 visits, with the totals of the period just before. |
| `post_performance` | Every published post, the most read first: its views, visitors, the visitors it brought to the site, their sources and campaigns, and the count the post shows. |
| `read_post` | A post's text in Markdown, with its title, description, tags and dates. |

It takes the token in `OXPLOY_MCP_TOKEN`, sent as `Authorization: Bearer <token>`, and a
missing or wrong one gets a 401 after a second; without the variable, nobody gets in.
oxploy reaches it: turn MCP on in the site's **MCP** tab at the path `/admin/mcp`, not the
default `/mcp`, since Caddy then hides the path from the site's domains, and oxploy gives
the site the token and passes its tools on to the organization's tokens.

A link shared with UTM tags, such as
`https://vitor.ws/blog/<slug>?utm_source=linkedin&utm_medium=post&utm_campaign=oxploy-launch`,
shows in `/admin` and the tools as a campaign, which tells which share brought readers.

The server answers each JSON-RPC message, sent by POST, with plain JSON, with no session
and no event stream, in `src/app/admin/mcp.rs`. Each tool is a file of its own in
`src/app/admin/mcp/`, with a line in `definitions` and in `call_tool`.

## Project structure

| Path | Contents |
| --- | --- |
| `src/app.rs`, `src/app/` | The route tree: the router, layouts, pages, and routes. |
| `src/features.rs`, `src/features/` | Everything one feature owns, such as its data and business rules. |
| `src/components.rs`, `src/components/` | Shared UI components that belong to no single feature. |
| `src/app/privacy.rs` | The privacy page, and the cookie banner. |
| `src/app/admin.rs` | The admin's page: signing in, and the visits. |
| `src/app/admin/mcp.rs`, `src/app/admin/mcp/` | The MCP server, at `/admin/mcp`, and its tools. |
| `src/features/analytics.rs`, `src/features/admin.rs`, `src/features/mcp.rs` | The visits, the admin's sign-in, and the MCP token. |
| `src/app/seo.rs` | The structured data for search engines, and the RSS feed. |
| `scripts/social_image.swift` | Draws `assets/og.jpg`, the picture of shared links. |
| `src/db.rs` | The database connection. |
| `toasty/` | The database migrations. Commit them with your code. |

Pages in `app` use features and components. A feature can use shared components but never imports from `app`, and shared components never import from a feature.

## Learn more

- [Getting started](https://github.com/tokio-rs/topcoat/blob/main/docs/getting_started.md)
- [Guides](https://github.com/tokio-rs/topcoat/tree/main#learn-topcoat)
