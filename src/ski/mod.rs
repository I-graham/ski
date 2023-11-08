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
	($x:ident) => {
		$x.rc()
	};
	($a:tt $b:tt) => {
		combinator!($a).rc().on(&combinator!($b).rc())
	};
	(($a:tt $b:tt)) => {
		combinator!($a).rc().on(&combinator!($b).rc())
	};
	($a:tt $b:tt $($r:tt)+) => {
		combinator!( ($a $b) $($r)+ )
	};
	(($a:tt $b:tt $($r:tt)+)) => {
		combinator!( ($a $b) $($r)+ )
	};
}
