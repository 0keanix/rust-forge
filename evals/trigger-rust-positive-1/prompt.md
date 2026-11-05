---
tags: [trigger]
max_turns: 10
allowed_tools: [Read, Glob, Grep, Skill]
---

I'm getting this and I don't get why:

```
error[E0502]: cannot borrow `scores` as mutable because it is also borrowed as immutable
 --> src/main.rs:6:5
  |
5 |     let best = scores.iter().max().unwrap();
  |                ------ immutable borrow occurs here
6 |     scores.push(*best + 1);
  |     ^^^^^^^^^^^^-----^^^^^
  |     |           |
  |     |           immutable borrow later used here
  |     mutable borrow occurs here
```

How do I fix it properly?
