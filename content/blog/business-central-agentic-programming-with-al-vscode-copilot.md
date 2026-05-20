+++
date = '2026-05-20T10:30:00+01:00'
draft = false
title = 'Agentic programming for Business Central with AL, VS Code, and Copilot'
+++

Business Central development is getting a much better agentic programming story.

The important change is not that Copilot can generate AL code. That has been possible for a while, with mixed results depending on the context you give it.

The more interesting change is that the **AL extension and AL tooling are now exposing real development actions to agents**.

Microsoft documents this under the new [AI agent tools for AL development](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/al-agent-tools/al-agent-tools-overview). These tools let GitHub Copilot in VS Code, and MCP-compatible agents outside VS Code, perform Business Central development tasks such as building, publishing, downloading symbols, reading diagnostics, searching symbols, running tests, and debugging.

That changes the shape of the workflow.

Instead of asking an agent to guess what is wrong from a pasted compiler error, we can ask it to use the same tooling we use every day.

## From chat assistant to development loop

In my previous post about my [Business Central dev workflow with Glaze WM and AI agents](/blog/business-central-dev-workflow-with-glazewm-and-agents/), the main idea was that agents become useful when they live inside a real development lane.

That means:

- a real repository
- a real branch
- a real Business Central instance
- a predictable place on screen
- a repeatable way to compile, publish, test, debug, and review

The recent AL and VS Code changes fit directly into that idea.

For Business Central, agentic programming cannot just mean "write some AL for me". The useful version is closer to this:

1. inspect the existing AL project
2. search symbols and dependencies
3. make a small change
4. compile or build
5. read diagnostics
6. publish to the right environment
7. run the relevant tests
8. attach the debugger
9. help reason about the runtime behavior

That loop is where agents start to feel practical.

## Why VS Code matters again

I like command-line workflows. I use Claude Code and Codex a lot, and the AL MCP server is a big step because it gives non-VS Code agents access to official AL tooling.

But the newest VS Code integration is important because some actions only make sense inside the editor.

The clearest example is debugging.

Microsoft's [AL debugging tools](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/al-agent-tools/al-tool-debug) expose `al_debug`, `al_setbreakpoint`, and `al_snapshotdebugging` to Copilot in VS Code. These tools are VS Code-only because they interact with the VS Code debugger, breakpoints, Problems panel, and snapshot debugging views.

That distinction matters.

An MCP agent can compile, build, publish, download symbols, and inspect diagnostics. But when the job is to control the interactive debugger, VS Code is still the right place.

That makes Copilot Agent mode more useful for Business Central than a plain chat window. It can sit in the same editor where the breakpoints, launch configuration, source files, and debug session already exist.

## Debugging with Copilot

The debugger integration is the part I find most interesting.

The old flow is familiar:

1. set a breakpoint manually
2. publish the app
3. reproduce the scenario in Business Central
4. step through the AL code
5. inspect variables
6. adjust the code
7. repeat

With Copilot using the AL tools, that loop can become more conversational without losing the normal debugger workflow.

For example, I can ask Copilot to start debugging without republishing. Behind the scenes it can call `al_debug`, which is equivalent to the VS Code **Start Without Publishing** action.

That is useful when the extension is already deployed and I only want to attach the debugger to investigate a specific scenario.

I can also ask it to set a breakpoint at a specific line or in a specific AL file. Copilot can use `al_setbreakpoint` instead of just telling me where I should click.

That is a small but important difference.

An agent that can only explain what to do still leaves the mechanical work to me. An agent that can place the breakpoint, attach the debugger, and then help interpret what happened is much closer to the kind of assistant I actually want during development.

Snapshot debugging also becomes more interesting here. Instead of manually walking through the setup every time, Copilot can help initialize the snapshot, let me reproduce the scenario, finish the capture, and open the trace for inspection.

The human still owns the judgment. But the repetitive debugger setup becomes easier to drive.

## Publishing through AL tools

Publishing is another place where the agent needs real tools, not just code generation.

The [al_publish tool](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/al-agent-tools/al-tool-publish) can deploy an AL extension to a Business Central cloud or on-premises environment. In VS Code, Copilot can call it using the connection details from `launch.json`.

That means prompts like this become reasonable:

```text
Publish this extension to my sandbox without starting the debugger.
```

Or:

```text
Publish the full dependency tree and show me any errors.
```

The tool supports different publishing modes, including full publish, incremental RAD publish in VS Code, full dependency tree publishing, and skip-build publishing.

That is exactly the kind of operational surface agents need.

Business Central development often fails in the steps around the code, not only in the code itself. Dependencies are missing. Symbols are stale. The wrong environment is selected. The app builds locally but fails when deployed. The schema update mode matters.

