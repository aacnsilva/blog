+++
date = '2026-09-18T10:30:00+01:00'
draft = false
title = 'How AL interfaces might finally grow without a version bump'
+++

I have an `IPaymentGateway` with `Authorize` and `Capture`. It is published. Partners implemented it. Then a customer asks for `Refund`.

**That is the moment a published AL interface stops being a contract and starts being a fossil.**

I am reading this from the [29.0 public preview notes](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/whatsnew/whatsnew-update-29-0), not from a sandbox compile. Here is what Microsoft said, and what that would mean for a published ISV-style interface. `IPaymentGateway` below is just an example.

## The ISV story that never gets less true

You ship a payment add-in. The interesting bit is not your codeunit. It is the interface other apps implement so they can plug in their own gateway:

```al
interface "IPaymentGateway"
{
    procedure Authorize(var Request: Record "Payment Request"): Boolean
    procedure Capture(var Request: Record "Payment Request"): Boolean
}
```

That shape is the documented AL interface: a name, some signatures, no body. [Interfaces in AL](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al) still describes that model, and implementors use `implements`.

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

<figure class="theme-diagram">
  <img class="theme-diagram__light" src="/images/al-interface-evolution-bc29-preview.svg" width="760" height="560" alt="Four-step timeline: publish an AL interface, need a new method, the old pain of breaking or extending, then the BC 29.0 preview of default method bodies plus RequiredPending and analyzer rules.">
  <img class="theme-diagram__dark" src="/images/al-interface-evolution-bc29-preview-dark.svg" width="760" height="560" alt="Four-step timeline: publish an AL interface, need a new method, the old pain of breaking or extending, then the BC 29.0 preview of default method bodies plus RequiredPending and analyzer rules.">
</figure>

## What Microsoft announced for BC 29.0

Then the 29.0 public preview what's-new table grew a development row I have wanted for years:

**Evolve AL interfaces with default implementations** — *Add default method bodies to AL interfaces so existing implementors continue working, then use RequiredPending and analyzer rules to introduce required methods safely.*

That is the understanding I am working from. I am not going to invent a procedure-body syntax, a property name beyond `RequiredPending`, or a sample that pretends to compile. Syntax samples were still thin on Learn when I wrote this, which I expect will fill in toward GA.

Read that sentence the ISV way, and the `Refund` story changes shape:

1. You add `Refund` with a default body, so the partner codeunit that only did `Authorize` / `Capture` keeps compiling.
2. You use `RequiredPending` and analyzer rules to move that method from “there is a default” toward “you really should implement this.”
3. Existing implementors continue working while you do it.

If that lands as described, published interfaces can grow **in place**. No v2. No `extends` just to add one procedure. No “sorry, bump your dependency.”

This is public preview, not GA. Preview notes can still change — for better or worse — and that is a normal part of the run-up to October.

## Where the docs live today

The [Interfaces in AL](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al) page still mostly describes today's signature-only model. It still tells you to avoid adding methods to published interfaces, and it still names AS0066. It still says interfaces can only contain procedure declarations (AL0584, AL0585, AL0612). That is the contract we have been shipping against.

Easy to mix up: that page also says *consider using default implementations for methods in interfaces to reduce boilerplate code.* In AL today, that is not a method body on the interface. It is the [DefaultImplementation](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/properties/devenv-defaultimplementation-property) property on an **enum** that implements the interface — a fallback codeunit when a value has no `Implementation`. Same family as `Implementation = IAddressProvider = CompanyAddressProvider` in the Learn sample. AppSourceCop [AS0067](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/analyzers/appsourcecop-as0067) even requires a default when you add an interface to a published extensible enum, so enum extensions keep compiling.

```al
enum 50135 SomeEnum implements IFoo
{
    Extensible = true;
    DefaultImplementation = IFoo = DefaultFooImpl;

    value(0; Yes)
    {
        Implementation = IFoo = YesFooImpl;
    }
    value(1; No)
    {
        // uses DefaultFooImpl
    }
}
```

Useful. Different story. The 29.0 [what's-new row](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/whatsnew/whatsnew-update-29-0) is about default **method bodies on the interface itself**. I expect Learn to grow as GA approaches — more syntax, more analyzer detail.

The same what's-new article is also clear about the preview window:

- **Preview only.** Business Central 29.0 public preview.
- **Online sandboxes.** Not production. Not on-premises.
- Public preview runs from the first week of September until 29.0 is generally available in **the first week of October 2026**.
- Preview sandboxes get deleted about 30 days after GA.

I am excited. I am not installing this in a production app yet, because it is not GA yet.

## What I want to try in a 29.0 preview sandbox

Before this goes anywhere near an AppSource interface I actually own, I want the compiler, the analyzers, and a second implementor in the same workspace. Normal due diligence for a preview language feature — the same instinct as [boolean operators not short-circuiting](/al-does-not-short-circuit-and-that-is-why-my-first-microsoft-pr-exists/). Check the contract. Then enjoy it.

Things I will actually try:

- Can I add a method with a default body to a **published** interface without breaking an existing `implements` codeunit?
- What does `RequiredPending` look like in AL — keyword, property, diagnostic, something else? Learn named it. I have not seen a public syntax sample yet, which is normal at this stage of preview.
- Which analyzer rules fire on the way from “default exists” to “this is required,” and does **AS0066 still fire** if I add a method the old way?
- Do AL0584 / AL0585 / AL0612 still say interfaces may only contain declarations, or did the language rules move with the what's-new row?
- At runtime, if both the interface default and the implementor provide a body, whose code runs?
- What is the platform / runtime / `application` version floor? An interface that compiles in a 29.0 sandbox needs a clear story for dependents still on 26.
- Does `extends` still matter, or is it the path for a different kind of change (new capability surface, not a new method on the same surface)?

I will be delighted if default interface methods make `Refund` a boring addition. If the preview syntax shifts a little before GA, that is fine too. I just want to see it compile once, then write the partner mail.

Links:

- [What's new or changed in update 29.0 preview](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/whatsnew/whatsnew-update-29-0) — the feature row and the sandbox / GA constraints
- [Interfaces in AL](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al) — still mostly signature-only, still names AS0066
- [Extend interfaces in AL](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-interfaces-in-al-extend) — the v25 `extends` path
- [AppSourceCop AS0066](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/analyzers/appsourcecop-as0066) — still documented as: do not add methods to a published interface
- [DefaultImplementation property](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/properties/devenv-defaultimplementation-property) — enum fallback codeunit, not a method body on the interface
