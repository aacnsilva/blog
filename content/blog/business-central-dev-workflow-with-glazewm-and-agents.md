+++
date = '2026-03-14T09:00:00+00:00'
draft = false
title = 'My Business Central dev workflow with Glaze WM and AI agents'
+++

I have been refining my development workflow around **Business Central**, **Claude Code**, **VS Code**, and a tiling window manager called **Glaze WM**.

The main goal is simple: reduce friction.

When I am working on a feature, I do not want to spend too much energy switching contexts, opening the same tools again and again, or manually wiring together the same DevOps steps. I want a setup where I can move fast, keep a clear mental model, and let agents handle the repetitive parts.

## Why Glaze WM matters in this setup

Glaze WM is the piece that makes the workflow feel operational instead of experimental.

I use it to organize my screen into a layout where I can keep **Claude Code** and **VS Code** visible and predictable. Instead of treating the agent as a separate tool that I occasionally open, I give it a permanent place in the workspace.

That sounds like a small detail, but it changes how I work.

When the terminal, the editor, and the running Business Central environment all have a stable place on screen, it becomes much easier to stay in flow. I am not hunting for windows. I am not constantly rearranging tabs. I know where each piece of the loop lives.

## Five workspaces, five agents

The part I like most in this setup is that I run **five workspaces**.

In those workspaces I have **agent one, two, three, four, and five**. Each agent has its own **Business Central instance**.

They are also all connected to the **same repository**, but not through the same checkout.

Each agent works in its own **git worktree**.

That means I can keep multiple branches from the same repo active at the same time, with each workspace mapped to its own worktree, its own agent session, and its own Business Central instance.

That isolation is important.

Each agent can deploy the app, make a change, and test it in its own environment without stepping on the others. It gives me parallel lanes for experimentation. If I want one agent exploring a bug, another validating a refactor, and another trying a different implementation, I can do that without collapsing everything into one shared instance.

Using `git worktree` is what makes this practical on one repo. I do not need to duplicate the repository five times or constantly stash and switch branches. Each lane stays attached to its own branch and can move independently.

In practice, this makes AI-assisted development much more useful for Business Central work.

The problem with many agent demos is that they look good until you need a real development loop. Once publishing, testing, instance management, and source control enter the picture, things get messy fast. Separate Business Central instances make the loop much more robust because each agent gets a safer sandbox to work in.

## Skills for the boring but important work

The other big part of my workflow is the set of **skills** I use with the agents.

I do not just want an agent that can suggest code. I want an agent that can participate in the full development process, especially the repetitive steps around Azure DevOps.

Two skills are especially useful for me.

### 1. Create branches from work items and gather context

When I start work, I can use a skill that creates a branch from a work item in **Azure DevOps** and gathers the relevant context.

That means the agent is not starting blind. It can pull the work item information, understand what is being asked, and begin from the same operational context I would normally assemble manually.

This is the kind of automation that saves more time than people first expect. The branch naming, the linkage to the work item, and the initial context gathering are not hard tasks, but they are exactly the kind of tasks that repeat often enough to deserve automation.

### 2. Create pull requests and link everything back

At the end of the work, I use another skill to create the **pull request** and link it back to the corresponding work items.

That closes the loop nicely.

Instead of treating source control, project tracking, and coding as separate activities, the workflow ties them together. The agent helps with implementation, but it also helps keep the surrounding DevOps process tidy.

That part matters a lot in team environments. A good workflow is not only about producing code faster. It is also about making the resulting work easier to review, track, and understand.

## Command-line tooling is what makes this real

Under the hood, these skills take advantage of command-line tooling, especially **Azure CLI** with the **Azure DevOps plugin**.

That is a key detail.

The more a workflow depends on clickable UI steps, the harder it is to make an agent genuinely useful. But once the workflow has a solid command-line surface, the agent can do real work instead of just acting like a chat companion beside the editor.

For me, that means the agent can:

- create branches from work items
- collect ticket context
- help with implementation
- create pull requests
- link pull requests and work items together

That is much closer to an actual development assistant than a code autocomplete tool.

## AI review before the human review

I also have skills for **code review**.

This is one of the areas where I think teams will get a lot of value very quickly.

Right now I use those skills in my own workflow, but I can easily imagine this becoming part of the company pipeline soon. When a developer creates a pull request, the first pass could come from AI: checking for obvious issues, reviewing patterns, and catching things that are easy to miss when moving fast.

I do not see that as a replacement for human review.

I see it as a stronger first filter.

If AI can handle the initial pass, human reviewers can spend more time on the parts that actually need judgment: architecture, trade-offs, business intent, and whether the implementation is the right one for the product.

## Wispr Flow for voice-driven development

Another tool that has become part of my daily workflow is **[Wispr Flow](https://wisprflow.ai/r/ANT%C3%93NIO49)**.

It is a voice-to-text tool that lets me dictate instead of type. That might sound minor, but when I am working across five workspaces, switching between agents, and moving between DevOps tasks, being able to speak my thoughts instead of typing them makes a real difference.

I use it for writing commit messages, describing pull requests, drafting work item notes, and even for talking through implementation ideas with the agent. It keeps my hands on the keyboard for code and lets my voice handle the rest.

If you want to try it, **[you can use my referral link to get a free month of the Pro plan](https://wisprflow.ai/r/ANT%C3%93NIO49)**.

## Why this workflow works for me

What I like about this setup is that it combines a few things that are individually useful into one system:

- a predictable desktop layout with Glaze WM
- separate workspaces for separate agents
- one shared repository split into multiple git worktrees
- isolated Business Central instances
- skills that automate Azure DevOps tasks
- AI-assisted review before the normal PR flow
- voice-driven input with Wispr Flow

None of these pieces alone is the full answer.

But together they create a workflow that feels much more capable than the usual “editor plus chatbot” story.

For Business Central development in particular, I think that matters. The work is not just about writing AL. It is also about deployment, validation, issue tracking, pull requests, and keeping a clean connection between code changes and business tasks.

That is the kind of environment where agents become genuinely useful when they are connected to the right tools and given a structured place in the workflow.

## Final thought

The interesting part is not that I have five agents on screen.

The interesting part is that each one can operate inside a real development lane with its own Business Central instance, while skills handle the repetitive DevOps plumbing around branches, pull requests, and reviews.

That is what makes the workflow practical for me.

It is not AI for the sake of AI. It is a setup that helps me keep momentum while working on Business Central apps.
