use crate::{
	node::{
		Node,
		Content,
		WeakNode,
		HedelGet,
		HedelFind,
		NodeComparable
	}
};

use std::fmt::Debug;

/// `NodeList` concreatly is its first `Node`.
/// This design allows for sibling nodes at the root-level.
#[derive(Debug, Clone)]
pub struct NodeList<T: Debug + Clone>(pub Option<Node<T>>);

impl<T: Debug + Clone> NodeList<T> {
	pub fn new(node: Node<T>) -> Self {
		let mut content = Box::new(node.get().content.clone());
		node.get_mut().content = Content::List(content);
		Self(Some(node))
	}

	pub fn get_first_sibling(&self) -> Option<Node<T>> {
		if let Some(s) = &self.0 {
			if let Some(last) = self.get_first_sibling() {
				return Some(last);
			} 

			return Some(s.clone());
		}
		None
	}

	pub fn get_last_sibling(&self) -> Option<Node<T>> {
		if let Some(s) = &self.0 {
			if let Some(last) = self.get_last_sibling() {
				return Some(last);
			} 

			return Some(s.clone());
		}
		None
	}

	pub fn find_sibling<P: NodeComparable<T>>(&self, ident: &P) -> Option<Node<T>> {

		if let Some(s) = &self.0 {
			if let Some(sib) = s.find_next(ident) {
				return Some(sib);
			} 
			if ident.compare(&s) {
				return Some(s.clone());
			}
		}

		None
	}
	
	pub fn find_linked_list<P: NodeComparable<T>>(&self, ident: &P) -> Option<Node<T>> {
	
		if let Some(s) = &self.0 {
			if let Some(sib) = s.find_next(ident) {
				return Some(sib);
			} 
			if ident.compare(&s) {
				return Some(s.clone());
			}
		}

		None	
	}
}

/// Generate a linked list blazingly fast and append any number of `Nodes`
/// 
/// # Example
///
/// ```
/// let my_list = list!{
/// 	node!(2, node!(3)),
///		node!(45),
///		node!(36)
/// };
/// ```
#[macro_export]
macro_rules! list {
	($($node: expr),*) => {
		{
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
				
				c = 0;

				for ref child in &children {
					
					let mut borrow = child.get_mut();
					
					if c != children.len() - 1 {
						borrow.next = Some(children[c + 1].clone()); 
					}

					if c != 0 {
						borrow.prev = Some(children[c - 1].downgrade());
					}

					borrow.parent = None;

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
			}

			hedel_rs::NodeList::new(children[0].clone())
		}
	}
}
