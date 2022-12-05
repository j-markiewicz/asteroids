/// Absolute value
pub trait Abs {
	fn abs(self) -> Self;
}

macro_rules! impl_abs {
	($( $t:ty ),*) => {
		$(
			impl Abs for $t {
				fn abs(self) -> Self {
					self.abs()
				}
			}
		)*
	}
}

macro_rules! impl_abs_nothing {
	($( $t:ty ),*) => {
		$(
			impl Abs for $t {
				fn abs(self) -> Self {
					self
				}
			}
		)*
	}
}

impl_abs!(f32, f64);
impl_abs!(i8, i16, i32, i64, i128);
impl_abs_nothing!(u8, u16, u32, u64, u128);

/// Return the operand, whose absolute value is larger
pub fn abs_max<T: PartialOrd + Abs + Clone>(a: T, b: T) -> T {
	if a.clone().abs() >= b.clone().abs() {
		a
	} else {
		b
	}
}
