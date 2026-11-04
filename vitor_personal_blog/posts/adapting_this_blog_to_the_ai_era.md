---
title: "Adapting this blog to the AI era"
subtitle: "Why I'm done writing tutorials, and what this blog is for now"
description: "This blog used to teach how to build things step by step. An AI agent
now writes that code faster than I can explain it, so from now on I'll write
about the projects I want to build and why I pick technologies like Rust and
Topcoat."
tags: ["ai", "opinion", "rust", "topcoat"]
date: "2026-11-02t19:00:00"
ready: true
---

Look at the posts on this blog: [How use axum + askama + htmx?](/blog/how_use_axum_askama_htmx),
[How host a rustlang server for free in the google cloud plataform?](/blog/how-host-a-rust-server-in-gcp)...
Almost all of them answer the same question: **how** do I build this thing?

In 2024 that made sense. You found a stack you liked, you fought it for a
weekend, and you wrote down the steps so the next person (usually future you)
didn't have to fight it again.

Today an AI agent writes that code faster than I can write the post explaining
it. And it doesn't need my post to do it.

## The how stopped being the hard part

A real example. This blog was a Next.js app: React, server actions, and
better-sqlite3 for the view counter. I moved it to
[Topcoat](https://github.com/tokio-rs/topcoat), a full-stack Rust framework,
with an AI agent doing most of the typing.

The agent read Topcoat's docs straight from the crate, ported the pages, did the
dark mode the way the docs recommend, moved the view counter to a new database
table, and added the SEO tags the old site never had.

So what did I do? I made decisions:

- Use Topcoat and Rust, not another JS framework.
- Keep only the view count from the old database. Everything else in there was
  either never read or only existed for the AI chat, and the chat can wait.
- Take only the orange from the old design, and let the framework's design
  system do the rest.
- Count a view only when a real browser opens the post, not every crawler that
  passes by.

None of these is a "how". All of them are "what" and "why". That's the part the
agent can't do for me, because it depends on what I want, what I'm willing to
maintain, and what I'm betting on.

A tutorial here would explain the part that is now cheap, and skip the part
that still matters.

## So what is this blog for now?

From now on, this blog is about:

1. **The projects I want to build.** What problem, why it's worth my time, and
   where I think the hard part is.
2. **Why I choose the technologies I choose.** The trade-offs, what I gave up,
   and what I'm betting on.
3. **Opinions about code.** How I want things structured, and what I accept or
   reject from an agent. There will still be code here, but to make a point, not
   to be copied line by line.

What you won't find anymore is "step 1: `cargo new`". Your agent knows that one.

> disclaimer: the old tutorials stay up. If they still help someone, great, but
> I'm not writing new ones.

## Why Rust and Topcoat, in the AI era

Since this is the first post on the new stack, let's start with a "why".

### The compiler is my first reviewer

When an agent writes most of the code, my job moves from writing to reviewing.
And reviewing is the bottleneck: I can't read everything with the same
attention.

Rust helps here more than any language I've used. The type system and the
borrow checker reject a whole category of mistakes before I even open the diff.
The agent can't hand me a null it forgot to check, a type it guessed wrong, or a
data race, because that code doesn't compile. In JavaScript all of that runs
just fine, until it doesn't.

Back in 2024 I wrote that with Rust
[you have a mentor on your side](/blog/why_js_devs_are_migrating_from_js_to_rust).
Now that mentor reviews the agent too.

### One language, one binary

Topcoat renders the pages on the server, writes the browser interactions as
Rust expressions that it compiles to JavaScript, and talks to the database
through Toasty, a Rust ORM. Pages, interactive bits and database code are all
Rust, checked by the same compiler, shipped as one binary.

Compare with what I had. Most of the last commits on the old blog were me
fighting the deploy to rebuild better-sqlite3, a native Node module, just so the
view counter could open a SQLite file. Not hard, just wasted time. I don't want
that kind of problem anymore.

### Docs for the exact version

This one surprised me. Topcoat ships its guides inside the crate. When the agent
needs to know how dark mode or cookies work, it reads the docs for the exact
version in my `Cargo.lock`, on my machine. No outdated blog post, no guessing
from an older version.

Which is one more reason not to write tutorials: the best ones now live next to
the code, versioned with it.

### The honest part

Topcoat is young. This blog runs on version 0.11, and its own docs call the
browser runtime "highly experimental". I'm betting on it anyway, because I like
where it's going: server first, Rust everywhere, and code that is easy to read
for both humans and agents.

If the bet goes wrong, that will be a good post too.

## What's next

The next posts will be about the projects I want to build: why each one exists,
and why I chose to build it the way I did.

The how, my agent handles. The why is still mine.
