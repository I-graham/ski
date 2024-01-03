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

	normal_form(&I);
	normal_form(&T);
	normal_form(&F);
	normal_form(&combinator!(I I));
	normal_form(&Y);
	normal_form(&combinator!(M));
	normal_form(&combinator!(F I));
	normal_form(&combinator!(Y F));
	normal_form(&O);
	normal_form(&combinator!(Y T));
	normal_form(&combinator!(Y (K K)));
}

fn normal_form(term: &Combinator) {
	let name = format!("{}", &term);
	let normal = term.normal_form(10000);
	print!("{} -> ", name);
	match normal {
		Ok(nf) => println!("{}!", nf),
		Err(true) => println!("No NF exists."),
		Err(false) => println!("No NF found."),
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
