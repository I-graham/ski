use ski::*;

pub mod ski;

#[allow(non_snake_case)]
fn main() {
	let I = combinator!(S K K);
	let T = combinator!(K);
	let F = combinator! (K I);

	fully_reduce(&combinator!(T 't' 'f'));
	fully_reduce(&combinator!(F 't' 'f'));
	fully_reduce(&combinator!(I 'x'));

	/* 	let never = combinator!(S S S (S S) S S);

	fully_reduce(never);  */

	let M = combinator!(S I I);
	let O = combinator!(M M);

	fully_reduce(&combinator!(I I));
	print_bcl(&M);

	let B = combinator!(S (K S) K);
	let C = combinator!(S (B B S) (K K));
	let Y = combinator!(B M (C B M));

	normalized(&I);
	normalized(&T);
	normalized(&F);
	normalized(&combinator!(I I));
	normalized(&O);
	normalized(&Y);
	normalized(&combinator!(F I));
	normalized(&combinator!(Y F));
	normalized(&combinator!(Y T));
	normalized(&combinator!(Y (K K)));
	normalized(&combinator!(M));
}

fn normalized(term: &Combinator) {
	let name = format!("{}", &term);
	let normal = term.normal_form(100000);
	print!("{} -> ", name);
	match normal {
		Some(nf) => println!("{}!", nf),
		None => println!("No NF found."),
	}
}

fn print_bcl(term: &Combinator) {
	let bcl = term.bcl();
	for byte in bcl {
		print!("{:08b}", byte);
	}
	println!();
}

fn fully_reduce(term: &Combinator) {
	let mut lambda = term.clone();
	let mut i = 0;
	println!("{}", lambda);
	while lambda.reduce() {
		i += 1;
		println!("{}", lambda);
		//		if i % 10000 == 0 {println!("{}: {}", i, lambda.size())}
		if i > 100 {
			break;
		}
	}
	println!();
}
