+++
date = '2026-03-09T10:30:00+00:00'
draft = false
title = 'Business Central AL tools with Claude Code and Codex'
+++

One of the more interesting recent changes in the Business Central tooling story is that Microsoft now provides official **AL Development Tools** that can also launch an MCP server for an AL project.

That makes Claude Code and Codex much more relevant for AL work than they used to be.

Instead of treating AL development as something that only works properly inside VS Code, we can now use Microsoft's command-line tooling directly from an agent and let it work against a real AL app.

## The Microsoft-only setup

There is one Microsoft package involved here:

1. The `Microsoft.Dynamics.BusinessCentral.Development.Tools` NuGet package

The AL Development Tools package is documented here:

- [AL Development Tools package](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-al-tool-package)
- [Microsoft.Dynamics.BusinessCentral.Development.Tools](https://www.nuget.org/packages/Microsoft.Dynamics.BusinessCentral.Development.Tools)

The package exposes the `al` command-line tool. Microsoft documents commands such as `CompilePackage`, `GetPackageManifest`, `CreateSymbolPackage`, and `GetLatestSupportedRuntimeVersion`.

Installation is straightforward:

```bash
dotnet tool install Microsoft.Dynamics.BusinessCentral.Development.Tools --interactive --prerelease --global
```

Then:

```bash
al help
```

After installing the AL tools, the important part is that you can launch the MCP server directly from the Microsoft `al` tool against a specific AL project directory:

```bash
al launchmcpserver <project directory>
```

That is the part I had wrong before. You do not need some separate community AL MCP package for this workflow.

## Where Claude Code and Codex fit

The important point is that the NuGet-delivered `al` tool is doing both jobs here: it gives the agent official Microsoft command-line AL tooling and it also launches the MCP server for the AL project.

So the practical flow becomes:

1. Install the Microsoft AL tools from NuGet
2. Add the MCP server to Claude Code or Codex by invoking `al`
3. Open the agent in the AL project folder you want to work on

For Claude Code, the setup can be as simple as:

```bash
claude mcp add --transport stdio almcp -- al launchmcp .
```

That is a nice fit for normal project-based work. You open Claude Code inside a given AL project folder, the MCP server starts against that project, and the connection is automatically scoped to the project you are in. If you have multiple AL projects, you just open Claude Code in the one you want to work on, exactly like you normally would.

Once that is in place, the agent can work directly against the AL project in the current folder with Microsoft tooling behind it.

## What this enables

This setup is enough to make agent-assisted AL development practical.

With the official `al` tool, the agent can compile packages, inspect app manifests, create symbol packages, and generally work with the Microsoft-supported CLI surface.

In practice, that gives us a useful development loop in Claude Code or Codex:

- work on the AL app
- compile it with the official Microsoft toolchain
- publish through the existing local workflow
- run tests through the existing local workflow
- iterate on errors and failures

The important point is not that the `al` tool suddenly replaces every part of the development pipeline. It does not. The point is that Microsoft now provides enough official tooling that an AI agent can participate in the loop in a serious way.

## The drawback

There is still one important limitation: this is **not** full feature parity with the VS Code AL extension.

The NuGet package gives us a focused command-line surface, not the full editor experience. Some VS Code-specific AL settings and workflows do not carry over cleanly. One example is `al.packageCachePath`, which is documented as an AL Language extension setting in VS Code.

That is the main drawback I see today. The Microsoft tooling is now good enough to enable Claude Code and Codex workflows around AL, but it is still not the same thing as taking the whole VS Code AL experience and moving it into the command line.

## Final thought

That limitation aside, this is still a big step.

We can now use Microsoft's own AL tools to make Claude Code and Codex useful on AL projects: install the AL tools through NuGet, connect the MCP server through `al`, point the agent at the AL app, and let it work through the development loop with real tooling behind it.
