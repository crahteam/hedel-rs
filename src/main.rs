use hedel_rs::node::{
	Node
};

fn main() {
	let now = std::time::Instant::now();

	let x = Node::<i32>::new(7);

	x.get_mut().content = 90;
	println!("{:?}", x.get().content);

	let mut borr = x.get_mut();
	borr.content = 3;

	println!("{:?}", now.elapsed().as_nanos());
}
