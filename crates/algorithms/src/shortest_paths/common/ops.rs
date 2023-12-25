use core::{
    num::{Saturating, Wrapping},
    ops::Add,
};

// proxy trait to not need to carry around where clause.
/// Automatically implemented trait for adding a reference to a reference.
///
/// This trait is implemented for all types that implement `Add<&T, Output = T>` on `&T`.
///
/// This is a helper trait that shouldn't need to be implemented directly.
/// The reason for it's existence is due to limitations in the Rust trait system.
/// To have a similar bound on a trait like [`BellmanFordMeasure`] one would need to use
/// `where for<'a> &'a Self: Add<&'a Self, Output = Self>`, but that means that this where clause
/// would need to be repeated every time the trait is used.
///
/// This limitation is known as [`implied bounds`](https://github.com/rust-lang/rust/issues/44491).
pub trait AddRef<Rhs = Self> {
    type Output;

    fn add_ref(&self, rhs: &Rhs) -> Self::Output;
}

impl<T> AddRef for T
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    type Output = T;

    fn add_ref(&self, rhs: &T) -> Self::Output {
        self + rhs
    }
}

pub trait CastTo<T> {
    fn cast_to(self) -> T;
}

impl<T> CastTo<T> for T {
    fn cast_to(self) -> T {
        self
    }
}

macro_rules! impl_cast_to {
    (@impl $from:ty, $to:ty) => {
        impl CastTo<$to> for $from {
            fn cast_to(self) -> $to {
                self as $to
            }
        }
    };
    (@proxy $from:ty; $($to:ty),* $(,)?) => {
        $(impl_cast_to!(@impl $from, $to);)*
    };
    (@dist $head:tt; @() @( $($prev:tt)* )) => {
        impl_cast_to!(@proxy $head; $($prev),*);
    };
    (@dist $head:tt; @( $next:tt $($tail:tt)*) @( $($prev:tt)* )) => {
        impl_cast_to!(@dist $next; @( $($tail)* ) @( $($prev)* $head ));
        impl_cast_to!(@proxy $head; $($prev ,)* $next , $($tail ,)*);
    };
    ($first:tt $(, $tail:tt)* $(,)?) => {
        impl_cast_to!(@dist $first; @($($tail)*) @());
    };
}

impl_cast_to!(
    u8, u16, u32, u64, u128, usize, // unsigned integers
    i8, i16, i32, i64, i128, isize, // signed integers
    f32, f64, // floating point numbers
);

impl<T> CastTo<T> for Wrapping<T> {
    fn cast_to(self) -> T {
        self.0
    }
}

impl<T> CastTo<Wrapping<T>> for T {
    fn cast_to(self) -> Wrapping<T> {
        Wrapping(self)
    }
}

impl<T> CastTo<T> for Saturating<T> {
    fn cast_to(self) -> T {
        self.0
    }
}

impl<T> CastTo<Saturating<T>> for T {
    fn cast_to(self) -> Saturating<T> {
        Saturating(self)
    }
}
