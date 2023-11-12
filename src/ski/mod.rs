mod bitwriter;
pub mod cl_macro;

pub use bitwriter::BitWriter;
pub use cl_macro::*;

use std::collections::*;
use std::rc::*;

//Type used for cache during normalization
//TODO: Benchmark against BTree
type Normals = HashMap<Vec<u8>, Option<Rc<Combinator>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Combinator {
	S,
	K,
	//Combinators are stored in reverse,
	//to allow for efficient reduction
	App(Vec<Combinator>),
	Var(char),
	Named(&'static str, Box<Combinator>),
}

impl Combinator {
	pub fn normal_form(&self, limit: usize) -> Option<Self> {
		let mut copy = self.clone();

		if copy.normalize(limit, &mut Default::default()) {
			Some(copy)
		} else {
			None
		}
	}

	fn normalize(&mut self, limit: usize, cache: &mut Normals) -> bool {
		if limit == 0 {
			return false;
		}

		let bcl = self.bcl();

		if let Some(found) = cache.get(&bcl) {
			return match found {
				Some(term) => {
					*self = term.as_ref().clone();
					true
				}
				None => false,
			};
		}

		self.k_reduce();
		let normalized = match self {
			Self::S | Self::K | Self::Var(_) => true,
			Self::Named(_, term) => term.normalize(limit - 1, cache),
			Self::App(terms) => match &mut terms[..] {
				[.., z, _y, _x, Self::S] => {
					z.normalize(limit - 1, cache);
					if self.reduce() {
						self.normalize(limit - 1, cache)
					} else {
						self.is_normal()
					}
				}
				[.., Self::App(_)] | [.., Self::Named(_, _)] => {
					self.reduce();
					self.normalize(limit - 1, cache)
				}
				_ => terms.iter().all(|term| term.is_normal()),
			},
		};

		if normalized {
			cache.insert(bcl, Some(Rc::new(self.clone())));
		} else {
			cache.insert(bcl, None);
		}

		normalized
	}

	pub fn is_normal(&self) -> bool {
		match self {
			Self::S | Self::K | Self::Var(_) => true,
			Self::Named(_, term) => term.is_normal(),
			Self::App(terms) => match &terms[..] {
				[.., _, _, _, Self::S]
				| [.., _, _, Self::K]
				| [.., Self::App(_)]
				| [.., Self::Named(_, _)] => false,
				_ => terms.iter().all(|term| term.is_normal()),
			},
		}
	}

	pub fn reduce(&mut self) -> bool {
		match self {
			Self::App(args) => match &args[..] {
				[.., _y, _x, Self::K] => {
					args.pop();
					let x = args.pop().unwrap();
					args.pop();

					//For efficiency reasons, to avoid reallocation
					//and an extra reduction step
					if let Self::App(v) = x {
						args.extend_from_slice(&v);
					} else {
						args.push(x);
					}
					true
				}
				[.., _x, _g, _f, Self::S] => {
					args.pop();
					let mut f = args.pop().unwrap();
					let mut g = args.pop().unwrap();
					let x = args.pop().unwrap();

					g.apply(x.clone());
					args.push(g);

					//For efficiency reasons, to avoid reallocation
					//and an extra reduction step
					f.apply(x);
					if let Self::App(v) = f {
						args.extend_from_slice(&v);
					} else {
						args.push(f);
					}

					true
				}
				[.., Self::App(_)] => {
					let Some(Self::App(inner)) = args.pop() else {
						unreachable!()
					};

					args.extend_from_slice(&inner[..]);
					self.reduce()
				}
				[.., Self::Named(_, _)] => {
					let Some(Self::Named(_, top)) = args.pop() else {
						unreachable!()
					};

					if let Self::App(v) = *top {
						args.extend_from_slice(&v);
					} else {
						args.push(*top);
					}

					true
				}
				_ => false,
			},
			_ => false,
		}
	}

	//reduce all Ks, since K is strongly reducing, so this can be done
	//without worry of a misstep.
	pub fn k_reduce(&mut self) {
		if let Self::App(args) = self {
			match &args[..] {
				[.., _y, _x, Self::K] => {
					args.pop().unwrap();
					let x = args.pop().unwrap();
					args.pop().unwrap();

					for c in args.iter_mut() {
						c.k_reduce();
					}

					args.push(x);
				}
				_ => {
					for c in args {
						c.k_reduce();
					}
				}
			}
		}
	}

	//Apply without additional copy
	pub fn apply(&mut self, x: Self) {
		if let Self::App(args) = self {
			//Push x to front of args, without copy or reallocation
			args.reserve(1);
			let mut tmp = x;
			for arg in args.iter_mut() {
				std::mem::swap(&mut tmp, arg);
			}
			args.push(tmp);
		} else {
			*self = Self::App(vec![x, self.clone()]);
		}
	}

	pub fn size(&self) -> usize {
		match self {
			Self::App(args) => args.iter().map(Combinator::size).sum(),
			_ => 1,
		}
	}

	pub fn bcl(&self) -> Vec<u8> {
		let mut writer = BitWriter::default();
		self.write_bcl(&mut writer);
		writer.finish()
	}

	fn write_bcl(&self, writer: &mut BitWriter) {
		match self {
			Self::K => {
				//00
				writer.emit_bit(false);
				writer.emit_bit(false);
			}
			Self::S => {
				//01
				writer.emit_bit(false);
				writer.emit_bit(true);
			}
			Self::App(terms) => {
				//1 <term> <term>
				for _ in 0..terms.len() - 1 {
					writer.emit_bit(true);
				}
				for term in terms.iter().rev() {
					term.write_bcl(writer);
				}
			}
			Self::Named(_, term) => term.write_bcl(writer),
			_ => unimplemented!(),
		}
	}
}

impl std::fmt::Display for Combinator {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::S => write!(fmt, "S"),
			Self::K => write!(fmt, "K"),
			Self::Var(var) => write!(fmt, "{}", var),
			Self::Named(name, _) => write!(fmt, "{}", name),
			Self::App(combs) => {
				for comb in combs.iter().rev() {
					if comb.size() == 1 {
						write!(fmt, "{}", comb)?;
					} else {
						write!(fmt, "({})", comb)?;
					}
				}
				Ok(())
			}
		}
	}
}
