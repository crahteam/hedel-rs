use std::{
	rc::{
		Weak,
		Rc
	}
};

use crate::cell::{
	HedelCell,
	RefHedel,
	RefMutHedel,
};

use crate::errors::HedelError;

/// NodeInner contains pointers in both vertical and horizontal directions
/// and a custom content field.
pub struct NodeInner<T> {
	pub next: Option<Node<T>>,
	pub prev: Option<WeakNode<T>>,
	pub child: Option<Node<T>>,
	pub parent: Option<WeakNode<T>>,
	pub content: T
}

/// `Rc` is a strong pointer meaning it increment a reference counter.
/// `Weak` is a weak pointer meaning it doesn't increment the reference counter,
/// letting you access the value if it still exists in memory,
/// modify it as its pointing to `HedelCell`,
/// but without holding it in memory any longer.
/// Necessary to avoid memory leaking.
pub struct WeakNode<T> {
	pub inner: Weak<HedelCell<NodeInner<T>>>
}

impl<T> WeakNode<T> {
	/// upgrade `WeakNode` to `Node` if the `NodeInner` is still alive.
	pub fn upgrade(&self) -> Option<Node<T>> {
		Some(Node::<T> {
			inner: self.inner.upgrade()?
		})
	}
}

/// Wraps the inner value with an Rc<HedelCell<_>> pointer.
/// allowing for multiple owners and a mutable `NodeInner`
pub struct Node<T> {
	pub inner: Rc<HedelCell<NodeInner<T>>>,
}

impl<T> Clone for Node<T> {
	fn clone(&self) -> Self {
		Self {
			inner: Rc::clone(&self.inner),
		}
	}
}

impl<T> Node<T> {
	/// Default constructor. Notice how it builds a stand-alone node,
	/// not pointing to any parent, any sibling and any child,
	/// but owning the content
	pub fn new(content: T) -> Self {
		Self {
			inner: Rc::new(HedelCell::new(NodeInner::<T> {
				next: None,
				prev: None,
				child: None,
				parent: None,
				content
			})),
		}
	}

	/// A `WeakNode` has to be built by downgrading `Node`
	/// following the same logic to get a `Weak` from a `Rc`
	pub fn downgrade(&self) -> WeakNode<T> {
		WeakNode {
			inner: Rc::downgrade(&self.inner)
		}
	}

	/// Get access to `NodeInner` or return `HedelError` in case 
	/// the runtime borrow checker in `HedelCell` doesn't allow to get a shared reference.
	pub fn try_get(&self) -> Result<RefHedel<NodeInner<T>>, HedelError> {
		Ok(self.inner.try_get()?)
	}

	/// Get access to `NodeInner` or panic! in case 
	/// the runtime borrow checker in `HedelCell` doesn't allow to get a shared reference.
	pub fn get(&self) -> RefHedel<NodeInner<T>> {
		self.inner.get()
	}

	/// Get mutable access to `NodeInner` or return `HedelError` in case 
	/// the runtime borrow checker in `HedelCell` doesn't allow to get a mutable reference.
	pub fn try_get_mut(&self) -> RefMutHedel<'_, NodeInner<T>> {
		self.inner.get_mut()
	}

	/// Get mutable access to `NodeInner` or panic! in case 
	/// the runtime borrow checker in `HedelCell` doesn't allow to get a mutable reference.
	pub fn get_mut(&self) -> RefMutHedel<'_, NodeInner<T>> {
		self.inner.get_mut()
	}

	/// Get the next `Node` in horizontal direction
	pub fn next(&self) -> Option<Node<T>> {
		self.get().next.clone()	
	}

	/// Get the previous `Node` in horizontal direction by upgrading it.
	pub fn prev(&self) -> Option<Node<T>> {
		if let Some(ref p) = self.get().prev {
			return p.upgrade()
		} None
	}

	/// Get the parent `Node` in vertical direction by upgrading it.
	pub fn parent(&self) -> Option<Node<T>> {
		if let Some(ref p) = self.get().parent {
			return p.upgrade();
		} None
	}

	/// Get the first child `Node` in vertical direction.
	pub fn child(&self) -> Option<Node<T>> {
		self.get().child.clone()
	}

	/// Re-set the `parent`, `next` and `prev` fields on the `Node`.
	/// WARNING: this is meant to be used by `NodeCollection::free` after 
	/// the `HedelDetach::detach_preserve` function. Refer to it's documentation
	/// for an usage example. 
	///
	/// If you want to detach a single Node while iterating, most of the times
	/// you can simply break the loop and use `HedelDetach::detach`.
	/// WARNING: using this function instead of `HedelDetach::detach` 
	/// might break the linked list.
	pub fn free(&self) {
		let mut node = self.get_mut();
		node.parent = None;
		node.next = None;
		node.prev = None;
	}
}

