pub mod ski;

fn main() {
	let lambda = combinator!(S K K S);
	let mut lambda = combinator!(lambda K K);
	loop {
		println!("{}", lambda);

		if let Some(lambda2) = lambda.rc().reduce() {
			lambda = (*lambda2).clone();
		} else {
			break;
		}
	}
}
