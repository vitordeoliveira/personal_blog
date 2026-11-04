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

## Visits and the admin

The site counts the visits to its pages on its own server, with no analytics service, and
without keeping anyone's address. A visitor is a hash of their address and browser, made
with a secret key the site creates on its first start and the day, so the same person
counts once a day and cannot be followed from one day to the next. Each visit also keeps
the page, the time, the host of the site the visitor came from, the kind of device, the
browser's name and the link's `utm_source`. Programs, such as search engines, are left
out, and so are the browser runtime's own requests. Only the home page, the blog and the
published posts count, as `counts_as_visit` in `src/app.rs` lists them, so a new page goes
there too.

A banner asks each visitor whether the site may keep a cookie. Those who decline are still
counted, with only a `cookies=no` cookie that remembers their answer. Those who accept get a
`visitor` cookie with a random ID for 13 months, which tells when they come back on another
day; the site stores a hash of it, never the ID. The answer goes without reloading the page,
and a plain form sends it when JavaScript is off. `/privacy` explains all of it, the `theme`
cookie too, and lets visitors change their answer.

`/admin` shows the visits of the last 7, 30 or 90 days: visitors, page views, each day's, the
pages most opened, where visitors came from, their devices, browsers and campaigns, and,
among those who accepted the cookie, how many came back. Its name and password are
`ADMIN_USERNAME` and `ADMIN_PASSWORD`; without both, nobody can sign in. Days are in the
server's time zone, which `TZ` changes, such as `TZ=America/Sao_Paulo`.

The visits are in the `page_views` table of the site's database, next to the posts' view
counts, and are deleted after 400 days, when the site starts.

## Project structure

| Path | Contents |
| --- | --- |
| `src/app.rs`, `src/app/` | The route tree: the router, layouts, pages, and routes. |
| `src/features.rs`, `src/features/` | Everything one feature owns, such as its data and business rules. |
| `src/components.rs`, `src/components/` | Shared UI components that belong to no single feature. |
| `src/app/privacy.rs` | The privacy page, and the cookie banner. |
| `src/app/admin.rs` | The admin's page: signing in, and the visits. |
| `src/features/analytics.rs`, `src/features/admin.rs` | The visits, and the admin's sign-in. |
| `src/db.rs` | The database connection. |
| `toasty/` | The database migrations. Commit them with your code. |

Pages in `app` use features and components. A feature can use shared components but never imports from `app`, and shared components never import from a feature.

## Learn more

- [Getting started](https://github.com/tokio-rs/topcoat/blob/main/docs/getting_started.md)
- [Guides](https://github.com/tokio-rs/topcoat/tree/main#learn-topcoat)