pub trait HedelDetach<T> {
	/// Detaches a single node from the linked list by fixing the pointers between the 
	/// parent, the previous and next siblings. This also detaches all the children of the `Node`,
	/// which will only remain linked with the node itself.
	/// WARNING: This also re-sets the pointers in the node itself to None. 
	/// So when you are detecting nodes in a linked-list and detaching them, you cant iterate over them using this method
	/// as it would break the loop. Use `detach_preserve` instead.
	fn detach(&self);
	/// Detaches a single node from the linked list like `detach`, but doesn't re-set the pointers inside the Node.
	/// This should only be used when you have to iterate over a linked list and detach some `Node`s.
	/// You should create a vector to store the detached nodes, and iterate over them only when the while loop is 
	/// compleated, re-setting the `parent`, `prev`, `next` fields to `None`.
	///
	/// # Example
	///
	/// let mut detached_nodes: NodeCollection<T> = NodeCollection::<T>::new();
	///
	/// if let Some(next) = my_node.next() {
	///
	///		let mut next = next;
	///
	///		/* do */ {
	///
	///			if ident.compare(&next) { next.detach_preserve(&mut detached_nodes); }
	///
	///		} while let Some(n) = next.next() {
	///
	///			next = n;
	///
	///			if ident.compare(&next) { next.detach_preserve(&mut detached_nodes); }
	///
	///		}
	///	}
	/// // this will re-set all the `parent`, `next`, `previous` pointers in every Node. 
	/// detached_nodes.free();
	///
	fn detach_preserve(&self, vec: &mut NodeCollection<T>);
}

/// `NodeCollection` represents a `Vec` of `Node`s. Usually retrived by collecting over
/// a `Node` linked list using the `HedelCollect` trait implementation.
/// WARNING: this is not a linked list, but simply a collection of unrelated nodes.
/// The contained nodes might come from separated linked lists or from the same one.
pub struct NodeCollection<T> {
	pub nodes: Vec<Node<T>>
}

impl<T> NodeCollection<T> {
	
	/// Builds a new collection with the vector provided.
	pub fn new(nodes: Vec<Node<T>>) -> Self {
		Self {
			nodes
		}
	}

	/// Consume `self` and retrive its `Node`s.
	pub fn into_nodes(self) -> Vec<Node<T>> {
		self.nodes
	}

	/// Retrive a reference to the `Node`s.
	pub fn as_nodes(&self) -> &Vec<Node<T>> {
		&self.nodes
	}

	/// Retrive a mutable reference to the `Node`s.
	pub fn as_mut_nodes(&mut self) -> &mut Vec<Node<T>> {
		&mut self.nodes
	}

	/// Re-set the `parent`, `prev` and `next` pointers in every node of the collection.
	/// This function is commonly used when iterating over a linked list detaching the
	/// nodes satisfying an identifier using `HedelDetach::detach_preserve`.
	/// refer to `HedelDetach::detach_preserve` for a code example.
	///
	/// WARNING: don't use this function to detach a node from a linked list.
	pub fn free(&self) {
		for node in self.nodes.iter() {
			node.free();
		}
	}

}

impl<T> IntoIterator for NodeCollection<T> {
	type Item = Node<T>;
	type IntoIter = std::vec::IntoIter<Node<T>>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}

}

