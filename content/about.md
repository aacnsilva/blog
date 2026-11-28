+++
title = "About"
menu = "main"
weight = 20
+++

# About me

## Senior Software Engineer moving into applied AI, with a background in ERP and backend systems

I'm António, a software engineer from **Porto, Portugal**, with about **11 years** of experience. I'm moving my work into applied AI and AI application engineering: LLM APIs, agents, evals, and AI apps that run in production.

### What I'm up to

**Focus:** Applied AI and AI product engineering. I build with LLM APIs, write agents, and measure them with evals before and after they ship.

**AI solutions:** Agents and apps built for one specific business process, together with the process improvement and optimisation work around them.

**Also:** Backend and distributed systems.

**How I work:** Agent-first, every day. Claude Code is my main tool, and I also use Cursor and Grok Bot.

**Learning:** I've earned three Claude Academy badges: Building with the Claude API, MCP: Advanced topics, and Claude Code in action. I'm studying for Microsoft AI-103, LangChain LCAE, and AWS AIP-C01.

### Projects

**[Justa](/justa/)** is a household-expenses app for iPhone, written in SwiftUI and [live on the App Store](https://apps.apple.com/us/app/justa-household-management/id6813808595). Its assistant, Ask Justa, is backed by an eval suite of 71 cases. Over three full runs, 206 of 213 checks passed, with a mean score of 4.69 out of 5. I wrote up the choices behind it in [Evaluating the Justa agent](/evaluating-the-justa-agent/).

**[Animal Racing Social](https://racing.aacnsilva.com)** is a live racing game, with the [source on GitHub](https://github.com/aacnsilva/animal-racing-social). The race simulation is written in Rust and prices its odds with Monte Carlo runs. Around it are TypeScript services that talk over Kafka with Protobuf messages, races sealed with AES-GCM, an outbox with a dead-letter queue, an idempotent wallet ledger, Playwright tests in CI, and OpenTelemetry with Grafana. I made the design calls.

**[Music lessons](https://music.aacnsilva.com)** is a site of interactive music lessons for guitar and piano. It has two courses, Guitar from zero and Piano from zero, each with 25 lessons (about 7 hours of practice). The lessons are short and guided, start from zero, and each one ends with practice against a metronome. I built it with Astro and TypeScript and write the lesson content as YAML. The site also serves a static JSON API of the courses and lessons, and it is hosted on Cloudflare Pages.

**[Ensino](https://ensino.aacnsilva.com)** is a study platform for nine Portuguese national exams, with 134 lessons, 70 quizzes, spaced repetition, and a grounded AI tutor. The tutor runs on my multi-tenant `ai-platform` with RAG over 164 documents, a 10-word answer-leakage guard, and two eval suites (73 retrieval cases and 96 end-to-end conversations). I wrote up how it works in [Why "just add RAG" still gets the exam wrong](/why-just-add-rag-still-gets-the-exam-wrong/).

### Background

My depth is in ERP and finance systems: Microsoft Dynamics 365 Business Central and Finance & Operations, at LS Retail (an Aptos company) and, earlier, at myPartner. That taught me how business rules, integrations, and money actually move through a company. I treat it as background I bring to AI work, not as the thing I plan to specialise in.

I care about clear tests, sensible automation, and tools that reduce friction without adding ceremony.

### What you'll find here

**[Blog](/)** — Technical posts and notes  
**[Resume](/resume/)** — Professional background

### Let's connect

**Email:** [antoniosilva1017@gmail.com](mailto:antoniosilva1017@gmail.com)  
**LinkedIn:** [linkedin.com/in/aacnsilva](https://www.linkedin.com/in/aacnsilva/)  
**GitHub:** [github.com/aacnsilva](https://github.com/aacnsilva)
