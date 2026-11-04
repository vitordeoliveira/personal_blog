---
title: "Oxploy: my own Dokploy, in Rust"
subtitle: "A lightweight, open source deploy panel for Rust apps on small servers"
description: "Dokploy is great, but it asks for more of my small server than I want to
give a deploy panel. So I'm building Oxploy: an experimental, open source
version in Rust, made with Topcoat and Caddy, focused on deploying Rust apps."
tags: ["oxploy", "rust", "topcoat", "self-hosting"]
date: "2026-11-02t21:00:00"
ready: true
---

> **Update:** since this post, Oxploy dropped Docker. Apps now run as plain
> processes behind Caddy, and SQLite is the only database it manages. The parts
> below are updated to match.

As I said in the [last post](/blog/adapting_this_blog_to_the_ai_era), this blog is
now about the projects I'm building and why. So here is the first one:
**Oxploy**, my own version of [Dokploy](https://dokploy.com), written in Rust.

The name is *oxide* + *deploy*. It lives at [oxploy.com](https://oxploy.com), and
it's open source.

## The problem: a small server

I like Dokploy. You connect a repository, it builds and runs your app, and you
get a nice panel with logs, domains and databases. Your own little Vercel, on
your own machine.

But my server is small. And Dokploy's own docs ask for at least 2GB of RAM and
30GB of disk, because the panel itself is a Next.js app, a PostgreSQL database
and Traefik, all running next to your apps.

On a big server, nobody cares. On a small one, the deploy panel is competing for
memory with the apps it's supposed to deploy. That's backwards: the panel should
be the smallest thing on the machine.

I believe Rust can do this much leaner. So I'm building it.

**The result:** memory use on my VPS dropped from 2.5GB to 0.5GB, and now all my
apps run on the same machine. Even oxploy.com runs on Oxploy.

## What Oxploy is

The idea is simple: register a repository, build it, run the binary, and see
everything in a UI.

The whole panel is **one Rust binary, one process**:

```
          ┌──────────── oxploy (1 binary) ────────────┐
browser ─►│ Topcoat UI ──► deploy engine ──► SQLite   │
          └──────────────────┬────────────────────────┘
                             ├─► processes ─► your apps (127.0.0.1:<port>)
                             └─► Caddy ─► your domains
```

- **[Topcoat](https://github.com/tokio-rs/topcoat)** for the UI. It renders on
  the server and only sends a small runtime to the browser, so there is no
  separate frontend and no internal HTTP API. A button calls the deploy code as
  a normal Rust function.
- **SQLite** for the panel's own data. No database server just to remember my
  apps.
- **Plain processes** for the apps. Each one runs on 127.0.0.1, in a process
  group of its own, at a port from 10000 to 19999 that Oxploy passes in `PORT`.
  A Rust binary needs nothing installed to run, so there is no container to pay
  for.
- **[Caddy](https://caddyserver.com)** in front of the apps, mapping your domains
  to them, with HTTPS certificates it gets and renews on its own. Every new app
  also gets an address of its own on sslip.io, with HTTPS, before you set a
  domain.

## Why only Rust apps?

Because I'm a sick Rust maximalist. I admit it.

But there is a practical reason too. When you only deploy one kind of app, you
can do it well instead of doing everything halfway. For a Rust app that means:

- **Build once, with `--release`.** One `cargo build --release` per deploy, on
  the server, with the Rust version, the binary and the features chosen per
  service, or a build script of your own. Each app keeps its own cargo target
  folder, so dependencies are not rebuilt on every deploy. Then only the binary
  runs. It starts instantly, never recompiles on restart, and the two releases
  before the live one are kept. You can also skip building on the server: build
  in CI, and Oxploy deploys the binary from a GitHub release.
- **No runtime to install.** A Rust app is one binary. No Node version, no
  Python virtualenv, no "works on my machine" between the build and the run.
  That's exactly why Oxploy doesn't need containers: there is nothing to pack
  around the binary.

Other languages may come later. At the start, no.

## Experimental, on purpose

Oxploy will not have everything Dokploy has. It's not trying to.

The first version is about the basics:

1. Register an app: repository, branch and environment variables. GitHub is
   connected through a private, read-only GitHub App you register from the
   panel, so private repositories work too.
2. Deploy: clone, build with cache, run. A script after the build, such as a
   migration, runs before the new release starts. If it fails, the running
   release stays.
3. Live logs of the build and of the app.
4. Status, and start, stop and restart. An app that crashes restarts on its
   own, waiting longer each time, up to a minute.
5. SQLite databases in the app's storage folder, which deploys keep, connected
   through a variable such as `DATABASE_URL`, with backups and restores in one
   click.

Auto-deploy is here now, from GitHub releases: CI builds the binary, and Oxploy
downloads it from the latest release. A switch deploys each new release on its
own, checking every minute, and Oxploy refuses a binary built for another
machine before it stops the running release. Push webhooks for branch builds,
deploys without downtime (today the old release stops before the new one
starts), multiple servers and CPU and memory limits per app come later, if they
come.

When I first wrote this, the panel already had sign-in, organizations, projects
and services, and live monitoring of the server's CPU, memory, disk and network,
and the deploy pipeline was what I was building. Now it's live: oxploy.com
itself runs on Oxploy, deployed from its GitHub releases. Apps keep running while
Oxploy restarts or updates itself, and when a new version is out, the panel says
"Update available" and updates in one click.

It solves **my** problem. If it solves yours too, even better.

## The risks I already know

Being honest about the bet:

- **Build time.** Rust compiles slowly, and that's problem number one for a
  deploy tool. Caching has to be right from day one. Each app keeps its own
  cargo target folder, so a deploy doesn't rebuild the dependencies.
  [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) was on the table,
  but it's for Docker layers, and without Docker the cargo cache does that job.
  And with releases, the server doesn't build at all: CI does, and Oxploy deploys
  the binary.
- **Security.** There is no Docker socket anymore. Oxploy runs as an
  unprivileged `oxploy` user under systemd, and the apps run as that user too.
  The first sign-up needs a setup code from Oxploy's log, so whoever opens the
  address first can't take the server. The honest trade-off: without containers,
  apps are not isolated. They share the machine and the `oxploy` user, with no
  CPU or memory limit per app. The monitoring shows what each app uses, so at
  least I see who is eating the server.
- **Topcoat is young.** It's at version 0.11 and its browser runtime is still
  experimental. I'm betting on it, like this blog does.

## Why open source

Because this is the kind of tool you should be able to read before giving it
your server. And because if other people with small servers and Rust apps find
it useful, they can make it better than I would alone.

Next posts will follow Oxploy as it grows: what worked, what didn't, and why I
chose each piece.
