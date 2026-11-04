---
title: "What are Axum + Askama + HTMX and Why use it?"
subtitle: "A quick explanation about Axum, Askama, and HTMX"
description: "I am a fan of some tools and technologies, I think all my blog posts make this clear to everyone.
Today, I will show you guys what and why to use those technologies on Rust.
"
tags: ["rust", "axum", "htmx", "askama", "opinion"]
date: "2024-04-01t17:52:00"
ready: true
---

I am a fan of some tools and technologies, I think all my blog posts make this clear to everyone.

Today, I will show you guys [what is HTMX](#what-is-htmx) and [why use these tools](#why-use-these-tools) to use **AXUM + HTMX + Askama** technologies on Rust.

> If you already know what Axum, Askama, and HTMX are and why you should use them, just [click here how use axum + askama + htmx](/blog/how_use_axum_askama_htmx), and we jump to the HOW blog post.

OK, now lets go to **What** is these tools :D 

> They say "Start with the why", but I don't see the reason to explain the why if you do not even know what the tools are haha

So let's go to the definitions:
- [What is HTMX?](#what-is-htmx)
- [What is Askama?](#what-is-askama)
- [What is Axum?](#what-is-axum)

## <a name="what-is-htmx"></a> What is HTMX?
HTMX is a nutshell HTML as language :D

It is a JS script that gives super powers to HTML, different from React and others which is a Javascript that generates HTML.

The idea is that HTML can from EVERY tag dispatch an event to the backend, and deal with the response (that should also be HTML).

In raw HTML, only a few tags can dispatch an event to the backend, the \<a\> tag, and the form with submit.

HTMX allows every element to do that...

This is very simple, elegant, and powerful because this way you just use JavaScript where is needed and not in everything how is done in the modern frameworks.

that also allows the backend devs to use the language of choice in the frontend, making you run away from JS or TS.

## <a name="what-is-askama"></a> What is Askama
Askama is simply the rust crate that we will use to make Rust read and manipulate HTML.

You might think "I can do that myself, I don't need a tool just to deliver HTML to the client".

Yes, and that is how your website receives a Cross-site scripting (XSS) attack...

I have in my mind that at least 95% of the world's problems happen because of overconfidence.

Admitting ignorance is good because this way we can learn and improve.

Some good devs (probably better than you) already deal with the security stuff, do not reinvent the wheel.

## <a name="what-is-axum"></a> What is Axum?
Axum is a Rust crate that allows us to serve a web application framework.

So in a nutshell he is the one we use to create the server, routes, etc...

## <a name="why-use-these-tools"></a> Why use these tools?

### HTMX is the star of the show

HTMX to simplify the frontend...

We DONT NEED a heavy and very confusing SPA to write a simple blog... to be honest more I code on those tools more I think we do not even need SPAs for nothing...

> The HTMX has a size of approximately 10 kb, just as comparison, React it's 316kb for base React + 4.5mb for React DOM + ALL the other libraries.

> HTMX is at least 481.6x smaller than React, and yes that makes difference for the client.


Ok, lets put this in different words, the SPAs was created to have more dynamic pages and bring a better user experience, they have their value. But in a world with blazing-fast internet, and awesome infrastructure tools, MAYBE, and just maybe, we should have the backend deal with the state of the application, and rely on the Frontend to be just a view and not a copy of the state of the backend.

What do I mean by that? With SPAs the frontend is a JS bundle that is downloaded in the user browser, and generates the HTML in the client side, after this the client requests the backend data to be loaded, so the backend sends to the client a JSON, which is on this point in time a copy of the state of the backend, then the client takes this JSON and react (Now you understand why the ReactJs has this name :D ) to it, updating the client on runtime. 

Here is a very simplified draw of this explanation: 

![spa exemple](/blog_assets/spa_draw_exemple.png)

And here is a very simplified draw of what we want to achieve with HTMX

![htmx exemple](/blog_assets/htmx_exemple.png)

As you can see on this second draw we just have ONE state of the application, this way the view/frontend doesn't need to store information, and the client shouldn't know anything about the logic of the application.

Another advantage of HTMX is the fact, that now you can create the entire application using whatever language you like, here we are using Rust, but you can do with, GO, Java, Rails, and even NodeJS... as the frontend is not forced to be only JS anymore, you can use the backend of your preference.

Ok, but that is exactly how it was in the past, so we are just backing to the old days right?

Kinda... the difference here is that with HTMX you don't need to render the entire page, the backend will just send an HTML fragment, and the HTMX on the client side will take care of where this new HTML should be placed... This changes some stuff, in the past when we navigated to another page we had a "blink" of half of a second because the entire page had been updated, with HTMX we don't have this because we are just swapping or appending divs in the HTML of the client side.

If you want to understand better, how HTMX works, I strongly recommend watching and reading those links:

Official page:

<a href="https://htmx.org/" target="_blank">https://htmx.org/</a>

The PrimeGOage video on Frontend Masters


<a href="https://www.youtube.com/watch?v=SZ0nR3QHebM" target="_blank">
Youtube PrimeGOage Frontend Masters
</a>

### Askama
Because is easy to make shit, and they already apply all the security stuff the templating needs.

and also because it makes life very easy with HTML manipulation.

### Axum
Because is easy to use and very extendable... In a nutshell is that.

## How to implement them in RUST?

The real and fun part is a different blog post, just [click here how use axum + askama + htmx](/blog/how_use_axum_askama_htmx) to go there :D




