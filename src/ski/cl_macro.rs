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
		$crate::ski::Combinator::Named(&stringify!($x), Box::new($x.clone()))
	};
	(($($r:tt)*)) => {
		combinator!($($r)*)
	};
	($($r:tt)*) => {{
		let mut combs = vec![$(combinator!($r)),*];
		combs.reverse();
		$crate::ski::Combinator::App(combs)
	}};
}
