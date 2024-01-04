use ski::*;

#[allow(non_snake_case)]
fn main() {
	let I = combinator!(S K K);
	let T = combinator!(K);
	let F = combinator! (K I);

	//composition
	let B = combinator!(S (K S) K);
	//flip
	let C = combinator!(S (B B S) (K K));

	let fst = combinator!(K);
	let snd = combinator!(K I);

	let pair = combinator!(S (B (B B S) (B B S I K)) (K K));

	let tuple = combinator!(pair 'a' 'b');

	let nil = combinator!(F);

	let cons = combinator!(pair);

	let head = combinator!(fst);
	let tail = combinator!(snd);

	let isnil = combinator!(pair (K (K (K F))) T);

	let construct = |list: &[Combinator]| -> Combinator {
		let mut out = combinator!(nil);

		for term in list.iter().rev() {
			let mut new = cons.clone();
			new.apply(term.clone());
			new.apply(out);
			out = new;
		}

		out
	};

	let list = construct(&[combinator!(S), combinator!(K), combinator!(I)]);

	normal_form(&combinator!(isnil list));
	normal_form(&combinator!(list head));

	normal_form(&combinator!(isnil (list tail)));
	normal_form(&combinator!((list tail) head));

	normal_form(&combinator!(isnil ((list tail) tail)));
	normal_form(&combinator!(((list tail) tail) head));

	normal_form(&combinator!(isnil (((list tail) tail) tail)));
	normal_form(&combinator!((((list tail) tail) tail) head));
}

fn normal_form(term: &Combinator) {
	let name = format!("{}", &term);
	let normal = term.normal_form(1000);
	print!("{} -> ", name);
	match normal {
		Ok(nf) => println!("{}!", nf),
		Err(true) => println!("No NF exists."),
		Err(false) => println!("No NF found."),
	}
}
