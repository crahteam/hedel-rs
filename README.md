
# hedel-rs

> **A Rust Hierarchical Doubly Linked List**

[![License](https://img.shields.io/badge/licence-GPL3.0-blue)](LICENSE-GPL)

hedel-rs provides all you need to create your own abstraction over a
hierarchical doubly linked list in Rust, suitable choice for a DOM tree.
Designed for when you need a nested generation of nodes. ( e.g with macros ```node!(1, node!(2))``` )
Based on `Rc`, `Weak`, and a safe wrapper around `UnsafeCell` (`HedelCell`).

If you are new to linked lists, consider reading [Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)

# Ideology

hedel isn't exactly a tree structure.

- `NodeList` is a wrap around its first node. There isn't any root. This allows for
  sibling nodes at the root-level while keeping a different treatment compared to `Node`.
- Given any node in the linked lists you should be able to navigate it all.
- `Node` is simply a wrap on an `Rc` pointer to `HedelCell<NodeInner<T>>`, which contains the actual data in the `content` field.
- Every `Node` has a `child` field which is the first child, allowing you to move vertically.
- Support for node generation by defining the inner nodes first and the outer later.
  This means you can use the node!() macro and nest as many nodes as you want.

# Features

- `HedelCell`: a cell structure safely relying on UnsafeCell, similar to `RefCell` but smaller in size.
- `Node`/`WeakNode`: to avoid memory-leaking we also provide a weak version of `Node`.
- Identify and collect: create your own identifier implementing the `NodeComparable` trait, iterate over the linked list and collect
  only the nodes matching the identifier.
- Identify and detach: iterate over the linked list and detach only the nodes matching the identifier (move out or remove).
- Macros: generate nodes blazingly fast with node! and list!
  ```rust
  let node = node!(45);
  let my_node = node!("Parent",
    node!("Child"),
    node!("Child")
  );

  let my_list = list!(
    node!(2),
    node!(3)
  );
  ```
