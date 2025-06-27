---
title: What are events in Business Central
date: 2020-03-29
---

Events were introduced in **Dynamics NAV 2016**.

But you’re asking — what *are* events?

In short, **Events in Dynamics NAV/Business Central implement a Design Pattern known as the “Observer Pattern.”**

---

## Explanation

In Dynamics NAV/Business Central, you can create new functions and make them **Events**.
An Event can be of type **Subscriber** or **Publisher**.

### Publisher Event Types:

* **Integration**
* **Business**

> *(You may also see `Global` and `Trigger` types, but you can only define the first two)*

The difference?

* **Business Events**: Microsoft guarantees they won’t be removed in future versions.
* **Integration Events**: These *might* be removed if deemed not useful by Microsoft.

## What’s the Difference Between a Publisher and a Subscriber?

* A **Publisher** knows its **Subscribers**.
* A **Publisher** stays alert to a certain condition and, when met, it *calls* the subscriber to act.

> Think of the **Subscriber** as the *response* to a given situation monitored by the **Publisher**.

* One **Publisher** can have many **Subscribers**.
* One **Subscriber** can only have one **Publisher**.

---

## Built-in Events in NAV/Business Central

All table and page triggers like:

* `Insert`
* `Modify`
* `Delete`
* `Validate`

...have **two default publishers**:

* `OnBeforeInsertEvent`
* `OnAfterInsertEvent`

### Note:

Since a **Publisher** can have multiple **Subscribers**, **execution order is not guaranteed**.

So, if you want `sub1` to run *before* `sub2`, you need to:

* Remove `sub2`
* Combine its logic inside `sub1`

---

## Practical Example

You might start with:

```al
// Sub1 and Sub2 both subscribing to the same event
[EventSubscriber(...)]
procedure Sub1(...)

[EventSubscriber(...)]
procedure Sub2(...)
```

After applying the workaround:

```al
// Only Sub1 remains, containing logic from both
[EventSubscriber(...)]
procedure Sub1(...)
    // Logic for Sub1
    // Logic for Sub2
```

---

## The Observer Pattern

Let’s compare it to the Observer Pattern:

### Class Diagram Focus:

![class diagram](https://aacnsilva.wordpress.com/wp-content/uploads/2020/03/image-4.png)

#### Subject:

* Holds a list of Observers (`observerCollection`)
* Functions:

  * `registerObserver` → Adds an observer
  * `unregisterObserver` → Removes an observer
  * `notifyObservers` → Invokes all observers

#### Observer:

* Has one method: `update` (the response to the subject)

### Sequence Diagram

![sequence diagram](https://aacnsilva.wordpress.com/wp-content/uploads/2020/03/image-5.png)

1. `Subject s1` adds `Observer o1` and `Observer o2` (attach/register).
2. The monitored condition occurs (notifyObservers).
3. `Observer o1` is invoked.
4. Then `Observer o2`.

---

## Mapping to NAV/BC

| Observer Pattern | Dynamics NAV/BC |
| ---------------- | --------------- |
| Subject          | Publisher       |
| Observer         | Subscriber      |

## TIP

You can check subscribers in **NAV/Business Central** via the **Event Subscriptions** page.

Example link (BC14 Web Client):

```
http://bc14/NAV/?company=CRONUS%20International%20Ltd.&page=9510
```

---
