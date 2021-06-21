use std::{
	cmp::Ordering,
	hash::{Hash, Hasher},
};

use gc::{Finalize, Trace};


#[derive(Debug, Clone)]
#[derive(Trace, Finalize)]
pub struct Float(pub f64);


impl Float {
	pub fn negate(&self) -> Float {
		Float(- self.0)
	}
}


impl PartialEq for Float {
	fn eq(&self, other: &Self) -> bool {
		match (self.0.is_nan(), other.0.is_nan()) {
			(false, false) => self.0 == other.0,
			(true, true) => true,
			_ => false,
		}
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
		match (self.0.is_nan(), other.0.is_nan()) {
			(false, false) => self.0
				.partial_cmp(&other.0)
				.expect("non-nan float comparison failed"),
			(false, true) => Ordering::Greater,
			(true, false) => Ordering::Less,
			(true, true) => Ordering::Equal,
		}
	}
}


impl Hash for Float {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.to_bits().hash(state)
	}
}


impl From<f64> for Float {
	fn from(f: f64) -> Self {
		Self(f)
	}
}
