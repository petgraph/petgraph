use core::num::{Saturating, Wrapping};

/// Cast a value from a different type.
///
/// This corresponds to the `as` keyword in Rust.
pub trait CastFrom<T> {
    fn cast_from(value: T) -> Self;
}

impl<T> CastFrom<T> for T {
    fn cast_from(value: T) -> Self {
        value
    }
}

/// Cast a value to as different type.
///
/// This corresponds to the `as` keyword in Rust.
///
/// This is the inverse of [`CastFrom`], and is implemented for all types that implement
/// [`CastFrom`].
/// You should prefer to implement [`CastFrom`] instead of this trait.
pub trait CastTo<T> {
    fn cast_to(self) -> T;
}

impl<T, U> CastTo<U> for T
where
    U: CastFrom<T>,
{
    fn cast_to(self) -> U {
        U::cast_from(self)
    }
}

macro_rules! impl_cast_from {
    (@impl $from:ty, $to:ty) => {
        impl CastFrom<$to> for $from {
            fn cast_from(value: $to) -> Self {
                value as Self
            }
        }
    };
    (@proxy $from:ty; $($to:ty),* $(,)?) => {
        $(impl_cast_from!(@impl $from, $to);)*
    };
    (@dist $head:tt; @() @( $($prev:tt)* )) => {
        impl_cast_from!(@proxy $head; $($prev),*);
    };
    (@dist $head:tt; @( $next:tt $($tail:tt)*) @( $($prev:tt)* )) => {
        impl_cast_from!(@dist $next; @( $($tail)* ) @( $($prev)* $head ));
        impl_cast_from!(@proxy $head; $($prev ,)* $next , $($tail ,)*);
    };
    ($first:tt $(, $tail:tt)* $(,)?) => {
        impl_cast_from!(@dist $first; @($($tail)*) @());
    };
}

impl_cast_from!(
    u8, u16, u32, u64, u128, usize, // unsigned integers
    i8, i16, i32, i64, i128, isize, // signed integers
    f32, f64, // floating point numbers
);

// I'd love to have some sort of `Wrapping<u16>::cast_from(2u8)`, but I don't really see how to do
// that without overlapping impls.
impl<T> CastFrom<T> for Wrapping<T> {
    fn cast_from(value: T) -> Self {
        Self(value)
    }
}

impl<T> CastFrom<Wrapping<T>> for T {
    fn cast_from(value: Wrapping<T>) -> Self {
        value.0
    }
}

impl<T> CastFrom<T> for Saturating<T> {
    fn cast_from(value: T) -> Self {
        Self(value)
    }
}

impl<T> CastFrom<Saturating<T>> for T {
    fn cast_from(value: Saturating<T>) -> Self {
        value.0
    }
}

#[cfg(test)]
mod test {
    use core::num::{Saturating, Wrapping};

    use static_assertions::assert_impl_all;

    use crate::cast::CastTo;

    // can we do normal `as`?
    assert_impl_all!(u8: CastTo<u16>);
    assert_impl_all!(u16: CastTo<u8>);
    assert_impl_all!(u16: CastTo<Wrapping<u16>>);

    // can we do `as` with wrapping?
    assert_impl_all!(Wrapping<u16>: CastTo<u16>);
    assert_impl_all!(u16: CastTo<Wrapping<u16>>);

    // can we do `as` with saturating?
    assert_impl_all!(u16: CastTo<Saturating<u16>>);
    assert_impl_all!(Saturating<u16>: CastTo<u16>);

    #[test]
    fn compile() {}
}
