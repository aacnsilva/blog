+++
date = '2026-09-04T10:00:00+00:00'
draft = false
title = 'AL does not short-circuit — and that is why my first Microsoft PR exists'
+++

I just merged my first contribution into a Microsoft repository: [PR #136](https://github.com/microsoft/BCQuality/pull/136) on [BCQuality](https://github.com/microsoft/BCQuality).

It is not a feature. It is not a new analyzer rule. It is two knowledge articles — with good and bad AL samples — about something most of us assume without checking:

**AL boolean operators do not short-circuit.**

Or, more carefully: the docs never give you a short-circuit guarantee for `and`, `or`, or `xor`, so you must not write code as if they did.

## What BCQuality actually is

[BCQuality](https://github.com/microsoft/BCQuality) is Microsoft’s shared quality bar for Business Central development — for humans and for agents.

It has two halves:

- **Knowledge** — small, atomic facts about how AL and the platform really behave
- **Skills** — how agents should use that knowledge when reviewing or writing code

There are layers: `microsoft/`, `community/`, and `custom/` in your fork. Community content can get promoted once it proves itself. Agents that consume BCQuality (through AL-Go and other orchestrators) already know how to pick up knowledge by domain.

My two articles landed first under `community/knowledge/performance/`. They now live under `microsoft/knowledge/performance/`.

## The assumption that keeps biting us

If you write C#, JavaScript, TypeScript, or SQL, your brain is trained on short-circuit evaluation.

`(A) and (B)` means: if `A` is false, do not evaluate `B`.
`(A) or (B)` means: if `A` is true, do not evaluate `B`.

That is muscle memory. It is also what large language models bring into AL by default, because they were trained mostly on those languages.

AL does not give you that guarantee.

Neither [AL operators](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-al-operators) nor [Boolean operators](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-al-boolean-operators) documents a lazy evaluation order either way. There is also no compiler diagnostic that says “this guard does not guard.” So the bug ships quietly.

### Shape 1: the guard that does not guard

```al
exit((Index >= 1) and (Index <= ArrayLen(Thresholds)) and (Amount > Thresholds[Index]));
```

You think the range check protects the subscript. In AL, `Thresholds[Index]` still gets evaluated. Out of range is still on the table.

### Shape 2: the silent wrong answer

```al
exit((CustomerNo <> '') and Customer.Get(CustomerNo) and (Customer.Blocked <> Customer.Blocked::" "));
```

If `Get` fails, you still read `Blocked` from a record that was never loaded. You do not always get a loud error. You get a **quietly wrong result**. That is worse.

### Shape 3: paying for work you already decided

```al
exit((SalesHeader."Amount Including VAT" >= 1000) or HasActiveLoyaltyBenefit(SalesHeader."Sell-to Customer No."));
```

If the amount alone already qualifies, you still run the loyalty lookup. Correctness is fine. Performance is not. On a hot path, that is the opposite of going brrr.

## The fix for two or three conditions: nest

For dependent `and`, nest:

```al
if (Index >= 1) and (Index <= ArrayLen(Thresholds)) then
    if Amount > Thresholds[Index] then
        exit(true);
exit(false);
```

And for a `Get` then a field:

```al
if CustomerNo = '' then
    exit(false);
if not Customer.Get(CustomerNo) then
    exit(false);
exit(Customer.Blocked <> Customer.Blocked::" ");
```

Important: **do not reuse the nested-`if` rewrite for `or`.**

`A or B` is not the same as `if A then if B`. Nesting drops the case where `A` is true and `B` is false. For `or`, early-exit:

```al
if SalesHeader."Amount Including VAT" >= 1000 then
    exit(true);
exit(HasActiveLoyaltyBenefit(SalesHeader."Sell-to Customer No."));
```

`xor` is its own story. Its result always depends on both sides in every language. Keep both operands cheap.

## The trick that makes long chains go brrr: `case true of` / `case false of`

Nested `if` is fine for two or three guards. Past that you are writing a ladder. The body drifts right. The evaluation order lives only in the indentation. Shared failure paths get copy-pasted at every level.

The wrong escape from that ladder is collapsing everything into one big `and` chain. That trades nesting for a real defect, because every operand still runs.

The flat alternative is AL’s `case` statement.

[AL control statements](https://learn.microsoft.com/en-us/dynamics365/business-central/dev-itpro/developer/devenv-al-control-statements) say a value set “must be an expression or a range,” and that **the first matching value set executes**. That stop-at-first-match behaviour is exactly the laziness `and` / `or` do not give you.

### Guard chain: `case false of`

```al
procedure IsShippableLine(SalesLine: Record "Sales Line"): Boolean
var
    Item: Record Item;
begin
    case false of
        SalesLine.Type = SalesLine.Type::Item,
        SalesLine."No." <> '',
        SalesLine."Qty. to Ship" > 0:
            exit(false);
        Item.Get(SalesLine."No."):
            exit(false);
        not Item.Blocked:
            exit(false);
    end;
    exit(true);
end;
```

`case false of` matches when an expression is false. The first three checks are pure and order-independent, so they share one value set. `Get` and `Blocked` each keep their own value set, in order, because the documented guarantee is across value sets — not inside one comma-separated list.

If you put `Item.Get(...)` and `not Item.Blocked` in the same value set, you are betting on an evaluation order the docs do not promise. That would recreate the original bug with nicer syntax.

### First-match dispatch: `case true of`

```al
case true of
    HasOpenDocument(CustomerNo, "Sales Document Type"::Quote):
        exit('Quote');
    HasOpenDocument(CustomerNo, "Sales Document Type"::Order):
        exit('Order');
    HasOpenDocument(CustomerNo, "Sales Document Type"::Invoice):
        exit('Invoice');
end;
exit('None');
```

Once an earlier probe matches, the later lookups never run. That is how you make a chain of database checks go faster without inventing a new language feature.

Bonus: `case` value sets do not need the parentheses forest that `and` / `or` force on you. In AL, `and` and `or` bind tighter than comparisons, so every operand in a boolean chain needs parentheses. A value set does not.

(I personally like the `in` operator for a lazy-ish feel in some places. For this guidance, nested `if` and `case` stay clearer for most AL developers — and that is what landed in BCQuality.)

## Why agents benefit from this

This is the real reason the PR exists.

An LLM trained mostly on C#, JavaScript, or SQL carries short-circuiting in as a default assumption. Business Central has no compiler diagnostic that contradicts it. So agents happily generate — and approve in review — the silent-wrong `Get and Field` shape.

BCQuality’s admission test is blunt: if a capable model already knew the fact reliably, the knowledge file would not need to exist. This one does.

Putting the articles in `domain: performance` meant the existing `al-performance-review` skill could pick them up across layers with no skill change. Teach the agent the BC-specific fact once. Every review after that inherits it.

That is also why the promotion into the Microsoft layer matters. If a consumer disables the community layer, the Microsoft performance review skills still see the content. The shared bar stays shared.

## Final thought

Next time you — or Copilot, or Claude, or whoever sits next to your AL project — write something like:

```al
Customer.Get(No) and (Customer.Blocked <> ...)
```

stop.

Nest it. Or turn the long chain into `case false of` / `case true of`.

It is a small language detail. It prevents real defects, skips real database work, and is exactly the kind of thing agents get wrong until you put it where they can read it.

Links:

- [PR #136](https://github.com/microsoft/BCQuality/pull/136) — the contribution
- [PR #153](https://github.com/microsoft/BCQuality/pull/153) — promotion into Microsoft performance knowledge
- [boolean-operators-do-not-short-circuit](https://github.com/microsoft/BCQuality/blob/main/microsoft/knowledge/performance/boolean-operators-do-not-short-circuit.md)
- [case-true-of-for-long-condition-chains](https://github.com/microsoft/BCQuality/blob/main/microsoft/knowledge/performance/case-true-of-for-long-condition-chains.md)
