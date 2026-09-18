+++
date = '2026-09-18T10:30:00+01:00'
draft = false
title = 'How AL interfaces might finally grow without a version bump'
+++

I have an `IPaymentGateway` with `Authorize` and `Capture`. It is published. Partners implemented it. Then a customer asks for `Refund`.

**That is the moment a published AL interface stops being a contract and starts being a fossil.**

I am not writing this from a 29.0 sandbox. I have not compiled a default interface method. I am reading [What's new in update 29.0](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/whatsnew/whatsnew-update-29-0) the way I read language docs: slowly, and without filling in the blanks Microsoft left empty.

## The ISV story that never gets less true

You ship a payment add-in. The interesting bit is not your codeunit. It is the interface other apps implement so they can plug in their own gateway:

```al
interface "IPaymentGateway"
{
    procedure Authorize(var Request: Record "Payment Request"): Boolean
    procedure Capture(var Request: Record "Payment Request"): Boolean
}
```

That shape is the documented AL interface: a name, some signatures, no body. [Interfaces in AL](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al) still says the interface itself contains only signatures, and implementors use `implements`.

A year later, finance wants `Refund`.

If you add `Refund` to the published interface, every existing implementor is incomplete. AppSourceCop [AS0066](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/analyzers/appsourcecop-as0066) is the analyzer that exists specifically to stop you: *a new method to an interface that has been published must not be added, because dependent extensions may break.*

So you do the ISV dance. You leave `IPaymentGateway` alone. You invent `IPaymentGateway v2`. You version the enum. You write a blog post for partners. Nobody is going brrr. Everybody is merging mapping tables.

## v25: `extends` was the first real evolution path

Business Central 2024 release wave 2 (v25) gave us a documented way to grow the family without editing the original contract: [`interface … extends …`](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al-extend).

```al
interface "IPaymentGateway v2" extends "IPaymentGateway"
{
    procedure Refund(var Request: Record "Payment Request"): Boolean
}
```

That is honest. It is also a version bump in a nice keyword.

Callers that still hold `Interface "IPaymentGateway"` only see `Authorize` and `Capture`. New callers take the v2 interface. Old implementors stay valid for the old contract. New implementors must provide **every** inherited procedure plus `Refund`. If you want to detect the extra capability at runtime, v25 also documented `is` and `as`.

**`extends` grows the tree. It does not grow the node.**

The published interface is still frozen. AS0066 is still on the books. I am not claiming that rule went away — Learn still describes it that way.

![Four-step timeline: publish an AL interface, need a new method, the old pain of breaking or extending, then the BC 29.0 preview claim of default method bodies plus RequiredPending and analyzer rules.](/images/al-interface-evolution-bc29-preview.svg)

## The BC 29.0 preview claim

Then the 29.0 public preview what's-new table grew a development row I have wanted for years:

**Evolve AL interfaces with default implementations** — *Add default method bodies to AL interfaces so existing implementors continue working, then use RequiredPending and analyzer rules to introduce required methods safely.*

That is the whole public claim, as of writing. I am not going to invent a procedure-body syntax, a property name beyond `RequiredPending`, or a sample that pretends to compile. Full public syntax samples were thin when I wrote this.

Read that sentence the ISV way, though, and the `Refund` story changes shape:

1. You add `Refund` with a default body, so the partner codeunit that only did `Authorize` / `Capture` keeps compiling.
2. You use `RequiredPending` and analyzer rules to move that method from “there is a default” toward “you really should implement this.”
3. Existing implementors continue working while you do it.

If that is what actually ships, published interfaces can grow **in place**. No v2. No `extends` just to add one procedure. No “sorry, bump your dependency.”

**If.** Preview claim. Not “ships as.”

## The docs have not caught up, and that is the point

The [Interfaces in AL](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al) page still largely describes signature-only interfaces. It still tells you to avoid adding methods to published interfaces, and it still names AS0066 twice. It still says interfaces can only contain procedure declarations (AL0584, AL0585, AL0612).

It also has a one-liner under interface creation: *consider using default implementations for methods in interfaces to reduce boilerplate code.* That sentence is sitting next to guidance that still forbids bodies. Treat it as a breadcrumb, not as syntax.

So for this feature, the source of truth is the [29.0 what's-new row](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/whatsnew/whatsnew-update-29-0), not a reconstructed compiler. Docs can change. This is prerelease documentation. Microsoft says so at the top of the page.

Also the boring-but-load-bearing constraints, from that same article:

- **Preview only.** Business Central 29.0 public preview.
- **Online sandboxes.** Not production. Not on-premises.
- Public preview runs from the first week of September until 29.0 is generally available in **the first week of October 2026**.
- Preview sandboxes get deleted about 30 days after GA.

I am excited. I am not installing this in a production app.

## What I will verify in a 29.0 preview sandbox before I trust it

Before this goes anywhere near an AppSource interface I actually own, I want the compiler, the analyzers, and a second implementor in the same workspace. Not a blog sentence.

Things I will actually check:

- Can I add a method with a default body to a **published** interface without breaking an existing `implements` codeunit?
- What does `RequiredPending` look like in AL — keyword, property, diagnostic, something else? Learn named it. Learn did not show it.
- Which analyzer rules fire on the way from “default exists” to “this is required,” and does **AS0066 still fire** if I add a method the old way?
- Do AL0584 / AL0585 / AL0612 still say interfaces may only contain declarations, or did the language rules move with the what's-new row?
- At runtime, if both the interface default and the implementor provide a body, whose code runs?
- What is the platform / runtime / `application` version floor? An interface that compiles in a 29.0 sandbox is useless if dependents are still on 26.
- Does `extends` still matter, or is it the path for a different kind of change (new capability surface, not a new method on the same surface)?

Until those answers come from a sandbox and from Learn pages that show syntax, this stays in the “language-semantics I am watching” pile — same family as [boolean operators not short-circuiting](/al-does-not-short-circuit-and-that-is-why-my-first-microsoft-pr-exists/). Contracts. What the compiler allows versus what we assume because C# would allow it.

I will be delighted if default interface methods make `Refund` a boring addition. I will be equally fine if preview syntax changes before GA. Either way I want the contract in writing, not in a hope.

Links:

- [What's new or changed in update 29.0 preview](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/whatsnew/whatsnew-update-29-0) — the feature row and the sandbox / GA constraints
- [Interfaces in AL](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al) — still mostly signature-only, still names AS0066
- [Extend interfaces in AL](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al-extend) — the v25 `extends` path
- [AppSourceCop AS0066](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/analyzers/appsourcecop-as0066) — still documented as: do not add methods to a published interface