An agent that can call the publishing tool, then call diagnostics, then help fix the actual errors is much more useful than an agent that only edits `.al` files.

## Compile, build, diagnostics, and symbols

The other core tools are just as important:

- `al_build` builds the AL project and generates the `.app` package
- `al_compile` validates AL code without generating an `.app`, available through MCP
- `al_getdiagnostics` retrieves compiler diagnostics
- `al_downloadsymbols` downloads dependent symbols
- `al_symbolsearch` searches AL symbols across the project and dependencies

This is where the workflow starts to look like a proper agent loop.

The agent can make a change, compile it, read the errors, search the related symbols, make a smaller correction, and compile again.

That does not remove the need for review. It does remove some of the slow back-and-forth where the agent writes code without being able to validate it.

For AL specifically, symbol search is a big deal. Business Central code is full of events, table extensions, page extensions, codeunits, enums, interfaces, and dependencies. If the agent cannot search the real symbols, it is going to hallucinate names or make weak guesses.

With the AL tools, the agent has a better chance of grounding its changes in the actual project.

## Running and debugging tests from VS Code

The other recent change that fits this workflow is the new AL test runner experience in VS Code.

Microsoft documents this as [running AL tests from Visual Studio Code with Test Explorer](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-test-explorer-vscode). The release plan also calls out the practical benefit: developers can discover tests in the project, run and debug them, and get coverage results without switching to an external test runner or the Business Central web client.

That is a big improvement for agentic programming.

If tests are visible in VS Code's Test Explorer, Copilot has a much better place to operate from. The agent can help set up the test project, generate or adjust test code, run the tests, inspect failures, and then debug a failing test when the result alone is not enough.

That last part matters.

For Business Central work, a failing test is often not just a red assertion. It may involve setup data, event subscribers, permissions, posting routines, test isolation, or a specific order of operations inside the application code. Being able to run the test from VS Code and then debug it from the same environment makes the loop much tighter.

The workflow becomes:

1. ask the agent to make the change
2. build or compile the app
3. publish if needed
4. run the related tests from VS Code
5. debug the failing test when the failure needs runtime inspection
6. iterate with diagnostics, symbols, and debugger context available

That is much stronger than asking an agent to "fix the tests" from a pasted error message.

The agent can now work closer to the way a developer works: run the test, inspect the result, attach the debugger when needed, and use the actual AL project context instead of guessing.

## VS Code and MCP are different surfaces

There are now two useful surfaces for agentic AL development:

1. **VS Code Language Model Tools**
2. **AL MCP Server**

The VS Code surface is best when I am working interactively with GitHub Copilot in the editor. It has access to editor-specific actions, including debugging.

The MCP surface is best when I want Claude Code, Codex, or another MCP-compatible agent to work outside VS Code. Microsoft documents the AL MCP server as a standalone server started with `altool launchmcpserver`. It exposes tools over STDIO or HTTP and does not require the agent itself to run inside VS Code.

That means the two surfaces are complementary.

For my workflow, I see it like this:

- use MCP agents for broader implementation work, compile loops, diagnostics, and project inspection
- use VS Code with Copilot when the task needs debugger integration, breakpoints, snapshot debugging, Test Explorer, or editor state
- keep Business Central instances isolated per worktree so each agent has a real place to publish and test

This is very close to the workflow I already use with multiple agents and multiple Business Central instances. The difference is that the official tooling is now catching up to that way of working.

## What this means for Business Central developers

The practical result is that AL development is becoming more automatable without becoming detached from the normal Business Central workflow.

That is the balance I care about.

I do not want an agent inventing a separate development process. I want it to use the same Business Central concepts I already use: `app.json`, symbols, dependencies, launch configurations, publishing, debugging, diagnostics, and isolated environments.

The recent AL and VS Code changes move in that direction.

They make it possible for Copilot to do more than suggest code. It can build the project, publish it, run tests, attach the debugger, set breakpoints, inspect diagnostics, and help iterate.

That is a better model for agentic programming in Business Central.

The agent is not replacing the developer. It is joining the development loop.

## Final thought

For me, the next step is not to ask whether AI can write AL.

The better question is whether the agent can participate in the full Business Central development cycle.

Can it compile? Can it publish? Can it run the tests? Can it read diagnostics? Can it search the actual symbols? Can it attach the debugger? Can it help me understand what happened after I reproduce the issue?

With the latest AL and VS Code tooling, the answer is increasingly yes.

That is what makes this interesting.

Agentic programming for Business Central is becoming less about a chatbot beside the editor and more about a tool-aware development partner inside the same workflow.
