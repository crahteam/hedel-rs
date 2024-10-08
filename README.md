
# hedel-rs 
[![License](https://img.shields.io/badge/licence-GPL3.0-blue)](LICENSE-GPL)   [![Latest Version](https://img.shields.io/badge/crates.io-v0.1.3-yellow)](https://crates.io/crates/hedel-rs)   [![Documentation](https://img.shields.io/badge/docs.rs-hedel--rs-red)](https://docs.rs/hedel-rs)

**A Hierarchical Doubly Linked List**

Hedel-rs provides all you need to create your own abstraction over a
hierarchical doubly linked list in Rust, suitable choice for a DOM tree.
Designed for when you need a nested generation of nodes. ( e.g with macros ```node!(1, node!(2))``` )
Based on `Rc`, `Weak`, and a safe wrapper around `UnsafeCell` (`HedelCell`).

If you are new to linked lists, consider reading [Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)

# Ideology

Hedel isn't exactly a tree structure.

- `NodeList` is a wrap around its first node. There isn't any root. This allows for
  sibling nodes at the root-level.
  `NodeList` also dereferences to its firt node letting you call `Node`'s methods.
- `Node` is a pointer to its content and other pointers to allow navigation. Those pointers are:
  `parent`, `child`, `prev` and `next`, where child is a pointer to its first child.
- Support for node generation using macros: you can use node!(1) and nest how many nodes you want.

# Features

- `HedelCell`: a cell structure safely relying on UnsafeCell, similar to `RefCell` but smaller in size.
- `Node`/`WeakNode`: to avoid memory-leaking we also provide a weak version of `Node`.
- Macros: generate nodes blazingly fast with node!() and list!()
  
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
  
- Identify and compare: create your own identifier implementing the `CompareNode` trait.

  ```rust
  pub enum NumIdent {
        Equal(i32),
        BiggerThan(i32),
        SmallerThan(i32)
  }
  
  impl CompareNode<i32> for NumIdent {
      fn compare(&self, node: &Node<i32>) -> bool {
          match &self {
            Equal(n) => {
                  as_content!(node, |content| {
                    return content == &n;
                  });
              },
            BiggerThan(n) => {
              as_content!(node, |content| {
                return content > &n;
              });
            },
            SmallerThan(n) => {
              as_content!(node, |content| {
                  return content < &n;
              });
            }
        }
    }
  }
  
  fn main() {
    let node = node!(3);
    assert!(NumIdent::BiggerThan(2).compare(&node));
  }  
  ```
- Collect: iterate over the linked list and collect
  only the nodes matching the identifier.
  ```rust
  let node = node!(1,
    node!(2),
    node!(3),
    node!(4),
    node!(5)
  );
  
  let collection = node.collect_children(&NumIdent::BiggerThan(3));
  
  for node in collection.into_iter() {
    println!("{}" node.to_content());
  }
  ```
  
- Detach: detach the nodes matching an identifier in the linked list.
  ```rust
  let node = node!(1
    node!(2),
    node!(3),
    node!(4),
    node!(5)
  );

  let three = node.find_child(&NumIdent::Equal(3)).unwrap();
  three.detach();

  assert_eq!(node.find_child(&NumIdent::Equal(3)), None);
  ```
- Insert or Append: insert a node at any position in a linked list.
  ```rust
  let node = node!(1,
    node!(3),
    node!(4),
    node!(5)
  );

  node.insert_child(0, node!(2));

  assert_eq!(node.child().unwrap().to_content(), 2);

  node.append_child(node!(6));

  assert_eq!(node.get_last_child().unwrap().to_content(), 6);
  ```
