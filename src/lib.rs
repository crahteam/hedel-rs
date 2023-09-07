pub mod node;
pub mod cell;
pub mod errors;
pub mod list;

//! # hedel-rs 
//!
//!**A Hierarchical Doubly Linked List**
//!
//!Hedel-rs provides all you need to create your own abstraction over a
//!hierarchical doubly linked list in Rust, suitable choice for a DOM tree.
//!Designed for when you need a nested generation of nodes. ( e.g with macros ```node!(1, node!(2))``` )
//!Based on `Rc`, `Weak`, and a safe wrapper around `UnsafeCell` (`HedelCell`).
//!
//!If you are new to linked lists, consider reading [Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)
//!
//!# Ideology
//!
//!Hedel isn't exactly a tree structure.
//!
//!- `NodeList` is a wrap around its first node. There isn't any root. This allows for
//! sibling nodes at the root-level.
//!  `NodeList` also dereferences to its firt node letting you call `Node`'s methods.
//!- `Node` is a pointer to its content and other pointers to allow navigation. Those pointers are:
//!  `parent`, `child`, `prev` and `next`, where child is a pointer to its first child.
//!- Support for node generation using macros: you can use node!(1) and nest how many nodes you want.

pub mod prelude {
	pub use crate::node::{
		FindNode,
		GetNode,
		CollectNode,
		DetachNode,
		AppendNode,
		InsertNode,
		CompareNode
	};
}

pub use node::{
	Node,
	WeakNode,
	NodeCollection,
	Content
};

pub use list::{
	NodeList
};
