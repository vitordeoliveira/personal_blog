---
title: "Why Js devs are migrating from Js to Rust?"
subtitle: "why I make the choice of migrating from JS after 8 years, to full Rust on my projects?"
description: "Increasingly, we've seen many Rust developers joining the community
coming from languages like JS, here we will try to explain why, using my own
experience of why I switched to rust"
tags: ["js", "rust", "opinion"]
date: "2024-03-16t12:58:00"
ready: true
---

> why I make the choice of migrating from JS after 8 years, to full Rust on my projects?

Increasingly, we've seen many Rust developers joining the community
coming from languages like JS, here we will try to explain why, using my own
experience of why I switched to rust

## Reason 1: To build "real software"

Lets start with this contradictory one, of course all software written in JS or
TS is real software, and software of millions of dollars was build using those
languages, probably more successful softwares that your entire career, and
better then the all code you will ever write.

The thing is, normally when we think in softwares that we admire, like linux,
windows (I don't agree with this one), NGINX, Bitcoin and etc... all those
softwares was written with "low" level languages, like C or C++, some even with
Go like Docker and Kubernetes. Never with JS or Python, and have a really good
reason for it.... SPEED BOYYYYY.

> disclaimer: again, of course that exist awesome tools builded with JS or
> Python, but that is not the point here

Yes, performance is for a lot of applications one of the core features, and if
you are a JS dev, that is not something you will achieve with maestry, you can
have speed, but is naturally not comparable, remembering that no language will
fix our shit code you write this morning.

The point is, with Rust you can create EVERYTHING that need extreme
performance, to be honest you can create everything with him, you not
necessarily should, but you can.

With Rust and JS in your toolbox, you have to very complementary tools for a
engineer, of course you need more tools to be a good engineer, but with those 2
at least you understand the basic of how the things works on a computer.

With just JS.... I am sorry to say, you probably need to learn languages near
from the machine to understand what you are doing. Being franc, majority of
devs not even know how their tools works... don't be like that... Be a engineer
not a tool operator.

## Reason 2: You have a mentor in your side

My change it start when I was reading the bitcoin code, who is in C++, and then
start to create some code around to play with it. The thing is.... I am not a
C++ dev, I dont have 30 years of experience to jump away from all the possible
pitfalls I was creating on that code, and I am aware of it, 99% of the
humanity problems is the fact people does not admit they are incompetent in
some fields, and I know I am incompetent in C and C++. (skill issues).

Of course, with perseverance and time you will naturally be an awesome c and
c++ dev, and I would do exactly that if Rust does not exist.... BUT thanks for
some crazy engineers, they create a language that is like coding with an expert
in your side, and THAT my friends, changes everything.

The way how our brain learns is in cycles of feedbacks, you do shit -> check
the shit -> smell -> taste it -> and learn that was a bad idea -> if you are
not stupid, does not taste again.

This is the process of learning...

If you have a good mentor, he will show you, and explain to you before WHY you
should not taste it, and that make the process WAY faster

Rust compiler does exactly that, is like having a C++ master saying to you,
"hey men, this is a bad idea, because of this, and this, can you change the 3
line of your code with this option here".

and that is a HUGE shortcut in the path of "low" level coding, you have kinda
of mentor in your side.

> Rust make me a better c and c++ dev

## Reason 3: Cargo and Crates.io

As Rust is a new language he has learn with the mistakes and successes of the
previous languages. One of this learnings is the package manager, Rust has his
own "NPM", it is called Cargo, and the packages repository is called Crates.io.

That makes way easier to import and share code, and also makes the coding more
familiar for those who came from JS world.

## Reason 4: To be a better and more complete dev

I already mention that in the previous topics, but I also think is important to
mention in a separate one. Majority of the JS devs are devs with less then 5
years of experience, that start coding in the JS.

Lets be francs every Js dev who REALLY loves coding, know that to learn the
real forbidden arts of computer program we need to learn **low level
languages**, that gives to the dev full power of the machine.

Rust gives you exactly what you need, a low level language with a modern syntax
and a "package manager"

I call that top -> bottom way of learning, where you start in a high level,
high abstraction topic, and gradually goes to low level, low abstraction.

This approach has 2 main problems:

1. The base of every knowledge is WAY bigger than the top, looks like a pyramid
2. In the JS world things change so quickly, with people reinventing the wheel
   all the time, it's very easy and common to get stuck in this “new learning”
   that is more of the same in a different way, BUT ARE YOU SURE, that it's a
   thing new.

If you accept to stop to re-invent the wheel, and deep-in in the computer
science, **Rust is the right language to you.**
