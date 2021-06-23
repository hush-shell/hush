/// Implement std::ops for the given type.
macro_rules! op_impl {
	($type: ident, unary, $trait: ident, $method: ident) => {
		impl $trait for $type {
			type Output = Self;

			fn $method(self) -> Self::Output {
				Self(self.0.$method())
			}
		}

		impl $trait for &$type {
			type Output = $type;

			fn $method(self) -> Self::Output {
				$type(self.0.$method())
			}
		}
	};

	($type: ident, binary, $trait: ident, $method: ident) => {
		impl $trait for $type {
			type Output = Self;

			fn $method(self, rhs: Self) -> Self::Output {
				Self(self.0.$method(rhs.0))
			}
		}

		impl $trait for &$type {
			type Output = $type;

			fn $method(self, rhs: Self) -> Self::Output {
				$type(self.0.$method(rhs.0))
			}
		}
	}
}
