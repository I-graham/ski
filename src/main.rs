use ski::*;

pub mod ski;

#[allow(non_snake_case)]
fn main() {
	let I = combinator!(S K K);
	let T = combinator!(K);
	let F = combinator! (K I);

	fully_reduce(combinator!(T 't' 'f'));
	fully_reduce(combinator!(F 't' 'f'));
	fully_reduce(combinator!(I 'x'));

	/* 	let never = combinator!(S S S (S S) S S);

	fully_reduce(never);  */

	let M = combinator!(S I I);
	let O = combinator!(M M);

	fully_reduce(O);

	fully_reduce(combinator!(I I));
	print_bcl(&combinator!(I I));

}

fn print_bcl(term: &Combinator) {
	let bcl = term.bcl();
	for byte in bcl {
		print!("{:08b}", byte);
	}
	println!();
}

fn fully_reduce(mut lambda: Combinator) {
	let mut i = 0;
	println!("{}", lambda);
	while lambda.reduce() {
		i += 1;
		println!("{}", lambda);
		//		if i % 10000 == 0 {println!("{}: {}", i, lambda.size())}
		if i > 20 {
			break;
		}
	}
	println!();
}
