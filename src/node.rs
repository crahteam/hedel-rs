use std::{
	rc::{
		Weak,
		Rc
	}
};

use std::fmt::Debug;

use crate::cell::{
	HedelCell,
	RefHedel,
	RefMutHedel,
};

use crate::errors::HedelError;

/// Necessary indirection to ensure the presence of the `List` variant,
/// used in `NodeList::new`.
#[derive(Debug, Clone)]
pub enum Content<T: Debug + Clone> {
	List(Box<Content<T>>),
	Custom(T)
}

/// NodeInner contains pointers in both vertical and horizontal directions
/// and a custom content field.
#[derive(Debug, Clone)]
pub struct NodeInner<T: Debug + Clone> {
	pub next: Option<Node<T>>,
	pub prev: Option<WeakNode<T>>,
	pub child: Option<Node<T>>,
	pub parent: Option<WeakNode<T>>,
	pub content: Content<T>
}

/// `Rc` is a strong pointer meaning it increment a reference counter.
/// `Weak` is a weak pointer meaning it doesn't increment the reference counter,
/// letting you access the value if it still exists in memory,
/// modify it as its pointing to `HedelCell`,
/// but without holding it in memory any longer.
/// Necessary to avoid memory leaking.
#[derive(Debug, Clone)]
pub struct WeakNode<T: Debug + Clone> {
	pub inner: Weak<HedelCell<NodeInner<T>>>
}

impl<T: Debug + Clone> WeakNode<T> {
	/// upgrade `WeakNode` to `Node` if the `NodeInner` is still alive.
	pub fn upgrade(&self) -> Option<Node<T>> {
		Some(Node::<T> {
			inner: self.inner.upgrade()?
		})
	}
}

/// Wraps the inner value with an Rc<HedelCell<_>> pointer.
/// allowing for multiple owners and a mutable `NodeInner`
#[derive(Debug)]
pub struct Node<T: Debug + Clone > {
	pub inner: Rc<HedelCell<NodeInner<T>>>,
}

impl<T: Debug + Clone> Clone for Node<T> {
	fn clone(&self) -> Self {
		Self {
			inner: Rc::clone(&self.inner),
		}
	}
}

