use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Combinator {
	S,
	K,
	App(Vec<Combinator>),
	Var(char),
}

impl Combinator {
	pub fn reduced(&self) -> Option<Self> {
		match self {
			Self::App(args) => match &args[..] {
				[Self::K, x, _y, rest @ ..] => {
					let x = x.clone();
					let new_comb = [&[x], rest].concat();
					Some(Self::App(new_comb))
				}
				[Self::S, f, g, x, rest @ ..] => {
					let f = f.clone();
					let new_comb = [&[f, x.clone(), g.of(x)], rest].concat();
					Some(Self::App(new_comb))
				}
				[Self::App(args), rest @ ..] => {
					let mut new_comb = args.clone();
					new_comb.extend_from_slice(rest);
					Some(Self::App(new_comb))
				}
				_ => None,
			},
			_ => None,
		}
	}

	pub fn k_reduced(&self) -> Self {
		match self {
			Self::App(args) => match &args[..] {
				[Self::K, x, _y, rest @ ..] => {
					let new_comb = [x.clone()]
						.iter()
						.chain(rest.iter())
						.map(|c| c.k_reduced())
						.collect::<Vec<_>>();
					Self::App(new_comb)
				}
				_ => {
					let new_comb = args.iter().map(|c| c.k_reduced()).collect::<Vec<_>>();
					Self::App(new_comb)
				}
			},
			_ => self.clone(),
		}
	}

	pub fn of(&self, x: &Combinator) -> Combinator {
		match self {
			Self::App(v) => {
				let mut left = v.clone();
				left.push(x.clone());
				Self::App(left)
			}
			_ => Self::App(vec![self.clone(), x.clone()]),
		}
	}

	pub fn size(&self) -> usize {
		match self {
			Self::K | Self::S | Self::Var(_) => 1,
			Self::App(args) => args.iter().map(Combinator::size).sum(),
		}
	}
}

impl Display for Combinator {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::S => write!(fmt, "S"),
			Self::K => write!(fmt, "K"),
			Self::Var(var) => write!(fmt, "{}", var),
			Self::App(combs) => {
				for comb in combs {
					if comb.size() <= 1 {
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
