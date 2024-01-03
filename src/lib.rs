mod bitwriter;
pub mod cl_macro;

pub use bitwriter::BitWriter;
pub use cl_macro::*;

use std::collections::hash_map::*;
use std::rc::*;

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

//Type used for cache during normalization
//TODO: Benchmark against BTree
type Cache = HashMap<String, CacheCell>;

#[derive(Debug)]
enum CacheCell {
	Normal(Rc<Combinator>),
	Abnormal,
	Unsure,
}

impl Combinator {
	//Ok if normalized
	//Err(true) means known not to normalize
	//Err(false) if limit reached
	pub fn normal_form(&self, limit: usize) -> Result<Self, bool> {
		let mut copy = self.clone();

		match copy.normalize(limit) {
			Some(true) => Ok(copy),
			Some(false) => Err(true),
			None => Err(false),
		}
	}

	//Put everything in terms of S & K
	pub fn sk_ify(&mut self) {
		match self {
			Self::S | Self::K | Self::Var(_) => (),
			Self::Named(_, def_box) => {
				let def = std::mem::replace(def_box.as_mut(), Self::S);
				*self = def;
				self.sk_ify()
			}
			Self::App(terms) => match &terms[..] {
				[_] | [.., Self::App(_)] => {
					self.reduce();
					self.sk_ify()
				}
				_ => {
					for term in terms {
						term.sk_ify()
					}
				}
			},
		}
	}

	pub fn normalize(&mut self, mut limit: usize) -> Option<bool> {
		//self.sk_ify();
		self.normalize_with(&mut limit, &mut Default::default())
	}

	//Some to indicate whether it can be normalized
	//None if unsure
	fn normalize_with(&mut self, limit: &mut usize, cache: &mut Cache) -> Option<bool> {
		if *limit == 0 {
			return None;
		}

		*limit -= 1;

		self.simplify_with(cache);

		let name = format!("{}", self);

		use CacheCell::*;
		match cache.get_mut(&name) {
			Some(cell) => {
				return match &cell {
					Normal(term) => {
						*self = term.as_ref().clone();
						Some(true)
					}
					Abnormal | Unsure => {
						*cell = Abnormal;
						Some(false)
					}
				}
			}
			None => {
				cache.insert(name.clone(), CacheCell::Unsure);
			}
		}

		let normalizes = match self {
			Self::S | Self::K | Self::Var(_) => Some(true),

			Self::Named(_, def) => def.normalize_with(limit, cache),

			Self::App(terms) => match &terms[..] {
				[] | [_] | [.., Self::App(_)] => unreachable!(),

				[.., Self::Named(_, _)] => {
					let named = terms.last_mut().unwrap();
					named.normalize_with(limit, cache);

					self.reduce();
					self.normalize_with(limit, cache)
				}

				[_, _, _, Self::S] | [_, _, Self::K] => {
					self.reduce();
					self.normalize_with(limit, cache)
				}

				[.., _x, _g, _f, Self::S] => {
					let _s = terms.pop().unwrap();
					let f = terms.pop().unwrap();
					let g = terms.pop().unwrap();
					let x = terms.pop().unwrap();

					terms.push(Self::App(vec![x, g, f, Self::S]));

					let redex = terms.last_mut().unwrap();

					match redex.normalize_with(limit, cache) {
						Some(true) => self.normalize_with(limit, cache),
						Some(false) => Some(false),
						None => None,
					}
				}

				[.., _y, _x, Self::K] => {
					let _k = terms.pop().unwrap();
					let x = terms.pop().unwrap();
					let y = terms.pop().unwrap();

					terms.push(Self::App(vec![y, x, Self::K]));

					let redex = terms.last_mut().unwrap();

					match redex.normalize_with(limit, cache) {
						Some(true) => self.normalize_with(limit, cache),
						Some(false) => Some(false),
						None => None,
					}
				}

				_ => terms
					.iter_mut()
					.rev()
					.map(|term| term.normalize_with(limit, cache))
					.reduce(|acc, next| acc.zip(next).map(|(a, b)| a && b))
					.unwrap(),
			},
		};

		if let Some(normal) = normalizes {
			if normal {
				cache.insert(name, CacheCell::Normal(self.clone().into()));
			} else {
				cache.insert(name, CacheCell::Abnormal);
			}
		}

		normalizes
	}

	//Normalizes unnamed combinators that have already been seen
	//Does not perform any reductions otherwise
	fn simplify_with(&mut self, cache: &Cache) {
		let name = format!("{}", self);
		use CacheCell::Normal;
		if let Some(Normal(simple)) = cache.get(&name) {
			*self = simple.as_ref().clone();
		};

		if let Self::App(terms) = self {
			match &mut terms[..] {
				[_] | [.., Self::App(_)] => {
					self.reduce();
					self.simplify_with(cache);
				}

				_ => {
					for term in terms.iter_mut() {
						term.simplify_with(cache)
					}
				}
			}
		}
	}

	//Performs all 'alias reductions', i.e., reductions which don't correspond
	//to a redex in the SK basis.
	//Used so that two calls to reduce don't generate the same BCL if an alias is used
	pub fn alias_reduce(&mut self) {
		loop {
			match self {
				Self::S | Self::K | Self::Var(_) => break,
				Self::Named(_, _) => {
					self.reduce();
				}
				Self::App(terms) => match &mut terms[..] {
					[_] | [.., Self::App(_) | Self::Named(_, _)] => {
						self.reduce();
					}
					_ => break,
				},
			}
		}
	}

	pub fn reduce(&mut self) -> bool {
		match self {
			Self::Named(_, inner) => {
				let old_inner = std::mem::replace(inner.as_mut(), Self::S);
				*self = old_inner;
				true
			}
			Self::App(terms) => match &mut terms[..] {
				[_] => {
					*self = terms.pop().unwrap();
					self.reduce()
				}
				[.., _y, _x, Self::K] => {
					terms.pop();
					let x = terms.pop().unwrap();
					terms.pop();

					//For efficiency reasons, to avoid reallocation
					//and an extra reduction step
					if let Self::App(v) = x {
						terms.extend_from_slice(&v);
					} else {
						terms.push(x);
					}
					true
				}
				[.., _x, _g, _f, Self::S] => {
					terms.pop();
					let mut f = terms.pop().unwrap();
					let mut g = terms.pop().unwrap();
					let x = terms.pop().unwrap();

					g.apply(x.clone());
					terms.push(g);

					//For efficiency reasons, to avoid reallocation
					//and an extra reduction step
					f.apply(x);
					if let Self::App(v) = f {
						terms.extend_from_slice(&v);
					} else {
						terms.push(f);
					}

					true
				}
				[.., Self::App(_)] => {
					let Some(Self::App(inner)) = terms.pop() else {
						unreachable!()
					};

					terms.extend_from_slice(&inner[..]);
					self.reduce()
				}
				[.., Self::Named(_, _)] => {
					let Some(Self::Named(_, top)) = terms.pop() else {
						unreachable!()
					};

					if let Self::App(v) = *top {
						terms.extend_from_slice(&v);
					} else {
						terms.push(*top);
					}

					true
				}
				_ => false,
			},
			_ => false,
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
