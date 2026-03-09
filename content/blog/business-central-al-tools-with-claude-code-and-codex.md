+++
date = '2026-03-09T10:30:00+00:00'
draft = false
title = 'Business Central AL tools with Claude Code and Codex'
+++

One of the more interesting recent changes in the Business Central tooling story is that Microsoft now provides official **AL Development Tools** that can launch an MCP server for an AL project.

That is the key piece. An MCP server means Claude Code and Codex can work against a real AL app with Microsoft tooling behind them, instead of treating AL development as something that only works properly inside VS Code.

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

Note that you might need to add the NuGet source first if it is not already configured in your environment. Check your NuGet sources with `dotnet nuget list source` and add `https://api.nuget.org/v3/index.json` if it is missing.

Then:

```bash
al help
```

After installing the AL tools, the important part is that you can launch the MCP server directly from the Microsoft `al` tool against a specific AL project directory:

```bash
al launchmcpserver <project directory>
```

## Where Claude Code and Codex fit

The practical flow is:

1. Install the Microsoft AL tools from NuGet
2. Add the MCP server to Claude Code or Codex by invoking `al`
3. Open the agent in the AL project folder you want to work on

For Claude Code:

```bash
claude mcp add --transport stdio almcp -- al launchmcpserver .
```

For Codex:

```bash
codex mcp add almcp -- al launchmcpserver .
```

Or add it directly to your `.codex/config.toml`:

```toml
[mcp_servers.almcp]
command = "al"
args = ["launchmcpserver", "."]
```

That is a nice fit for normal project-based work. You open the agent inside a given AL project folder, the MCP server starts against that project, and the connection is automatically scoped to the project you are in. If you have multiple AL projects, you just open the agent in the one you want to work on.

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

The NuGet package gives us a focused command-line surface, not the full editor experience. Some VS Code-specific AL settings and workflows do not carry over cleanly. One example is `al.packageCachePath`, which is documented as an AL Language extension setting in VS Code. Without it, the MCP server cannot locate the symbol packages for your dependencies, which means compilation fails with missing symbol errors. There is no workaround for this today.

That is the main drawback I see. The Microsoft tooling is now good enough to enable Claude Code and Codex workflows around AL, but it is still not the same thing as taking the whole VS Code AL experience and moving it into the command line.

## Final thought

That limitation aside, this is still a big step.

We can now use Microsoft's own AL tools to make Claude Code and Codex useful on AL projects: install the AL tools through NuGet, connect the MCP server through `al`, point the agent at the AL app, and let it work through the development loop with real tooling behind it.
