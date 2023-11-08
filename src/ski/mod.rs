pub mod combinator;

pub use combinator::*;

#[macro_export]
macro_rules! combinator {
	(S) => {
		$crate::ski::Combinator::S.rc()
	};
	(K) => {
		$crate::ski::Combinator::K.rc()
	};
	($x:ident) => {
		$x
	};
	(($a:tt $b:tt)) => {
		combinator!($a).on(&combinator!($b))
	};
	($a:tt $b:tt) => {
		combinator!($a).on(&combinator!($b))
	};
}