/// Users are supposed to impl `NodeComparable` for an enum they would
/// like to use as an identifier.
///
/// # Example
///
/// pub enum NumIdent {
///		BiggerThan(i32),
///		SmallerThan(i32)
/// }
///
/// impl NodeComparable<i32> for NumIdent {
/// 	fn compare(&self, node: &Node<i32>) -> bool {
///			match &self {
///				NumIdent::BiggerThan(num) => {
///					node.get() > num
///				},
///				NumIdent::SmallerThan(num) => {
///					node.get() < num
///				}
///			}
///		}
/// }
///
pub trait NodeComparable<T> {
	fn compare(&self, node: &Node<T>) -> bool;
}

pub trait HedelCollect<P, T: NodeComparable<P>> {
	/// Given an identifier of type implementing `NodeComparable` this iterates over all the nodes
	/// in the linked list horizontally ( iterates over the siblings, previous and next ),
	/// and compare every node. The nodes satisfying the identifier get collected into a `NodeCollection`.
	fn collect_siblings(&self, ident: &T) -> NodeCollection<P>;
	/// Given an identifier of type implementing `NodeComparable` this iterates over all the nodes in the 
	/// linked list both horizontally and vertically ( iterates horizontally in each hierarchical level,
	/// up to the top parent and down to the deepest child also
	/// iterating vertically and horizontally for each layer of the children ).
	/// The nodes satisfying the identifier get collected into a `NodeCollection`.
	fn  collect_linked_list(&self, ident: &T) -> NodeCollection<P>;
	/// Given an identifier of type implementing `NodeComparable` this iterates over all the nodes that stand 
	/// lower and deeper in the linked list. Every child satysfying the identifier get collected into a `NodeCollection`
	fn collect_children(&self, ident: &T) -> NodeCollection<P>;
}                                                         

impl<P, T: NodeComparable<P>> HedelCollect<P, T> for Node<P> {

	fn collect_siblings(&self, ident: &T) -> NodeCollection<P> {
	
		let mut collection = Vec::new();
		
		if ident.compare(&self) {
			collection.push(self.clone());
		}

		// search in the previous nodes before
		// search in the next nodes after 

		let mut current;

		if let Some(prev) = self.prev() {

			/* do */ {

				current = prev;

				if ident.compare(&current) {
					collection.push(current.clone());
				}

			} while let Some(prev) = current.prev() {

				current = prev;

				if ident.compare(&current) {
					collection.push(current.clone());
				}
			}
		}

		if let Some(next) = self.next() {

			/* do */ {

				current = next;

				if ident.compare(&current) {
					collection.push(current.clone());
				}

			} while let Some(next) = current.next() {

				current = next;

				if ident.compare(&current) {
					collection.push(current.clone());
				}
			}
		}

		NodeCollection::<P> {
			nodes: collection
		}
	}

	fn collect_children(&self, ident: &T) -> NodeCollection<P> {

		let mut collection = Vec::new();

		if let Some(child) = self.child() {

			let mut child = child;

			while let Some(c) = child.child() {

				// we reached a new depth-level in hierarchy

				child = c;

				if ident.compare(&child) {
					collection.push(child.clone());
				}

				// iterates horizontally in the previous siblings
				
				if let Some(prev) = child.prev() {
					let mut prev = prev;

					/* do */ {

						if ident.compare(&prev) {
							collection.push(prev.clone());
						}

						collection.extend(prev.collect_children(ident).nodes);

					} while let Some(p) = prev.prev() {
						
						prev = p;

						if ident.compare(&prev) {
							collection.push(prev.clone());
						}

						collection.extend(prev.collect_children(ident).nodes);
					}
				}

				// iterates horizontally in the next siblings

				if let Some(n) = child.next() {

					let mut next = n;

					/* do */ {

						if ident.compare(&next) {
							collection.push(next.clone());
						}

						collection.extend(next.collect_children(ident).nodes);

					} while let Some(n) = next.next() {

						next = n;

						if ident.compare(&next) {
							collection.push(next.clone());
						}

						collection.extend(next.collect_children(ident).nodes);
					}
				}
			}
		}

		NodeCollection::<P> {
			nodes: collection
		}
	}

