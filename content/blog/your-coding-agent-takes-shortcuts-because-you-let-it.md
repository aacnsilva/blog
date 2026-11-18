+++
date = '2026-11-18T12:00:00+00:00'
draft = false
title = 'Your coding agent takes shortcuts because you let it'
+++

You ask a coding agent for a feature and it comes back saying everything works. Then you read the code. The hard part is a comment that says "known limitation". A lint rule was switched off for one line. The difficult problem is still there, it is just documented now. You either fix it yourself or ask again, and the time you thought you saved is gone.

Most of the advice I see for this is about prompting better. That helps, but it was not what made the difference for me. What made the difference happened before I opened the agent: I decided what it was not allowed to do, and how it would prove its work. And the first rule surprises people: no comments in the code. None.

I tested this on a real project. For a long time I have wanted to learn to play the guitar and never made the time, so I built myself a course: [music](https://music.aacnsilva.com). It went from the first commit to 50 interactive lessons, for guitar and piano, in two days and 28 commits. This post is about the setup that made that possible, and about the one thing the setup could not do for me.

## Ban the comments

We are taught that comments are good, so this sounds wrong. But look at what an agent uses comments for. When it hits something difficult, a comment is the cheapest way out: explain the shortcoming, promise to handle it later, move on. The work looks finished and the problem is still there.

Take the comments away and that exit is closed. Our friends can no longer document their shortcomings, so they do the proper thing: refactor what needs to be refactored and fix the underlying issue. Not every time, but most of the time.

A rule in a prompt is not enough, though. An agent can forget a rule, it cannot forget a failing check. In music the ban is a lint rule that fails the build, it covers HTML comments too, and inline directives that disable the linter are switched off. If the code needs explaining, the only options left are better names and smaller functions.

## Treat the agent like a new teammate

Can you spot the similarity to how humans build together? When someone joins a team we give them the project conventions, the linting rules and the compiler settings, so that they are set up for success. The same applies to agents.

Music is a simple project, so I picked one language, TypeScript, with the strictest compiler settings and the practices the community recommends. Functions have to stay small: limited complexity, limited nesting, at most three parameters. One more rule matters as much as the others: never weaken a rule to make the code pass. Fix the code. If your project has more than one language, do the same steps for each one.

## "It runs" is not verification

How do we, as engineers, know our work is correct? Not by running it once and seeing that it works. It goes deeper, and with agents going deeper is fast and no longer expensive.

- The feature has to be correct.
- It has to be built without spaghetti code. Deep modules, as John Ousterhout advises in [A Philosophy of Software Design](https://web.stanford.edu/~ouster/cgi-bin/aposd2ndEdExtract.pdf), a book I highly recommend.
- It has to be fast enough and small enough. It is not enough that I can add 2+2 if it takes me ten minutes to do it.

So think about which tools you would need to check all of that, put them in one harness, and tell the agent to use it to verify its own work. In music that is a single command that runs formatting, lint, type checks, tests and the build, and CI runs the same steps on every pull request. Performance and memory checks are not in there yet. They are the next thing to add.

<figure class="theme-diagram">
  <img class="theme-diagram__light" src="/images/music-guardrails.svg" width="760" height="568" alt="Four layers of checks: rules written down in the repo, a pre-commit hook, npm run check running format, lint, types, tests and build, and CI running the same steps on every pull request.">
  <img class="theme-diagram__dark" src="/images/music-guardrails-dark.svg" width="760" height="568" alt="Four layers of checks: rules written down in the repo, a pre-commit hook, npm run check running format, lint, types, tests and build, and CI running the same steps on every pull request.">
</figure>

## Design before the agent builds

With the guardrails in place it is tempting to say "build me a guitar course" and watch. I don't. An agent without a design will guess, and every guess is something to undo later. So I design in the same order every time.

First the API. Once it is solid, the data model that supports it.

<figure class="theme-diagram">
  <img class="theme-diagram__light" src="/images/music-api-design.svg" width="760" height="556" alt="API design: lesson YAML in git is built by Astro into static HTML pages and a read-only JSON API under /api/v1 with instruments, a curriculum per instrument and one JSON file per lesson.">
  <img class="theme-diagram__dark" src="/images/music-api-design-dark.svg" width="760" height="556" alt="API design: lesson YAML in git is built by Astro into static HTML pages and a read-only JSON API under /api/v1 with instruments, a curriculum per instrument and one JSON file per lesson.">
</figure>

<figure class="theme-diagram">
  <img class="theme-diagram__light" src="/images/music-data-model.svg" width="760" height="462" alt="Data model: one curriculum file per instrument lists modules and lesson slugs in order, each lesson is one YAML file, and each lesson holds an ordered list of typed blocks.">
  <img class="theme-diagram__dark" src="/images/music-data-model-dark.svg" width="760" height="462" alt="Data model: one curriculum file per instrument lists modules and lesson slugs in order, each lesson is one YAML file, and each lesson holds an ordered list of typed blocks.">
</figure>

Then the system. The rule here is to build what we need right now, with a design that can grow. Music does not have a database because it does not need one. Lessons are YAML files stored in git, made of typed blocks (text, diagrams, videos and so on). Astro does most of the heavy lifting: on every deploy to Cloudflare Pages the YAML is compiled into static HTML. The videos are hosted on YouTube and embedded.

<figure class="theme-diagram">
  <img class="theme-diagram__light" src="/images/music-system-design.svg" width="760" height="474" alt="System design: pull requests go to a GitHub repo, Cloudflare Pages builds every push to main and publishes static files to the CDN, and the browser loads the site over HTTPS and the lesson videos from YouTube.">
  <img class="theme-diagram__dark" src="/images/music-system-design-dark.svg" width="760" height="474" alt="System design: pull requests go to a GitHub repo, Cloudflare Pages builds every push to main and publishes static files to the CDN, and the browser loads the site over HTTPS and the lesson videos from YouTube.">
</figure>

Only then do I start a new session and tell the agent to implement the project. I ask it to read the designs and, if something is not clear, to ask instead of guessing. It comes back with a few questions, we settle them, and it builds the app. Because the guardrails and the verification were there from the first line, it landed in a good place: I ran it locally, navigated around, and it was fast and correct.

The look came after. I asked the agent to workshop a few designs from my hints about the layout, picked one, and we worked together on making it beautiful. If you do this, add "no layout shift" to what you verify.

Deployment was the easy part. These days it is as easy as "deploy this app to" wherever you decided. I went with Cloudflare, because I have been working with Cloudflare Workers and really enjoy the developer experience there.

## What the guardrails did not catch

Everything passed. Then I picked up a guitar, opened the first lesson on my phone, and immediately found problems.

I could not understand some finger positions from text alone, so I added diagrams. The chords and keys just sat there on the page, although what I wanted was to tap a chord and hear it, so I made them playable. No lint rule would have told me any of this. The lessons themselves were written by a different agent, with references and sources so the content can be trusted, but whether a lesson actually teaches you is something you only find out with the instrument in your hands.

Guardrails give you correct code. They do not give you a good product. That part is still yours, and it is the best part: polishing, and adding more and more taste.

## Before your next project

- Write the rules down in the repo and enforce them with tools, not with prompts.
- Ban comments, so the agent has to fix a problem instead of describing it.
- Give the agent one command that verifies everything, and tell it to use it.
- Design the API, the data model and the system before the agent writes any code.
- Use the thing yourself. Only you can notice what is missing.

That is how I built music, and how I am finally learning to play the guitar.