impl<T: Debug + Clone> Node<T> {
	/// Default constructor. Notice how it builds a stand-alone node,
	/// not pointing to any parent, any sibling and any child,
	/// but owning the content
	pub fn new(content: Content<T>) -> Self {
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
	
	/// clones the `NodeInner`'s content field and returns it.
	/// If the content is wrapped in nested `Content::List(...)`, 
	/// the value T in `Content::Custom(T)` at the end of the recursion is returned.
	pub fn to_content<'a>(&'a self) -> T {
		match &self.get().content {
			Content::Custom(val) => {
				val.clone()
			},
			Content::List(ptr) => {
				
				let mut content = &**ptr;
				
				while let Content::List(c) = content {
					content = &*c;
				}

				if let Content::Custom(ref val) = content {
					return val.clone();
				}

				unreachable!("There always is a Content::Custom at the end of the recursion.");
			}
		}
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

/// Copy-free alternative to `Node::to_content`.
///
/// # Example
/// ```
/// let node = node!(34);
/// let c = 20;
/// as_content!(&node, |num| {
///		if num > &c {
///			println!("I am {}", num);
///		}
/// });
/// ```
#[macro_export]
macro_rules! as_content {
	($node: expr, |$name: ident| $cl: expr) => {
		{
			match &$node.get().content {
				hedel_rs::Content::Custom($name) => {
					$cl;
				}
				hedel_rs::Content::List(ptr) => {
					let mut content = &**ptr;
					while let hedel_rs::Content::List(c) = content {
						content = &*c;
					}

					if let hedel_rs::Content::Custom($name) = content {
						$cl;
					}

					unreachable!("There always is a Content::Custom at the end of the recursion.");
				}
			}
		}
	}
}

pub trait HedelDetach<T: Debug + Clone> {
	fn detach(&self);
	fn detach_preserve(&self, vec: &mut NodeCollection<T>);
}

impl<T: Debug + Clone> HedelDetach<T> for Node<T> {
	/// Detaches a single node from the linked list by fixing the pointers between the 
	/// parent, the previous and next siblings. This also detaches all the children of the `Node`,
	/// which will only remain linked with the node itself.
	/// WARNING: This also re-sets the pointers in the node itself to None. 
	/// So when you are detecting nodes in a linked-list and detaching them, you cant iterate over them using this method
	/// as it would break the loop. Use `detach_preserve` instead.
	fn detach(&self) {
						// 1				3
		let mut tuple: (Option<Node<T>>, Option<Node<T>>) = ( None, None );

		if let Some(one) = self.prev() {
			// 1,2,3
			if let Some(three) = self.next() {
				tuple = (Some(one), Some(three));
			} else {
				// 1,2
				tuple = (Some(one), None);
			}
		} else {
			// 2, 3
			if let Some(three) = self.next() {
				tuple = ( None, Some(three));
			}
		}
		
		match tuple {
			(Some(one), Some(three)) => {
				one.get_mut().next = Some(three.clone());
				three.get_mut().prev = Some(one.downgrade());
			},
			(Some(one), None) => {
				one.get_mut().next = None;
			},
			(None, Some(three)) => {
				three.get_mut().prev = None;
				if let Some(parent) = self.parent() {
					parent.get_mut().child = Some(three.clone());
				}
			},
			(None, None) => {
				if let Some(parent) = self.parent() {
					parent.get_mut().child = None;
				}
			}
		}

		self.free();
	}
	/// Detaches a single node from the linked list like `detach`, but doesn't re-set the pointers inside the Node.
	/// This should only be used when you have to iterate over a linked list and detach some `Node`s.
	/// You should create a vector to store the detached nodes, and iterate over them only when the while loop is 
	/// compleated, re-setting the `parent`, `prev`, `next` fields to `None`.
	///
	/// # Example
	/// 
	/// ```
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
	/// ```
	fn detach_preserve(&self, vec: &mut NodeCollection<T>) {
							// 1				3
		let mut tuple: (Option<Node<T>>, Option<Node<T>>) = ( None, None );

		if let Some(one) = self.prev() {
			// 1,2,3
			if let Some(three) = self.next() {
				tuple = (Some(one), Some(three));
			} else {
				// 1,2
				tuple = (Some(one), None);
			}
		} else {
			// 2, 3
			if let Some(three) = self.next() {
				tuple = ( None, Some(three));
			}
		}
		
		match tuple {
			(Some(one), Some(three)) => {
				one.get_mut().next = Some(three.clone());
				three.get_mut().prev = Some(one.downgrade());
			},
			(Some(one), None) => {
				one.get_mut().next = None;
			},
			(None, Some(three)) => {
				three.get_mut().prev = None;
				if let Some(parent) = self.parent() {
					parent.get_mut().child = Some(three.clone());
				}
			},
			(None, None) => {
				if let Some(parent) = self.parent() {
					parent.get_mut().child = None;
				}
			}
		}

		vec.push(self.clone());
	}
}

/// `NodeCollection` represents a `Vec` of `Node`s. Usually retrived by collecting over
/// a `Node` linked list using the `HedelCollect` trait implementation.
/// WARNING: this is not a linked list, but simply a collection of unrelated nodes.
/// The contained nodes might come from separated linked lists or from the same one.
pub struct NodeCollection<T: Debug + Clone> {
	pub nodes: Vec<Node<T>>
}

impl<T: Debug + Clone> NodeCollection<T> {
	
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

	/// Push a node to the collection.
	pub fn push(&mut self, node: Node<T>) {
		self.nodes.push(node);
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

impl<T: Debug + Clone> IntoIterator for NodeCollection<T> {
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
/// ```
/// pub enum NumIdent {
///		BiggerThan(i32),
///		SmallerThan(i32)
/// }
///
/// impl NodeComparable<i32> for NumIdent {
/// 	fn compare(&self, node: &Node<i32>) -> bool {
/// 		as_content!(node, |content| {
///				match &self {
///					NumIdent::BiggerThan(num) => {
///						content > num
///					},
///					NumIdent::SmallerThan(num) => {
///						content < num
///					}
///				}
///			});			
///		}
/// }
/// ```
pub trait NodeComparable<T: Debug + Clone> {
	fn compare(&self, node: &Node<T>) -> bool;
}

pub trait HedelCollect<P: Debug + Clone, T: NodeComparable<P>> {
	fn collect_siblings(&self, ident: &T) -> NodeCollection<P>;
	fn collect_children(&self, ident: &T) -> NodeCollection<P>;
	fn collect_linked_list(&self, ident: &T) -> NodeCollection<P>;
}                                                         

impl<P: Debug + Clone, T: NodeComparable<P>> HedelCollect<P, T> for Node<P> {
	/// Given an identifier of type implementing `NodeComparable` this iterates over all the nodes
	/// in the linked list horizontally ( iterates over the siblings, previous and next ),
	/// and compare every node. The nodes satisfying the identifier get collected into a `NodeCollection`.
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

		NodeCollection::<P>::new(collection) 
	}

	/// Given an identifier of type implementing `NodeComparable` this iterates over all the nodes that stand 
	/// lower and deeper in the linked list. Every child satysfying the identifier get collected into a `NodeCollection`
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

		NodeCollection::<P>::new(collection)
	}
	
	/// Given an identifier of type implementing `NodeComparable` this iterates over all the nodes in the 
	/// linked list both horizontally and vertically ( iterates horizontally in each hierarchical level,
	/// up to the top parent and down to the deepest child also
	/// iterating vertically and horizontally for each layer of the children ).
	/// The nodes satisfying the identifier get collected into a `NodeCollection`.
	///
	/// # Example
	///
	/// ```
	///	let node = node!(
	/// 	node!(8),
	///		node!(3),
	///		node!(7),
	///		node!(6, node!(3))
	/// );
	///
	/// // retrive any child
	/// let a = node.get_last_child().unwrap();
	///
	/// let collection = a.collect_linked_list(&NumIdent::SmallerThan(5));
	///
	/// for node in collection.into_iter() {
	/// 	println!("{}", node.to_content());
	/// }
	/// ```
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

		NodeCollection::<P>::new(collection)
	}
} 

pub trait HedelFind<P: Debug + Clone, T: NodeComparable<P>> {
	fn find_next(&self, ident: &T) -> Option<Node<P>>;
	fn find_prev(&self, ident: &T) -> Option<Node<P>>;
	fn find_sibling(&self, ident: &T) -> Option<Node<P>>;
	fn find_child(&self, ident: &T) -> Option<Node<P>>;
	fn find_linked_list(&self, ident: &T) -> Option<Node<P>>;
}                                                         

impl<P: Debug + Clone, T: NodeComparable<P>> HedelFind<P, T> for Node<P> {
	/// Get the first `Node` in the linked list, at the same depth-level of `&self` and coming after it,
	/// matching the identifier.
	/// This guarantees to actually retrive the closest `Node`.
	///
	/// # Example
	///
	/// ```
	/// let node = node!(
	/// 	node!(1),
	///		node!(34),
	///		node!(66)
	/// ); 
	/// 
	///	let one = node.get_first_child();
	/// let ss: Option<Node<i32>> = one.find_next(&NumIdent::BiggerThan(50)); // returns the `node!(66)`
	/// ```
	fn find_next(&self, ident: &T) -> Option<Node<P>> {
		if let Some(next) = self.next() {
			let mut next = next;

			/* do */ {

				if ident.compare(&next) {
					return Some(next);
				}
				
			} while let Some(n) = next.next() {
				next = n;

				if ident.compare(&next) {
					return Some(next);
				}
			}
		}
	
		None
	}
	
	/// Get the first `Node` in the linked list, at the same depth-level of `&self` and coming before it,
	/// matching the identifier.
	/// This guarantees to actually retrive the closest `Node`.
	fn find_prev(&self, ident: &T) -> Option<Node<P>> {
		if let Some(prev) = self.prev() {
			let mut prev = prev;

			/* do */ {

				if ident.compare(&prev) {
					return Some(prev);
				}
				
			} while let Some(n) = prev.prev() {
				prev = n;

				if ident.compare(&prev) {
					return Some(prev);
				}
	
			}
		}
		None

	}
	
	/// Get a `Node` somewhere in the linked list matching the identifier.
	/// WARNING: it's not guaranteed to retrive the closest `Node`. Only use when you don't
	/// care about which node is retrived as long as it matches the identifier or when you are 100% sure
	/// that there isn't more than one `Node` satisfying the identifier in the linked list.
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

	/// Get the first child `Node` of `&self` in the linked list matching the identifier. 
	/// WARNING: it's not guaranteed to retrive the closest `Node`. Only use when you don't
	/// care about which node is retrived as long as it matches the identifier or when you are 100% sure
	/// that there isn't more than one `Node` satisfying the identifier in the children.
	fn find_child(&self, ident: &T) -> Option<Node<P>> {
		if let Some(child) = self.child() {
			let mut child = child;
			/* do */ {

				if ident.compare(&child) {
					return Some(child);
				}
				
				if let Some(next) = child.next() {
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

			} while let Some(c) = child.child() {
				child = c;	

				if ident.compare(&child) {
					return Some(child);
				}
				
				if let Some(next) = child.next() {
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

			}
		}	

		None
	}

	/// In the case you can't know if the `Node` you are looking for comes before or after, here's a combination of the two previous methods. 
	/// Always prefer using `HedelFind::find_next` and `HedelFind::find_prev` when you know the position of the `Node`,
	/// as they might be faster.
	fn find_sibling(&self, ident: &T) -> Option<Node<P>> {
		// in case we dont have a parent
		// iterates in the previous siblings
		// iterates in the next siblings

		//if let Some(child) = self.find_child(ident) {
		//	return Some(child);
		//}

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

pub trait HedelGet<T: Debug + Clone> {
	fn get_first_sibling(&self) -> Option<Node<T>>;
	fn get_last_sibling(&self) -> Option<Node<T>>;
	fn get_last_child(&self) -> Option<Node<T>>;
}

impl<T: Debug + Clone> HedelGet<T> for Node<T> {

	/// Get the first `Node` in the linked list at the same depth level of `&self`.
	/// If None is returned, `&self` is the first `Node` at that depth level.
	fn get_first_sibling(&self) -> Option<Node<T>> {
		
		// faster in case there's a parent
		if let Some(parent) = self.parent() {
			return parent.child();
		}

		let mut first;

		/* do */ {
			
			if let Some(prev) = self.prev() {
				first = prev;
			} else { return None; }

		} while let Some(prev) = first.prev() {
			first = prev;
		}

		Some(first)
	}

	/// Get the last `Node` in the linked list at the same depth level of `&self`.
	/// If None is returned, `&self` is the last `Node` at that depth level.
	fn get_last_sibling(&self) -> Option<Node<T>> {
		
		let mut last;

		/* do */ {

			if let Some(next) = self.next() {
				last = next;
			} else { return None; }

		} while let Some(next) = last.next() {
			last = next;
		}

		Some(last)
	}

	/// Get the last child `Node` of `&self`
	/// If None is returned, `&self` doesn't have any children.
	/// NOTE: use &self.child() to get the first `Node`.
	fn get_last_child(&self) -> Option<Node<T>> {

		if let Some(child) = self.child() {
			
			let mut child = child;
			
			if let Some(s) = child.get_last_sibling() {
				return Some(s);
			}

			return Some(child);

		} None
	}
}

/// Generate a node blazingly fast, with any number of child nodes.
/// 
/// # Example
///
/// ```
/// let my_node = node!("Parent",
///		node!("Child"),
///		node!("Child")
///	);
///
/// let another_node = node!("Another");
/// ```
#[macro_export]
macro_rules! node {
	($content: expr $(,$node: expr)*) => {
		{
			let mut node = hedel_rs::Node::new(hedel_rs::Content::Custom($content));

			let mut children: Vec<hedel_rs::Node<_>> = Vec::new();

			let mut lists: Vec<usize> = Vec::new();

			let mut c = 0;

			$(
				let n: hedel_rs::Node::<_> = $node.into();
				
				if let hedel_rs::Content::List(_) = n.get().content {
					lists.push(c as usize);
				}

				children.push(n);

				c += 1;
			)*

			if children.len() > 0 {
				node.get_mut().child = Some(children[0].clone());
	
				c = 0;
				
				let max_idx = children.len() - 1;

				for ref child in &children {
					let mut borrow = child.get_mut();

					if c != max_idx {
						borrow.next = Some(children[c + 1].clone()); 
					}

					if c != 0 {
						borrow.prev = Some(children[c - 1].downgrade());
					}

					borrow.parent = Some(hedel_rs::WeakNode {
						inner: std::rc::Rc::downgrade(&node.inner)
					});

					c += 1;
				}

			} 
		
			for idx in lists.into_iter() {
				
				let first = children[idx].clone();

				if idx > 0 {
					if let Some(prev) = children.get(idx - 1) {
						prev.get_mut().next = Some(first.clone());
						first.get_mut().prev = Some(prev.downgrade());
					}
				}

				if let Some(last) = first.get_last_sibling() {
					if let Some(next) = children.get(idx + 1) {
						next.get_mut().prev = Some(last.downgrade());
						last.get_mut().next = Some(next.clone());
					}
				}

				let mut child = first;

				/* do */ {

					child.get_mut().parent = Some(node.downgrade());

				} while let Some(ch) = child.next() {
					child = ch;
					child.get_mut().parent = Some(node.downgrade());
				}
			}

			node
		}
	}
}

// TODO: create a node_no_child macro for cases when the user is sure there won't be any children
// e.g void html elements.
