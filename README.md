
# hedel-rs

> **A Rust Hierarchical Doubly Linked list**

[![License](https://img.shields.io/badge/licence-GPL3.0-blue)](LICENSE-GPL)

hedel-rs provides all you need to create your own abstraction over a
hierarchical doubly linked list in Rust, suitable choice for a DOM tree.
Based on `Rc`, `Weak`, and a safe wrapper around `UnsafeCell` (`HedelCell`).

If you are new to linked lists, consider reading [Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)

# Ideology

hedel isn't exactly a tree structure.

- `NodeList` is a wrap around its first node. There isn't any root. This allows for
  sibling nodes at the root-level while keeping a different treatment compared to `Node`.
- Given any node in the linked lists you should be able to navigate it all.
- `Node` is simply a wrap on an `Rc` pointer to `HedelCell<NodeInner<T>>`, which contains the actual data in the `content` field.
- Every `Node` has a `child` field which is the first child, allowing you to move vertically.
- This crate provides traits, methods, structs, but we don't provide any fast-way to generate an actual linked list. That's what you are going to build;
  this allows for multiple use cases and better flexibility: do it with macros, functions ...
- Tendency to make everything `pub`, for the previous point.
  
# Features

- `HedelCell`: a cell structure safely relying on UnsafeCell, similar to `RefCell` but smaller in size.
- `Node`/`WeakNode`: to avoid memory-leaking we also provide a weak version of `Node`.
- Identify and collect: create your own identifier implementing the `NodeComparable` trait, iterate over the linked list and collect
  only the nodes matching the identifier.
- Identify and detach: iterate over the linked list and detach only the nodes matching the identifier (move out or remove).
