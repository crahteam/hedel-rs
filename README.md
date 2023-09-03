 ![alt text](https://media.discordapp.net/attachments/1147987451663614012/1147989627194576906/hed.png)
 
# A Rust Hierarchical Doubly Linked list
hedel-rs provides structs, traits, methods to create your own abstraction over a hierarchical doubly linked list
in Rust. 

# Features
- HedelCell: a RefCell-like structure but smaller, safely relying on UnsafeCell.
- Node/WeakNode: linked lists are made of nodes pointing to each other. In this case the
  child field is pointing to the first child, and content is a custom field to let you own whatever you want.
``` rust
 pub struct Node<T> {
   pub inner: Rc<HedelCell<NodeInner<T>>> 
 }

 pub struct WeakNode<T> {
   pub inner: Weak<HedelCell<NodeInner<T>>> 
 }

 pub struct NodeInner<T> {
   pub parent: Option<WeakNode<T>>,
   pub child: Option<Node<T>>,
   pub next: Option<Node<T>>,
   pub prev: Option<WeakNode<T>>,
   pub content: T
 }
```
- NodeList: simply represents its first child. necessary because treated differently.
- NodeCollection: a vector of Nodes that can be retrived using methods.
