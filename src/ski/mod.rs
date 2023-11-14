mod bitwriter;
pub mod cl_macro;

pub use bitwriter::BitWriter;
pub use cl_macro::*;

use std::collections::*;
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
type Normals = HashMap<Vec<u8>, CacheCell>;

#[derive(Debug)]
enum CacheCell {
	Normal(Rc<Combinator>),
	Abnormal,
	Unsure,
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

	fn normalize(&mut self, mut limit: usize, cache: &mut Normals) -> bool {
		if limit == 0 {
			return false;
		}

		let name = format!("{}", self);
		//dbg!(&name);
		let bcl = self.bcl();

		if let Some(cell) = cache.get_mut(&bcl) {
			use CacheCell::*;
			return match cell {
				Normal(term) => {
					*self = term.as_ref().clone();
					true
				}
				Abnormal => false,
				Unsure => {
					*cell = Abnormal;
					false
				}
			};
		}

		let normalized = loop {
			if limit == 0 {
				break false;
			}

			match self {
				Self::S | Self::K | Self::Var(_) => break true,
				Self::Named(_, term) => {
					break term.normalize(limit - 1, cache);
				}
				Self::App(terms) => {
					match &mut terms[..] {
						[.., z, _y, _x, Self::S] => {
							z.normalize(limit - 1, cache);
							limit -= 1;
						}
						[.., _y, _x, Self::K] => {
							limit -= 1;
						}
						[.., Self::App(_) | Self::Named(_, _)] => {
							limit -= 1;
							self.reduce();
							continue;
						}
						_ => {
							break terms
								.iter_mut()
								.rev()
								.all(|term| term.normalize(limit - 1, cache));
						}
					}
				}
			}

			if !self.reduce() {
				//dbg!("normalized!");
				break true;
			}

			let name = format!("{}", self);
			//dbg!(&name);

			let new_bcl = self.bcl();

			if let Some(cell) = cache.get_mut(&new_bcl) {
				use CacheCell::*;
				match cell {
					Normal(term) => {
						let rc = term.clone();
						cache.insert(bcl.clone(), CacheCell::Normal(rc));
						break true;
					}
					Abnormal | Unsure => {
						*cell = Abnormal;
						break false;
					}
				};
			} else {
				cache.insert(new_bcl, CacheCell::Unsure);
			}
		};

		if normalized && !cache.contains_key(&bcl) {
			cache.insert(bcl, CacheCell::Normal(Rc::new(self.clone())));
		}

		//dbg!(&name, normalized);

		normalized
	}

	pub fn reduce(&mut self) -> bool {
		match self {
			Self::Named(_, inner) => {
				let old_inner = std::mem::replace(inner.as_mut(), Self::S);
				*self = old_inner;
				true
			}
			Self::App(terms) => match &mut terms[..] {
				[term] => {
					let old_t = std::mem::replace(term, Self::K);
					*self = old_t;
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
