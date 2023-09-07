/*
use hedel_rs::{
	node,
	list,
	node::{
		Node,
		Content,
	},
	prelude::*
};

pub enum Ident {
	BiggerThan(i32),
	SmallerThan(i32)
}

impl NodeComparable<i32> for Ident {
	fn compare(&self, node: &Node<i32>) -> bool {
		match &node.get().content {
			Content::Custom(num) => {
				match &self {
					Self::BiggerThan(n) => return num > n,
					Self::SmallerThan(n) => return num < n
				}
			},
			Content::List(ptr) => {
				let mut list = *ptr.clone();
				while let Content::List(b) = list {
					list = *b.clone();
				}

				if let Content::Custom(num) = list {
					match &self {
						Self::BiggerThan(n) => return num > *n,
						Self::SmallerThan(n) => return num < *n
					}
				}

				return false;
			}
		}
	}
}
fn main() {
    unsafe { backtrace_on_stack_overflow::enable() };
	let now = std::time::Instant::now();
	let l =  node!(
			16, // b
			node!(56,
				node!(11),
				node!(9)
			),
			node!(7),
			node!(4),
			node!(8)
		);

	//let b = l.find_linked_list(&Ident::SmallerThan(13)).unwrap();

	//b.detach();

	let a = l.find_linked_list(&Ident::SmallerThan(5)).unwrap();

	let b = a.collect_siblings(&Ident::SmallerThan(13));

	for node in b.into_iter() {
		println!("{:#?}", node.get().content);
	}


	let node = node!(2);
	let c = 0;
	hedel_rs::as_content!(&node, |num| {
		if num > &c {
			println!("sono il {}", num);
		}
	});
	
	let due = node.to_content();


	println!("sono il {}", due);

	let mut nodo = node!(1, node!(2), node!(4));
	let uno = nodo.get_last_child().unwrap();	
	uno.append_next(node!(5));
	let tre = nodo.get_last_child().unwrap();
	println!("INSERIMENTO: {}", tre.to_content());
	println!("{:?}", now.elapsed().as_nanos());
}
*/

fn main() {

}
