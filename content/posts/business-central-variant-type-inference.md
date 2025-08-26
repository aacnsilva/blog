+++
date = '2025-06-27T21:52:21+01:00'
draft = false
title = 'Variant Type Inference in Business Central'
+++

What happens when you use the Variant data type in Business Central and assign different values?

## Quiz

1. What type is inferred into the Variant variable when assigned a value of 'Variant'?

```al
var
  v: Variant;
begin
  v := 'Variant';
end;
```

  a) `IsText = true`

  b) `IsChar = true`

  c) `IsCode = true`

1. What type is inferred into the Variant variable when assigned a value of 'CODE'?

```al
var
  v: Variant;
begin
  v := 'CODE';
end;
```

  a) `IsText = true`

  b) `IsChar = true`

  c) `IsCode = true`

1. What type is inferred into the Variant variable when assigned a value of '1'?

```al
var
  v: Variant;
begin
  v := '1';
end;
```

  a) `IsText = true`

  b) `IsChar = true`

  c) `IsCode = true`

## Answers

1. a)
1. a)
1. c)

It is good to understand that in text based types - Char, Code and Text - the only possible inference will be Char and Text, because Code follows a different rule set that cannot be ensured by just providing a string literal to a variable, unlike Char and Text which only differ in size.
