 ![alt text](https://media.discordapp.net/attachments/1147987451663614012/1147989627194576906/hed.png)
 
# A Hierarchical Doubly Linked list Rust library
hedel-rs provides structs, traits, methods to create your own abstraction over a hierarchical doubly linked list
in Rust. 

# Features
- HedelCell: a RefCell-like structure but smaller, safely relying on UnsafeCell.
- Node/WeakNode: linked lists are made of nodes pointing to each other. In this case
``` rust
 pub struct Node<T> {
   inner: Rc<HedelCell<NodeInner<T>>> 
 }

 pub struct NodeInner<T> {
   parent: Option<WeakNode<T>>,
   child: Option<Node<T>>,
   next: Option<Node<T>>,
   prev: Option<WeakNode<T>>,
   content: T
 }
```
child is pointing to the first child, and content is a custom field to let you own whatever you want.