	fn collect_linked_list(&self, ident: &T) -> NodeCollection<P> {
		
		let mut collection = Vec::new();
		
		// collect on the current level
		// collect on the upper levels
		// collect on the inner levels
	
		if let Some(parent) = self.parent() {
			let mut parent = parent;
			
			while let Some(p) = parent.parent() {
				parent = p;
			}

			// we obtained the top parent node

			if ident.compare(&parent) {
				collection.push(parent.clone());
			}

			collection.extend(parent.collect_children(ident).nodes);
			
			// does the same thing on all the other next top parent nodes

			if let Some(n) = parent.prev() {
				let mut prev = n;

				/* do */ {

					if ident.compare(&prev) {
						collection.push(prev.clone());
					}

					collection.extend(prev.collect_children(ident).nodes);

				} while let Some(n) = prev.prev() {
					prev = n;

					if ident.compare(&prev) {
						collection.push(prev.clone());
					}

					collection.extend(prev.collect_children(ident).nodes);
				}
			}

			if let Some(n) = parent.next() {
				let mut next = n;

				/* do */ {

					if ident.compare(&next) {
						collection.push(next.clone());
					}

					collection.extend(next.collect_children(ident).nodes);

				} while let Some(n) = next.next() {
					next = n;

					if ident.compare(&next) {
						collection.push(next.clone());
					}

					collection.extend(next.collect_children(ident).nodes);
				}
			}
		} else {
			// in case we dont have a parent
			// iterates in the previous siblings
			// iterates in the next siblings

			if ident.compare(&self) {
				collection.push(self.clone());
			}

			collection.extend(self.collect_children(ident).nodes);
	
			if let Some(n) = self.prev() {
				let mut prev = n;

				/* do */ {

					if ident.compare(&prev) {
						collection.push(prev.clone());
					}

					collection.extend(prev.collect_children(ident).nodes);

				} while let Some(n) = prev.prev() {
					prev = n;

					if ident.compare(&prev) {
						collection.push(prev.clone());
					}

					collection.extend(prev.collect_children(ident).nodes);
				}
			}

			if let Some(n) = self.next() {
				let mut next = n;

				/* do */ {

					if ident.compare(&next) {
						collection.push(next.clone());
					}

					collection.extend(next.collect_children(ident).nodes);

				} while let Some(n) = next.next() {
					next = n;

					if ident.compare(&next) {
						collection.push(next.clone());
					}

					collection.extend(next.collect_children(ident).nodes);
				}
			}
		}

		NodeCollection::<P> {
			nodes: collection
		}
	}
} 

pub trait HedelFind<P, T: NodeComparable<P>> {
	/// Get the first `Node` in the linked list, at the same depth-level of `&self` and coming after it,
	/// matching the identifier.
	/// This guarantees to actually retrive the closest `Node`.
	fn find_next(&self, ident: &T) -> Option<Node<P>>;
	/// Get the first `Node` in the linked list, at the same depth-level of `&self` and coming before it,
	/// matching the identifier.
	/// This guarantees to actually retrive the closest `Node`.
	fn find_prev(&self, ident: &T) -> Option<Node<P>>;
	/// Get the first child `Node` of `&self` in the linked list matching the identifier. 
	/// WARNING: it's not guaranteed to retrive the closest `Node`. Only use when you don't
	/// care about which node is retrived as long as it matches the identifier or when you are 100% sure
	/// that there isn't more than one `Node` satisfying the identifier in the children.
	fn find_child(&self, ident: &T) -> Option<Node<P>>;
	/// Get a `Node` somewhere in the linked list matching the identifier.
	/// WARNING: it's not guaranteed to retrive the closest `Node`. Only use when you don't
	/// care about which node is retrived as long as it matches the identifier or when you are 100% sure
	/// that there isn't more than one `Node` satisfying the identifier in the linked list.
	fn find_linked_list(&self, ident: &T) -> Option<Node<P>>;
}                                                         

impl<P, T: NodeComparable<P>> HedelFind<P, T> for Node<P> {
	
	fn find_next(&self, ident: &T) -> Option<Node<P>> {
		if let Some(next) = self.next() {
			let mut next = next;

			/* do */ {

				if ident.compare(&next) {
					return Some(next);
				}
				
				if let Some(c) = next.find_child(ident) {
					return Some(c);
				}

			} while let Some(n) = next.next() {
				next = n;

				if ident.compare(&next) {
					return Some(next);
				}
	
				if let Some(c) = next.find_child(ident) {
					return Some(c);
				}
			}
		}
	
		None
	}

