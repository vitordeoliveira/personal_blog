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

## What Oxploy is

The idea is simple: register a repository, build it, run the binary, and see
everything in a UI.

The whole panel is **one Rust binary, one process**:

```
          ┌──────────── oxploy (1 binary) ────────────┐
browser ─►│ Topcoat UI ──► deploy engine ──► SQLite   │
          └──────────────────┬────────────────────────┘
                             ├─► Docker API ─► your apps
                             └─► Caddy ─► your domains
```

- **[Topcoat](https://github.com/tokio-rs/topcoat)** for the UI. It renders on
  the server and only sends a small runtime to the browser, so there is no
  separate frontend and no internal HTTP API. A button calls the deploy code as
  a normal Rust function.
- **SQLite** for the panel's own data. No database server just to remember my
  apps.
- **Docker** to isolate each app and limit its CPU and memory, controlled
  through its API with [bollard](https://crates.io/crates/bollard) instead of
  calling the CLI.
- **[Caddy](https://caddyserver.com)** in front of the apps, mapping your domains
  to them, with HTTPS certificates it gets and renews on its own.

## Why only Rust apps?

Because I'm a sick Rust maximalist. I admit it.

But there is a practical reason too. When you only deploy one kind of app, you
can do it well instead of doing everything halfway. For a Rust app that means:

- **Build once, with `--release`.** One `cargo build --release` per deploy, in a
  container with the cargo registry and `target/` cached between builds. Then
  only the binary runs, in a minimal container. It starts instantly, never
  recompiles on restart, and the previous binary can stay around for a rollback.
- **No runtime to install.** A Rust app is one binary. No Node version, no
  Python virtualenv, no "works on my machine" between the build and the run.

Other languages may come later. At the start, no.

## Experimental, on purpose

Oxploy will not have everything Dokploy has. It's not trying to.

The first version is about the basics:

1. Register an app: repository, branch, port and environment variables.
2. Deploy: clone, build with cache, run.
3. Live logs of the build and of the app.
4. Status, and start, stop and restart.
5. PostgreSQL and Redis as containers, with volumes, linked to the app.

Auto-deploy from a GitHub webhook, deploys without downtime and multiple
servers come later, if they come.

As I write this, the panel already has sign-in, organizations, projects and
services, and live monitoring of the server's CPU, memory, disk and network.
The deploy pipeline is what I'm building now. If all goes well, it's already
live by the time you read this.

It solves **my** problem. If it solves yours too, even better.

## The risks I already know

Being honest about the bet:

- **Build time.** Rust compiles slowly, and that's problem number one for a
  deploy tool. Caching has to be right from day one, and
  [cargo-chef](https://github.com/LukeMathWalker/cargo-chef) is on the table to
  cache dependencies.
- **Security.** The panel controls Docker, and access to the Docker socket is
  basically root on the machine. So the sign-in page was one of the first
  things I built, not a "later".
- **Topcoat is young.** It's at version 0.11 and its browser runtime is still
  experimental. I'm betting on it, like this blog does.

## Why open source

Because this is the kind of tool you should be able to read before giving it
root on your server. And because if other people with small servers and Rust
apps find it useful, they can make it better than I would alone.

Next posts will follow Oxploy as it grows: what worked, what didn't, and why I
chose each piece.
