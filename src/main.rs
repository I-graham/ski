pub mod ski;

fn main() {
	let identity = combinator!(((S K) K) S);
	println!("{}", identity);
	println!("{}", identity.reduce());
}
