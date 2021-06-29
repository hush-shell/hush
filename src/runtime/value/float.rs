use std::{
	cmp::Ordering,
	hash::{Hash, Hasher},
	ops::{Add, Sub, Mul, Div, Rem, Neg},
};

use gc::{Finalize, Trace};


/// Hush's float type.
/// This type supports full ordering and hashing.
/// NaN is lower and different than every other value, including itself, but the hash is
/// the same for all NaN values.
#[derive(Debug, Default, Clone)]
#[derive(Trace, Finalize)]
pub struct Float(pub f64);


impl Float {
	/// Shallow copy.
	pub fn copy(&self) -> Self {
		Self(self.0)
	}


	/// Check if the float is not a number.
	pub fn is_nan(&self) -> bool {
		self.0.is_nan()
	}
}


impl PartialEq for Float {
	fn eq(&self, other: &Self) -> bool {
		!self.is_nan()
			&& !other.is_nan()
			&& self.0 == other.0
	}
}


impl Eq for Float { }


impl PartialOrd for Float {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}


impl Ord for Float {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self.is_nan(), other.is_nan()) {
			(true, _) => Ordering::Less,
			(false, true) => Ordering::Greater,
			(false, false) => self.0
				.partial_cmp(&other.0)
				.expect("non-nan float comparison failed"),
		}
	}
}


impl Hash for Float {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let float =
			if self.is_nan() {
				f64::NAN // Make sure that the hash equals for all NaN values.
			} else {
				self.0
			};

		float.to_bits().hash(state)
	}
}


impl From<f64> for Float {
	fn from(f: f64) -> Self {
		Self(f)
	}
}


impl From<i64> for Float {
	fn from(int: i64) -> Self {
		Self(int as f64)
	}
}


impl From<&i64> for Float {
	fn from(int: &i64) -> Self {
		Self(*int as f64)
	}
}


op_impl!(Float, unary, Neg, neg);
op_impl!(Float, binary, Add, add);
op_impl!(Float, binary, Sub, sub);
op_impl!(Float, binary, Mul, mul);
op_impl!(Float, binary, Div, div);
op_impl!(Float, binary, Rem, rem);
