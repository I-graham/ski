use std::fmt::Display;
use std::sync::{Arc, OnceLock};

#[derive(Clone, PartialEq, Eq)]
pub enum Combinator {
	S,
	K,
	A(Arc<Combinator>, Arc<Combinator>),
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

	pub fn reduce(self: &Arc<Self>) -> Arc<Self> {
		let Self::A(ref cx, ref y) = **self else {
			return self.clone();
		};

		let Self::A(ref c, ref x) = **cx else {
			return self.clone();
		};

		match &**c {
			Self::K => x.rc(),
			Self::A(ref s, ref z) if **s == Self::S => {
				let func = z.on(y);
				let arg = x.on(y);
				func.on(&arg).rc()
			}
			_ => self.clone(),
		}
	}

	pub fn on(self: &Arc<Self>, x: &Arc<Combinator>) -> Arc<Combinator> {
		Self::A(self.clone(), x.clone()).rc()
	}

	pub fn size(self: &Arc<Self>) -> usize {
		match &**self {
			Self::K | Self::S => 1,
			Self::A(f, x) => f.size() + x.size(),
		}
	}
}

impl Display for Combinator {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::S => write!(fmt, "S"),
			Self::K => write!(fmt, "K"),
			Self::A(f, x) => {
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