	fn find_prev(&self, ident: &T) -> Option<Node<P>> {
		if let Some(prev) = self.prev() {
			let mut prev = prev;

			/* do */ {

				if ident.compare(&prev) {
					return Some(prev);
				}
				
				if let Some(c) = prev.find_child(ident) {
					return Some(c);
				}

			} while let Some(n) = prev.prev() {
				prev = n;

				if ident.compare(&prev) {
					return Some(prev);
				}
	
				if let Some(c) = prev.find_child(ident) {
					return Some(c);
				}
			}
		}
		None

	}
	
	fn find_linked_list(&self, ident: &T) -> Option<Node<P>> {
		if let 	Some(parent) = self.parent() {
			let mut parent = parent;
			
			while let Some(p) = parent.parent() {
				parent = p;
			}

			// we obtained the top parent node

			if ident.compare(&parent) {
				return Some(parent);
			}

			if let Some(c) = parent.find_child(ident) {
				return Some(c);
			}
			
			// does the same thing on all the other next top parent nodes

			if let Some(n) = parent.prev() {
				let mut prev = n;

				/* do */ {

					if ident.compare(&prev) {
						return Some(prev);
					}

					if let Some(c) = prev.find_child(ident) {
						return Some(c);
					}

				} while let Some(n) = prev.prev() {
					prev = n;

					if ident.compare(&prev) {
						return Some(prev);
					}

					if let Some(c) = prev.find_child(ident) {
						return Some(c);
					}
				}
			}

			if let Some(n) = parent.next() {
				let mut next = n;

				/* do */ {

					if ident.compare(&next) {
						return Some(next);
					}

					if let Some(c) = next.find_child(ident) {
						return Some(c);
					}

				} while let Some(n) = next.next() {
					next = n;

					if ident.compare(&next) {
						return Some(next);
					}

					if let Some(c) = next.find_child(ident) {
						return Some(c);
					}
				}
			}

		} else {

			if ident.compare(&self) {
				return Some(self.clone());
			}

			if let Some(child) = self.find_child(ident) {
				return Some(child);
			}

			if let Some(n) = self.prev() {
				let mut prev = n;

				/* do */ {

					if ident.compare(&prev) {
						return Some(prev);
					}

					if let Some(child) = prev.find_child(ident) {
						return Some(child);
					}

				} while let Some(n) = prev.prev() {
					prev = n;

					if ident.compare(&prev) {
						return Some(prev);
					}

					if let Some(child) = prev.find_child(ident) {
						return Some(child);
					}
				}
			}

			if let Some(n) = self.next() {
				let mut next = n;

				/* do */ {

					if ident.compare(&next) {
						return Some(next);
					}

					if let Some(child) = next.find_child(ident) {
						return Some(child);
					}

				} while let Some(n) = next.next() {
					next = n;

					if ident.compare(&next) {
						return Some(next);
					}

					if let Some(child) = next.find_child(ident) {
						return Some(child);
					}
				}
			}
		}

		None
	}

	fn find_child(&self, ident: &T) -> Option<Node<P>> {
		// in case we dont have a parent
		// iterates in the previous siblings
		// iterates in the next siblings

		if ident.compare(&self) {
			return Some(self.clone());
		}

		if let Some(child) = self.find_child(ident) {
			return Some(child);
		}

		if let Some(n) = self.prev() {
			let mut prev = n;

			/* do */ {

				if ident.compare(&prev) {
					return Some(prev);
				}

				if let Some(child) = prev.find_child(ident) {
					return Some(child);
				}

			} while let Some(n) = prev.prev() {
				prev = n;

				if ident.compare(&prev) {
					return Some(prev);
				}

				if let Some(child) = prev.find_child(ident) {
					return Some(child);
				}
			}
		}

		if let Some(n) = self.next() {
			let mut next = n;

			/* do */ {

				if ident.compare(&next) {
					return Some(next);
				}

				if let Some(child) = next.find_child(ident) {
					return Some(child);
				}

			} while let Some(n) = next.next() {
				next = n;

				if ident.compare(&next) {
					return Some(next);
				}

				if let Some(child) = next.find_child(ident) {
					return Some(child);
				}
			}
		}

		None
	}

}
