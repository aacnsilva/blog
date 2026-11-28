+++
date = '2026-11-28T12:00:00+00:00'
draft = false
title = 'Why "just add RAG" still gets the exam wrong'
+++

Ask a frontier model whether a Portuguese 12th-grader should study Pascal's triangle or mathematical induction for the 2027 Matemática A national exam, and it will happily say yes. Its training data is full of past exams where both appeared. Both were dropped under the 2023 curriculum that governs the 2027 exam. Ask what value of *g* is on the Physics and Chemistry exam table, and it will often give you 9.81 or 10 instead of the official 9.80.

When a study tutor gets that wrong, it is not a harmless hallucination—it sends a 17-year-old down the wrong study path for an exam that decides university entry.

A month ago, in [Evaluating the Justa agent](/evaluating-the-justa-agent/), I argued against building RAG. For Justa, that was the right call: the entire product manual was 3,800 words in one file, and caching it in the system prompt was cheaper and more reliable than retrieval. On [Ensino](https://ensino.aacnsilva.com), a study platform for nine Portuguese national exams, that assumption breaks. The tutor's corpus is 164 documents—134 published lessons and 30 official reference documents—totalling 1,613 chunks and nearly 79,000 words of reference specs alone. Stuffing it into every turn does not scale, and relying on the model's parametric memory teaches the wrong exam.

I went from zero to a grounded tutor across all nine subjects quickly because I did not build the AI plumbing from scratch—I already had my own multi-tenant `ai-platform` handling RAG, quotas, model routing, and sessions. That freed me to focus on the parts where naive RAG actually fails: the content pipeline, answer-leakage guards, cross-subject chunking, end-to-end citations, and layered evals.

## Standing on `ai-platform`

Most teams lose weeks on RAG before they ever test a single query: setting up a vector store, wiring chunking and embeddings, building rate limits, streaming responses, and tracking spend. I skipped all of that because Ensino is simply one tenant on `ai-platform`, a multi-tenant Cloudflare Worker with Durable Objects that already sits behind my other products.

Out of the box, `ai-platform` gave Ensino:

- **Per-tenant RAG:** a document admin API, heading-aware chunking, hybrid keyword (BM25) and vector ranking, and plan-scoped visibility, isolated per tenant.
- **Auth, plans, and spend limits:** signed 10-minute account tokens (carrying only subject and plan—never a student's name, email, or handle), per-plan monthly message caps, per-IP rate limits, and a monthly EUR spend pool with reserve-and-settle accounting.
- **Model routing and sessions:** capability tiers (`fast` on Claude Haiku 5.5 for the free plan, `balanced` on Claude Sonnet 5.5 for Plus), circuit breakers, stored conversations with turn locks and idempotent retries, streaming, and read-only tools behind an egress allowlist.
- **A built-in eval pass:** tenants on the dev deployment can opt in with a secret hash so eval runners bypass browser origins and receive full traces—model, rounds, tools called, and cost—with every answer.

Because the runtime holds all of those mechanisms, Ensino's entire integration is one tenant config (`tools/tutor/tenant.ts`), one sync script (`tools/tutor/push.ts`), and a token-signing endpoint. I spent my time on exam accuracy instead of infrastructure.

## A content pipeline that guards the index

Everything students study and everything the tutor retrieves flows from one git-backed Markdown pipeline under `content/`: 9 subjects, 67 units, 134 lessons, 70 quizzes, 883 questions across seven question types, 42 official sources, and 30 curated reference documents. Content IDs are permanent (retired questions are archived, never deleted). `pnpm content:sync` upserts everything into D1 in one transaction; `pnpm tutor:push` syncs the published lessons and reference documents into `ai-platform` and deletes anything stale.

That shared pipeline creates a hazard most RAG guides ignore: your platform stores explanations, grading criteria, and model answers right next to the teaching material. A tutor is supposed to guide a student with hints, not hand over the answer key. If solutions slip into the retrieval store, the tutor retrieves them and leaks them.

I enforce the boundary in two places:

- **Split read-only tools.** `get_lesson` reads the public lesson API, which returns reading blocks and question prompts while stripping answers, grading criteria, and explanations. `search_knowledge` searches the uploaded corpus: published lessons (prompts only) plus `content/reference/`.
- **A 10-word answer-overlap guard.** Even with separate files, it is easy for a human or an agent authoring reference material to paraphrase a quiz explanation too closely. `pnpm content:check` builds a 10-word shingle index across every question's explanation, open-question grading rubric, and model answer across all 883 questions, then scans every reference file against it. Any 10-word overlap fails the build.

The guard paid for itself immediately. While expanding the reference pack across six new subjects, it caught overlapping phrasing in Filosofia, História A, Biologia e Geologia, Economia A, and MACS before a single document reached the index.

## Nine subjects collide in one index

Even with clean documents, naive chunking breaks as soon as a corpus spans multiple subjects. School curricula reuse the same words across disciplines:

- **Funções** means mathematical functions in Matemática A and syntactic functions in Português.
- **Energia** means ionization energy in chemistry and kinetic energy in physics.
- **Intervalo de confiança** appears in both Matemática A (exam 635) and MACS (exam 835), with different scope and formulas.

When a chunker slices a document into 1,200-character blocks, middle chunks lose the title at the top of the file. A query for "funções sintáticas" or "intervalo de confiança no formulário de Matemática A" starts pulling chunks from the wrong subject.

I fixed this in the chunker and the content linter together. The chunker tracks the active Markdown heading and prepends it to every chunk produced under that section. `pnpm content:check` then enforces strict heading prefixes on every reference file: the H1 must carry the subject name and exam code (for example, `# Matemática A (635) · Informação-Prova do exame nacional de 2027`), and every H2 must start with a subject prefix (`## Mat A`, `## FQA`, `## Port`, `## MACS`, and so on).

Because BM25 indexes the document title plus the chunk text—with diacritic folding so `funcao` and `função` match—every 1,200-character chunk carries its own subject identity. A middle chunk can no longer masquerade as another exam.

## Citations are a data pipeline, not a prompt line

Telling a model "always cite your sources" without putting verifiable sources inside the chunks it retrieves only teaches it to invent plausible-looking citations. Citations have to travel through the whole pipeline:

1. **One canonical registry.** `content/sources.json` holds 42 official sources—EduQA 2027 *Informações-Prova*, DGE *Aprendizagens Essenciais*, official formula sheets, and exam archives—each with a permanent ID, title, publisher, URL, and retrieval date.
2. **Same sources in the UI and in RAG.** Lessons declare `sources: [...]` in frontmatter. The lesson page renders them as clickable links at the bottom of the lesson, and `tutor:push` appends a `## Fontes` block with those exact titles and URLs to every lesson uploaded to `ai-platform`.
3. **Citations inside every reference section.** In `content/reference/`, I do not put links only at the bottom of a 15,000-character file where chunking would orphan them. Every H2 section ends with its own `Fonte: <title> — <url>` line. Whichever 1,200-character chunk BM25 returns, the exact official document and URL come along for the ride.
4. **Prompt rules that prefer honesty over guessing.** The tutor prompt instructs the model to cite the source returned by `get_lesson` or `search_knowledge` at the end of its answer, never invent citations, dates, or formulas, and say clearly when it is not sure.

## Two eval suites, four layers of grading

When a RAG tutor gives a wrong answer, there are two different bugs it could be: retrieval never surfaced the right chunk, or the model ignored the chunk it was given. Debugging both at once against a live LLM is slow and expensive, so I split evaluation into two suites.

**1. Offline retrieval benchmark (`pnpm evals:retrieval`).** It mirrors `ai-platform`'s exact 1,200-character chunker and diacritic-insensitive BM25 ranker locally, with zero API calls. It runs 73 gold queries across all nine subjects, general exam rules, and 15 cross-subject disambiguation traps. Each case checks Recall@5, required factual terms in the retrieved text, and zero forbidden documents at rank 1. The suite fails below 90% Recall@5 or on any forbidden #1 hit:

| Group | Cases | Recall@5 | MRR@5 | Disambiguation |
| --- | --- | --- | --- | --- |
| 9 subjects + general rules | 58 | 100.0% | 0.991 | 100.0% |
| Cross-subject traps (`desambiguacao`) | 15 | 100.0% | 1.000 | 100.0% |
| **Total** | **73** | **100.0%** | **0.993** | **100.0%** |

A `--live` flag checks the deployed `ai-platform` tenant against the repository and flags any document that is missing, empty, or keyword-only without embeddings.

**2. End-to-end tutor evals (`pnpm evals`).** 96 student conversations across 10 groups—hints, concepts, checking attempts, refusing direct answers, study advice, off-topic, prompt injection, safety, 2027 exam scope (`ambito-2027`, 16 cases), and grounded citations (`citacao`, 19 cases). Every answer goes through three grading layers:

- **Code checks:** European Portuguese ("tu", never "você"), no leaked answers, required tool calls, word caps, ending with a question when coaching, exact table values, and a real citation marker.
- **Model judge:** Claude Opus 5.5 grades each case's rubric, factual accuracy against the reference, tone, and safety—without seeing the code-check results—and flags low-confidence judgments.
- **Human review:** the admin dashboard's *Avaliações* tab queues every failure, every low-confidence judgment, and one in five passes, tracking how often the judge agrees with a person.

## What the evals caught that RAG did not fix

100% Recall@5 in the retrieval benchmark did not give me 100% grounded answers from the tutor. Running the full 96-case suite against both tiers exposed a failure mode that pure retrieval metrics never see:

| Tier (model) | `citacao` checks | `ambito-2027` checks | Cost / answer |
| --- | --- | --- | --- |
| Balanced (Claude Sonnet 5.5) | 18 / 19 | 12 / 16 | €0.011 |
| Fast (Claude Haiku 5.5) | 17 / 19 | 10 / 16 | €0.0004 |

Look at why the `ambito-2027` cases failed. The right chunk was in the index, and the offline benchmark ranked it #1. The model simply **did not call `search_knowledge`**.

On a História A scope question—and on a Filosofia scope question on Haiku—the model felt confident enough to answer from parametric memory, skipped the search tool, dropped the citation, and got the 2027 exam scope wrong. On two other cases, it called the tool and got the concept right, but omitted exact table numbers (`9,80` for *g*, or `1,645 / 1,960 / 2,576` for confidence-interval z-values) that the student specifically asked for.

That is the real lesson from building both layers. Retrieval gives the model access to ground truth; it does not give the model the discipline to look before speaking. Without evals that assert tool calls and exact values, those answers read smoothly and look right to a casual skim—right up until a student trusts them in an exam.

## What actually holds up

- **Build the AI platform once, reuse it per product.** Having multi-tenant RAG, quotas, routing, and eval traces in `ai-platform` meant Ensino's tutor was a tenant file, not a rewrite.
- **Lint the corpus for answer leakage.** A 10-word shingle check against explanations and rubrics is cheap and catches leaks that manual review misses.
- **Make every chunk self-contained.** Prefix headings with the subject and put the `Fonte:` line inside each section so no chunk loses its scope or its citation.
- **Separate retrieval evals from model evals.** Test Recall@5, MRR, and negative disambiguation traps offline before spending tokens on end-to-end runs.
- **Check tool calls, not just prose.** A RAG pipeline the model chooses to skip is just parametric memory with extra infrastructure.
