use core::num::{Saturating, Wrapping};

/// Cast a value from a different type.
///
/// This corresponds to the `as` keyword in Rust.
///
/// This is currently implemented for all the following types:
/// * primitive numeric types (e.g. `u8`, `i32`, `f64`)
/// * [`Wrapping<T>`]
/// * [`Saturating<T>`]
///
/// where `T` is a primitive numeric type.
///
/// This means you are able to cast from [`Wrapping<u8>`] to [`Wrapping<u16>`] and [`u32`].
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
    (@impl {$from:tt<$from_inner:ty>}, {$to:tt<$to_inner:ty>}) => {
        impl CastFrom<$to<$to_inner>> for $from<$from_inner> {
            fn cast_from(value: $to<$to_inner>) -> Self {
                Self(value.0 as $from_inner)
            }
        }
    };
    (@impl {$from:tt}, {$to:tt<$inner:ty>}) => {
        impl CastFrom<$to<$inner>> for $from {
            fn cast_from(value: $to<$inner>) -> Self {
                value.0 as Self
            }
        }
    };
    (@impl {$from:tt<$inner:ty>}, {$to:tt}) => {
        impl CastFrom<$to> for $from<$inner> {
            fn cast_from(value: $to) -> Self {
                Self(value as $inner)
            }
        }
    };
    (@impl {$from:tt}, {$to:tt}) => {
        impl CastFrom<$to> for $from {
            fn cast_from(value: $to) -> Self {
                value as Self
            }
        }
    };
    (@proxy $from:tt; $($to:tt),* $(,)?) => {
        $(impl_cast_from!(@impl $from, $to);)*
    };
    (@dist $head:tt; @() @( $($prev:tt)* )) => {
        impl_cast_from!(@proxy $head; $($prev),*);
    };
    (@dist $head:tt; @( $next:tt $(, $tail:tt)*) @( $($prev:tt)* )) => {
        impl_cast_from!(@dist $next; @( $($tail),* ) @( $($prev)* $head ));
        impl_cast_from!(@proxy $head; $($prev ,)* $next , $($tail ,)*);
    };
    (@dispatch $first:tt $(, $tail:tt)*) => {
        impl_cast_from!(@dist $first; @($($tail),*) @());
    };
    (@flatten [$($primitive:tt),*] [] @( $($finish:tt),* )) => {
        impl_cast_from!(@dispatch $($finish),*);
    };
    (@flatten [$($primitive:tt),*] [$next:tt $(, $newtype:tt)*] @( $($finish:tt),* )) => {
        impl_cast_from!(@flatten [$($primitive),*] [$($newtype),*] @($($finish),* $(, { $next<$primitive> } )*));
    };
    (Context {primitives: [$($primitive:tt),*], newtypes: [$($newtype:tt),*]}) => {
        impl_cast_from!(@flatten [$($primitive),*] [$($newtype),*] @($({ $primitive }),*));
    };
}

// TODO: I'd love to solve this generically, but that doesn't seem to be possible without
// overlapping impls or specialization.
// (specifically the newtypes)
impl_cast_from!(Context {
    primitives: [
        u8, u16, u32, u64, u128, usize, // unsigned integers
        i8, i16, i32, i64, i128, isize, // signed integers
        f32, f64 // floating point numbers
    ],
    newtypes: [Wrapping, Saturating]
});

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

    assert_impl_all!(Saturating<u16>: CastTo<u8>);
    assert_impl_all!(Saturating<u16>: CastTo<Wrapping<u32>>);

    #[test]
    fn compile() {}
}
