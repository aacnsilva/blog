+++
date = '2026-05-26T18:10:00+00:00'
draft = false
title = 'Beyond VS Code: Microsoft’s AL LSP Opens Business Central Development to Agents and New Editors'
+++

Microsoft's new AL Language Server Protocol support is a bigger deal than it might look at first glance.

On the surface, the announcement is about a new ALTool command:

```bash
altool launchlspserver [<projects>...] [options]
```

That command launches an AL Language Server Protocol server. If you have worked with modern editors, you already know what that usually means: hover, completions, go-to-definition, find references, rename, formatting, document symbols, inlay hints, folding ranges, and similar code intelligence features.

But for Business Central developers, the more interesting point is this:

> Microsoft is separating AL language intelligence from the Visual Studio Code experience.

That does not mean VS Code stops being important. VS Code remains the primary and best-supported AL development environment. But by exposing AL intelligence through LSP, Microsoft is opening the door for other editors, automation systems, and AI coding agents to understand AL code in a much more reliable way.

## Why LSP matters

The Language Server Protocol is the standard contract used by editors and IDEs to talk to language tooling. Instead of every editor implementing its own parser, symbol engine, autocomplete system, and rename logic, an editor can talk to a language server.

The editor asks questions like:

- What symbol is under the cursor?
- Where is this procedure defined?
- What references this field?
- What completions are valid here?
- Can this symbol be renamed safely?
- What diagnostics apply to this file?

The language server answers using structured data.

For AL, this matters because Business Central code is not just text. It has object types, table extensions, page extensions, procedures, events, subscribers, app dependencies, symbol packages, test projects, visibility rules, and workspace relationships.

A text search can tell you where the word `Customer` appears. The AL language server can help distinguish whether that occurrence is a record variable, a table reference, a caption, a comment, or part of a completely different symbol.

That distinction is critical.

## Why this is important for agents

AI coding agents are only as good as the tools they can use.

Without language intelligence, an agent working on AL code has to rely heavily on file search, regular expressions, and reading large chunks of source. That can work for simple tasks, but it gets fragile quickly.

For example, suppose an agent is asked to rename a procedure across a Business Central extension. A text-based approach might:

- miss references in related projects
- update comments or strings accidentally
- confuse procedures with the same name in different objects
- overlook dependencies exposed through `app.json`
- fail to understand visibility rules

With LSP, the agent can ask the AL language server for the actual references. That means it can operate on symbols rather than character matches.

A useful way to think about it is:

> Text search tells an agent where characters appear. LSP tells it what the code means.

That is the difference between a helpful autocomplete bot and a much more capable coding assistant.

## LSP and MCP are two parts of the new AL tooling story

The AL LSP is especially interesting because Microsoft has also introduced the AL MCP server.

These two tools serve different purposes.

| Capability | AL LSP | AL MCP Server |
| --- | --- | --- |
| Go to definition | Yes | No |
| Find references | Yes | Limited or no |
| Hover and completions | Yes | No |
| Rename and formatting | Yes | No |
| Document symbols | Yes | No |
| Compile or build | No | Yes |
| Publish extension | No | Yes |
| Download symbols | No | Yes |
| Retrieve diagnostics | Editor-style diagnostics | Explicit tool call |
| Best for | Understanding and editing code | Running AL development actions |

The LSP gives an agent semantic understanding.

The MCP server gives an agent tools to act.

Together, they create a much more complete agentic development loop:

1. Use LSP to understand the AL workspace.
2. Navigate definitions and references semantically.
3. Make code changes.
4. Use MCP to compile or build.
5. Read diagnostics.
6. Fix issues.
7. Repeat until the extension is valid.

That is a major improvement over agents simply editing files and hoping the compiler catches everything later.

## What this means for other IDEs

Because LSP is an open protocol, AL is no longer tied as tightly to one editor surface.

In principle, any editor or tool that can launch an LSP server over stdio can integrate with ALTool's LSP support.

That could include:

- Neovim
- Emacs
- JetBrains IDEs with LSP support
- Zed
- Sublime Text
- Helix
- browser-based development environments
- internal developer portals
- custom AI coding tools
- CI/CD review agents

This does not mean every editor suddenly has a polished Business Central experience out of the box. There will still be configuration work, especially around package caches, workspace files, symbols, analyzers, and authentication for connected operations.

But the foundation is now much better.

Instead of alternative editors needing to reverse-engineer AL behavior, they can speak the same protocol other modern development tools already use.

## Why Business Central workspaces need semantic tooling

Business Central projects are often more complex than they look from the outside.

A real AL workspace may include:

- a main app
- one or more test apps
- dependent extensions
- shared libraries
- AppSource dependencies
- per-tenant extensions
- symbol packages
- custom analyzers
- rulesets
- `.code-workspace` configuration
- multiple `app.json` dependency relationships

Microsoft's AL LSP support is designed with this in mind. The documentation describes support for multi-project workspaces and dependency-aware behavior, including relationships such as `internalsVisibleTo` and `propagateDependencies`.

That matters because a generic coding agent does not naturally understand how a Business Central solution is wired together. The language server can.

For agents, that means less guessing.

For developers, that means AI tools can become more trustworthy when operating on AL code.

## A practical example

Imagine a developer asks an agent:

> Rename this posting helper procedure and update all references across the app and test project.

A weak agent might search for the procedure name as text and edit every match.

A better AL-aware agent could:

1. Start `altool launchlspserver` with the relevant projects.
2. Ask the LSP server for the symbol definition.
3. Request all references across the workspace.
4. Apply a symbol-aware rename.
5. Use the AL MCP server to compile the project.
6. Retrieve diagnostics.
7. Fix any remaining compiler or analyzer issues.

That is a very different workflow.

The agent is not just editing text. It is participating in the same semantic development model that a human gets from an IDE.

## This is not the end of VS Code

It is worth being clear: this does not make VS Code irrelevant for AL development.

VS Code still provides the most complete Business Central developer experience. Debugging, publishing workflows, launch configurations, authentication flows, and the broader AL extension experience are still deeply associated with VS Code.

The better interpretation is this:

> AL development is becoming protocol-driven.

VS Code remains a first-class client. But now agents, alternative editors, and automation tools can become clients too.

That is good for the ecosystem.

## What to watch next

The interesting next step is not just whether the LSP exists. It is how tools start using it.

Things to watch:

- Will non-VS Code editors publish AL setup guides?
- Will AI coding tools add first-class AL LSP support?
- Will Business Central partners build internal agent workflows around ALTool?
- Will CI systems use LSP for smarter code review?
- Will MCP and LSP be combined into reusable AL agent templates?

The real opportunity is not autocomplete in another editor. It is reliable automation.

## Final thought

Microsoft's AL LSP is an important step toward making Business Central development more open, more automatable, and more agent-friendly.

The most exciting part is not that AL code can now be understood outside VS Code. It is that agents can now work with AL through language-aware tooling instead of treating a Business Central project as a pile of text files.

For a language and ecosystem as specialized as AL, that distinction matters.

LSP gives tools understanding. MCP gives tools actions. Together, they point toward a future where Business Central development can happen across more editors, more environments, and more intelligent workflows.
