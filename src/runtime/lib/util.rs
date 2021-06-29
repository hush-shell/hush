use super::{Float, Value};


/// A triple of numbers promoted to the same type.
#[derive(Debug)]
pub enum Numbers<const N: usize> {
	Ints([i64; N]),
	Floats([Float; N]),
}


impl<const N: usize> Numbers<N> {
	/// Promote numbers to float if necessary.
	pub fn promote(values: [Value; N]) -> Result<Self, Value> {
		let mut numbers = Numbers::Ints([0; N]);

		for ix in 0..N {
			match (&mut numbers, &values[ix]) {
				(Numbers::Ints(ints), Value::Int(int)) => ints[ix] = *int,

				(numbers @ Numbers::Ints(_), Value::Float(float)) => {
					let floats = numbers.to_floats();
					floats[ix] = float.copy();
				}

				(Numbers::Floats(floats), Value::Int(int)) => floats[ix] = int.into(),

				(Numbers::Floats(floats), Value::Float(float)) => floats[ix] = float.copy(),

				(_, value) => return Err(value.copy()),
			}
		}

		Ok(numbers)
	}


	fn to_floats(&mut self) -> &mut [Float; N] {
		match self {
			Numbers::Ints(ints) => {
				const ZERO: Float = Float(0.0);
				let mut floats = [ZERO; N];

				for ix in 0..N {
					floats[ix] = ints[ix].into();
				}

				*self = Numbers::Floats(floats);

				self.to_floats()
			}

			Numbers::Floats(floats) => floats,
		}
	}
}
