use ski::Combinator;

pub mod ski;

#[allow(non_snake_case)]
fn main() {
	let I = combinator! (S K K);
	let T = combinator! (K);
	let F = combinator! (K I);

	fully_reduce(combinator!(T 't' 'f'));
	fully_reduce(combinator!(F 't' 'f'));

	
}

fn fully_reduce(mut lambda: Combinator) {
	while let Some(red) = lambda.reduced() {
		println!("{}", lambda);
		lambda = red;
	}
	println!("{}", lambda);
	println!();
}
