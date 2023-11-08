use std::fmt::Display;
use std::sync::{Arc, OnceLock};

use crate::combinator;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Combinator {
	S,
	K,
	On(Arc<Combinator>, Arc<Combinator>),
}

impl Combinator {
	pub fn rc(&self) -> Arc<Self> {
		static RC_S: OnceLock<Arc<Combinator>> = OnceLock::new();
		static RC_K: OnceLock<Arc<Combinator>> = OnceLock::new();

		match self {
			Self::S => RC_S.get_or_init(|| Arc::new(Self::S)).clone(),
			Self::K => RC_K.get_or_init(|| Arc::new(Self::K)).clone(),
			_ => Arc::new(self.clone()),
		}
	}

	pub fn reduce(self: &Arc<Self>) -> Option<Arc<Self>> {
		let Self::On(a, z) = &**self else {
			return None;
		};

		if let Some(red) = a.reduce() {
			return Some(red.on(z).rc());
		}

		if let Some(red) = a.reduce() {
			return Some(red.on(z).rc());
		}

		let Self::On(cx, y) = &**a else {
			return None;
		};

		match &**cx {
			Self::K => Some(y.clone()),
			Self::On(s, x) if **s == Self::S => Some(combinator!(x z (y z)).rc()),
			_ => None,
		}
	}

	pub fn on(self: &Arc<Self>, x: &Arc<Combinator>) -> Combinator {
		Self::On(self.clone(), x.clone())
	}

	pub fn size(self: &Arc<Self>) -> usize {
		match &**self {
			Self::K | Self::S => 1,
			Self::On(f, x) => f.size() + x.size(),
		}
	}
}

impl Display for Combinator {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::S => write!(fmt, "S"),
			Self::K => write!(fmt, "K"),
			Self::On(f, x) => {
				write!(fmt, "{}", f)?;

				if x.size() > 1 {
					write!(fmt, "({})", x)
				} else {
					write!(fmt, "{}", x)
				}
			}
		}
	}
}
