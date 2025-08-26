+++
date = '2025-06-27T21:42:21+01:00'
draft = false
title = 'Keyboard Shortcuts in Business Central'
+++

---
title: Keyboard Shortcuts in Business Central
date: 2025-06-27
---

Ever wondered why some actions and caption on pages have an & sign?
We'll, you guessed it, that is used to add keyboard shortcuts for actions, or better worded, configure an access key!

## Example

```al
action("&foo")
{
  Caption = '&Foo';
}
```

By pressing Alt  + F you'll be able to trigger the foo action.
In generic terms you get this: `access modifier` + `access key` to trigger an action.

**Note:**
Ensure that no two actions or captions in the same context use the same access key, as this can cause conflicts.
