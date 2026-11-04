---
title: "How use axum + askama + htmx?"
subtitle: "A quick explanation about how implement, Askama and HTMX"
description: "In a post before I explain why and what is Askama, Axum and HTMX, now we will implement then."
tags: ["rust", "axum", "htmx", "askama"]
date: "2024-04-03t20:35:00"
ready: true
---

Ok, I already have a post explaining [What is and Why to use Askama, HTMX, and Axum](/blog/what_are_axum_askama_htmx_and_why_use_it).

Now here is the place where the children cry and the mom doesn't see (Brazilian expression)

[Github final code](https://github.com/vitordeoliveira/blog_how_use_axum_askama_htmx/tree/main)

let's start from the beginning

Init the new project

```bash
cargo init axum_htmx_askama
cd axum_htmx_askama
```

Add the dependencies (run in the terminal)

```bash
# basic stuff
cargo add tokio -F full
cargo add askama
cargo add axum

# extra for tracing
cargo add tracing
cargo add tracing-subscriber
```

> I will in another post explain [how to properly set up a project](/blog/how_to_properly_setup_a_rust_project) here I will just show the basics of these three tools.

Let's start running the server, go to /axum_htmx_askama/src/main.rs and add the following code:

```rust

 use axum::{routing::get, Router};

 async fn root() -> &'static str {
     "Hello from axum"
 }

 #[tokio::main]
 async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt().init();
    let port = "8080";

    tracing::info!("router initialized, now listening on port {}", port);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    let app = Router::new().route("/", get(root));

    axum::serve(listener, app).await.unwrap();

    Ok(())
 }

```

now if you run cargo run and go to localhost:8080 you should receive a message from Axum.

Ok, pretty cool... but now let's add Askama also in the game

To do that, we need first to create a template folder in the root (NOT IN THE SRC/)

> /axum_htmx_askama/templates/

now create a file called root.html in the templates

> /axum_htmx_askama/templates/root.html

in root.html we will create the base template of our frontend.

> You can think like root.html from React

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  </head>
  <body>
    <div id="content">
      {% block content %}
      <p>Placeholder content</p>
      {% endblock %}
    </div>
  </body>
</html>
```

You can see a block called content inside the body, this block is how Askama will know which part of the HTML he should change when we use this base on other pages.

> As we are here, let also add tailwind as CDN (not recommended on production [see tailwind docs](https://tailwindcss.com/docs/installation))

> [How properly add tailwind CSS to a rust production code](/blog/how_properly_add_tailwind_css_to_a_rust_production_code)

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <script src="https://cdn.tailwindcss.com"></script>
  </head>
  <body>
    <div id="content">
      {% block content %}
      <p>Placeholder content</p>
      {% endblock %}
    </div>
  </body>
</html>
```

let's also add a title block, then every page that uses this base should pass a title (good for SEO)

```html
<!doctype html>
<html lang="en">
  <head>
    <title>{% block title %}{{ title }}{% endblock %}</title>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <script src="https://cdn.tailwindcss.com"></script>
  </head>
  <body>
    <div id="content">
      {% block content %}
      <p>Placeholder content</p>
      {% endblock %}
    </div>
  </body>
</html>
```

Now last but not least let's add HTMX to the base

```html
<!doctype html>
<html lang="en">
  <head>
    <title>{% block title %}{{ title }}{% endblock %}</title>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <script src="https://cdn.tailwindcss.com"></script>

    <script
      src="https://unpkg.com/htmx.org@1.9.10"
      integrity="sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC"
      crossorigin="anonymous"
    ></script>
  </head>
  <body>
    <div id="content">
      {% block content %}
      <p>Placeholder content</p>
      {% endblock %}
    </div>
  </body>
</html>
```

## Wait.... is HTMX ONLY that?? A script tag that loads a JS??

Quick answer... YES. Simple, elegant, and powerful.

> The HTMX has a size of approximately 10 kb, just as a comparison, React it's 316kb for base React + 4.5mb for React DOM + ALL the other libraries.

> HTMX is at least 481.6x smaller than React, and yes that makes a difference for the client.

Again if you want to understand why is so powerful, see the blog post of [what and why use HTMX, Askama, and Axum](/blog/what_are_axum_askama_htmx_and_why_use_it).

### OK, now we have the base HTML setup with tailwind and HTMX, what is next?

Now we "import" this HTML to inside Rust with Askama.

Go back to main.rs

look how simple it is

```rust
 use askama::Template;
 use axum::{
     http::StatusCode,
     response::{Html, IntoResponse},
     routing::get,
     Router,
 };

 #[derive(Template)]
 #[template(path = "root.html")]
 struct RootTemplate {
     title: &'static str,
 }

 async fn root() -> impl IntoResponse {
     let root = RootTemplate { title: "title" };
     (StatusCode::OK, Html(root.render().unwrap()))
 }
```

And as Rust makes the checks at compile time if you remove the

```html
<title>{% block title %}{{ title }}{% endblock %}</title>
```

from the root.html

Rust will give you a hint that the title was never used :D (oh man, I love Rust)

![warn message title](/blog_assets/warn_message_title_exemple.png)

You can make this check in your environment.

That is SOOOOO good because a lot of times Rust will give you a compile-time error **IF YOUR HTML IS WRONG**. Isn't that the most beautiful thing??

Ok, add the block title again (if you had removed it), and let's continue.

if you re-run your server you will see the "placeholder content".

let's pass now string from Rust to HTML (like we did with the title)

Replace the

```html
<p>Placeholder content</p>
```

To

```html
<p>{{ myownvalue }}</p>
```

If you re-run your server you will receive a BEAUTIFUL compile error, because your html has a variable that had not been passed.

If you go back to main.rs you will see this

![myownvalue error message exemple](/blog_assets/myownvalue_error_message_exemple.png)

Again, this is for me (and a lot of devs) a moment developer orgasmic moment and one of the reasons why Rust is so beloved by so many people.

The compiler is saying to you "Hey man, you have done shit, I will not allow you to ask for a variable that is not passed in your HTML".

Continue... let's add this new field in our struct

```rust
 #[derive(Template)]
 #[template(path = "root.html")]
 struct RootTemplate {
     title: &'static str,
     myownvalue: &'static str,
 }

 async fn root() -> impl IntoResponse {
     let root = RootTemplate {
         title: "title",
         myownvalue: "Hello from myownvalue",
     };
     (StatusCode::OK, Html(root.render().unwrap()))
 }
```

Re-run your server and see the result :D

![hello from myownvalue](/blog_assets/hello_from_myownvalue.png)
Let's create another file that inherits this base to be your home page

So create a file called home.html inside the templates

> /axum_htmx_askama/templates/home.html

on the file paste the following code

```html
{% extends "root.html" %} {% block content %}

<div>
  <h1>Hello from home page</h1>
  <p>here we will pass a value: {{stringvalue}}</p>
  <p>here we will pass a vector of strings and render then:</p>
  <ul>
    {% for value in vec_strings %}
    <li>{{ value }}</li>
    {% endfor %}
  </ul>
</div>

{% endblock %}
```

we will pass to home a string value and a vector/list of values.

This example is useful to show a little bit of the power of Askama

> The goal of this post is not to teach the technologies, just to teach HOW setup then in Rust, but is nice to give a little glance. I will make more in-depth posts about them individually.

> [What Askama can do?](/blog/what_askama_can_do)

Now in the main.rs, we need to create a new struct HomeTemplate

```rust
 #[derive(Template)]
 #[template(path = "home.html")]
 struct HomeTemplate {
     title: &'static str,
     stringvalue: &'static str,
     vec_strings: Vec<&'static str>,
 }

 async fn home() -> impl IntoResponse {
     let home = HomeTemplate {
         title: "title",
         stringvalue: "Hello from myownvalue",
         vec_strings: vec!["Rust", "is", "the", "best", "language"],
     };
     (StatusCode::OK, Html(home.render().unwrap()))
 }
```

Now let's add a new route to Axum, to do this you just need to append a new route in your Router. Axum works in a layer structure.

> The goal of this post is not to teach the technologies, just to teach HOW setup then in Rust, but is nice to give a little glance. I will make more in-depth posts about them individually.

> [How Axum works, and what it can do?](/blog/how_axum_works_and_what_it_can_do)

```rust
 let app = Router::new()
        .route("/", get(root))
        .route("/home", get(home));
```

If you access **_localhost:8080/home_** you will see this page

![hello from home](/blog_assets/hello_from_home.png)
Let's add some styling on this page with tailwind:

```html
{% extends "root.html" %} {% block content %}

<div
  class="container bg-gray-100 m-auto text-center text-xl mt-16 p-16 border-orange-700 border-4 rounded-lg"
>
  <h1 class="text-3xl mb-8">Hello from home page</h1>
  <p class="my-5">
    here we will pass a value:
    <span class="text-orange-700">{{stringvalue}}</span>
  </p>
  <p class="my-5">here we will pass a vector of strings and render then:</p>
  <ul class="text-orange-700 font-bold">
    {% for value in vec_strings %}
    <li>{{ value }}</li>
    {% endfor %}
  </ul>
</div>

{% endblock %}
```

> Don't be strict to tailwind here, definitely, the goal is not teaching him.
> If you want more information about [tailwind see their official docs](https://tailwindcss.com/docs/installation) or [How properly add tailwind CSS to a rust production code](/blog/how_properly_add_tailwind_css_to_a_rust_production_code).

let's also add a little style to the body, so go to root.html and add

```html
<body class="bg-gray-300">
  ... rest
</body>
```

good your page should look like this

![hello from home styled](/blog_assets/hello_from_home_styled.png)

Beautiful isn't it?? I was a professional designer for 10 years (contains irony)

Now the last thing we will do is add a little bit of HTMX to you guys understand the idea.

> The goal of this post is not to teach the technologies, just to teach HOW setup then in Rust, but is nice to give a little glance. I will make more in-depth posts about them individually.

> [What HTMX can do?](/blog/what_htmx_can_do)

In the root page, let's add a button that will request to append this home div to the root page.

Let's create a button, that will trigger on click, make a request to /home, take the response and target in the element that have id=content, and afterend append the response.

```html
<button
  hx-get="/home"
  hx-trigger="click"
  hx-target="#content"
  hx-swap="afterend"
>
  Append home after end of the element with id = content
</button>
```

I will add some styling with the tailwind, to looks a little better.

the body should look like this

```html
<body class="bg-gray-300">
  <div id="content" class="container m-auto text-center my-12 text-3xl">
    {% block content %}
    <p>{{ myownvalue }}</p>
    <button
      class="bg-orange-700 text-white border-orange-700 border-4 p-3 mt-5 rounded-lg"
      hx-get="/home"
      hx-trigger="click"
      hx-target="#content"
      hx-swap="afterend"
    >
      Append home after end of the element with id = content
    </button>
    {% endblock %}
  </div>
</body>
```

On main.rs change the myownvalue root function

```rust
async fn root() -> impl IntoResponse {
    let root = RootTemplate {
        title: "title",
        myownvalue: "Hello from AXUM+ASKAMA+HTMX",
    };
    (StatusCode::OK, Html(root.render().unwrap()))
}
```

The page should look like this

![hello from root styled](/blog_assets/hello_from_root_styled.png)

If you press the button, you should append the response of /home, then will look like this

![hello from root styled htmx hello_from_root_styled_htmx_event](/blog_assets/hello_from_root_styled_htmx_event.png)

**_PERFECT!!!!_**

Now you guys already know, how to use AXUM + HTMX + Askama

From this post, you guys should be able to set up your projects with these technologies.

So all final code should look like this

**axum_htmx_askama/src/main.rs**

```rust

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

#[derive(Template)]
#[template(path = "root.html")]
struct RootTemplate {
    title: &'static str,
    myownvalue: &'static str,
}

async fn root() -> impl IntoResponse {
    let root = RootTemplate {
        title: "title",
        myownvalue: "Hello from AXUM+ASKAMA+HTMX",
    };
    (StatusCode::OK, Html(root.render().unwrap()))
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    title: &'static str,
    stringvalue: &'static str,
    vec_strings: Vec<&'static str>,
}

async fn home() -> impl IntoResponse {
    let home = HomeTemplate {
        title: "title",
        stringvalue: "Hello from myownvalue",
        vec_strings: vec!["Rust", "is", "the", "best", "language"],
    };
    (StatusCode::OK, Html(home.render().unwrap()))
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt().init();
    let port = "8080";

    tracing::info!("router initialized, now listening on port {}", port);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(root))
        .route("/home", get(home));

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
```

**axum_htmx_askama/templates/root.html**

```html
<!doctype html>
<html lang="en">
  <head>
    <title>{% block title %}{{ title }}{% endblock %}</title>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <script src="https://cdn.tailwindcss.com"></script>

    <script
      src="https://unpkg.com/htmx.org@1.9.10"
      integrity="sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC"
      crossorigin="anonymous"
    ></script>
  </head>
  <body class="bg-gray-300">
    <div id="content" class="container m-auto text-center my-12 text-3xl">
      {% block content %}
      <p>{{ myownvalue }}</p>
      <button
        class="bg-orange-700 text-white border-orange-700 border-4 p-3 mt-5 rounded-lg"
        hx-get="/home"
        hx-trigger="click"
        hx-target="#content"
        hx-swap="afterend"
      >
        Append home after
      </button>
      {% endblock %}
    </div>
  </body>
</html>
```

**axum_htmx_askama/templates/home.html**

```html
{% extends "root.html" %} {% block content %}

<div
  class="container bg-gray-100 m-auto text-center text-xl mt-16 p-16 border-orange-700 border-4 rounded-lg"
>
  <h1 class="text-3xl mb-8">Hello from home page</h1>
  <p class="my-5">
    here we will pass a value:
    <span class="text-orange-700">{{stringvalue}}</span>
  </p>
  <p class="my-5">here we will pass a vector of strings and render then:</p>
  <ul class="text-orange-700 font-bold">
    {% for value in vec_strings %}
    <li>{{ value }}</li>
    {% endfor %}
  </ul>
</div>

{% endblock %}
```

[Github final code](https://github.com/vitordeoliveira/blog_how_use_axum_askama_htmx/tree/main)

    **It is easy, it is peasy, it is lemon and squeezy... We create a literal full-stack application with 3 files, something near 60 lines of Rust and some HTML.**
