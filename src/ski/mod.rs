pub mod combinator;

pub use combinator::*;

#[macro_export]
macro_rules! combinator {
	(S) => {
		$crate::ski::Combinator::S
	};
	(K) => {
		$crate::ski::Combinator::K
	};
	($x:literal) => {
		$crate::ski::Combinator::Var($x)
	};
	($x:ident) => {
		$x.clone()
	};
	(($($r:tt)+)) => {
		combinator!($($r)+)
	};
	($($r:tt)+) => {
		$crate::ski::Combinator::App(vec![$(combinator!($r),)+])
	};
}
