use crate::primitive::{FromPrimitive, ToPrimitive, TryFromPrimitive};

/// Cast a value from a different type.
///
/// This corresponds to the `as` keyword in Rust.
///
/// This is currently implemented for all the following types which implement `ToPrimitive` and
/// `FromPrimitive`, which allows transparent conversion between them.
///
/// This means you are able to cast from [`Wrapping<u8>`] to [`Wrapping<u16>`] and [`u32`].
///
/// Unlike [`From`] this type does not have a blanked implementation for: `impl<T> From<T> for T`,
/// instead it chooses to implement a blanket implementation which utilizes [`ToPrimitive`] and
/// [`FromPrimitive`], as casting is a lossy process (corresponding to the `as` keyword), it only
/// makes logical sense to blanket implement it on these traits instead of an identity
/// implementation.
///
/// # Note
///
/// Unlike [`From`] and [`Into`], this trait has the following limitations, due to specialization
/// concerns:
/// * Implementing [`CastFrom`] does not automatically implement [`CastTo`].
/// * [`CastTo`] is not implemented on the identity.
///
/// # Example
///
/// ```
/// use std::num::{Saturating, Wrapping};
///
/// use numi::cast::CastTo;
/// # #[cfg(feature = "ordered-float-impl")]
/// use ordered_float::OrderedFloat;
///
/// let a = Wrapping(1u8);
/// let b: Wrapping<u16> = a.cast_to();
///
/// // we can even cast across types!
/// let a = Wrapping(1u8);
/// let b: Saturating<u16> = a.cast_to();
///
/// # #[cfg(feature = "ordered-float-impl")]
/// # fn ordered_float() {
/// // ... and even across crates!
/// // (this conversion requires the `ordered-float-impl` feature)
/// let a = Wrapping(1u8);
/// let b: OrderedFloat<f32> = a.cast_to();
/// # }
/// # #[cfg(feature = "ordered-float-impl")]
/// # ordered_float();
/// ```
pub trait CastFrom<T> {
    /// Cast a value from a different type.
    ///
    /// Might be lossy.
    fn cast_from(value: T) -> Self;
}

impl<T, U> CastFrom<U> for T
where
    U: ToPrimitive,
    T: FromPrimitive,
{
    fn cast_from(value: U) -> Self {
        T::from_primitive(value)
    }
}

/// Cast a value from a different type.
///
/// Fallible version of [`CastFrom`].
pub trait TryCastFrom<T>: Sized {
    /// The error type returned when the conversion fails.
    type Error;

    /// Cast a value from a different type.
    ///
    /// Might be lossy.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    fn try_cast_from(value: T) -> Result<Self, Self::Error>;
}

impl<T, U> TryCastFrom<U> for T
where
    U: ToPrimitive,
    T: TryFromPrimitive,
{
    type Error = T::Error;

    fn try_cast_from(value: U) -> Result<Self, Self::Error> {
        T::try_from_primitive(value)
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

/// Cast a value to as different type.
///
/// Fallible version of [`CastTo`].
///
/// This is the inverse of [`TryCastFrom`], and is implemented for all types that implement
/// [`TryCastFrom`].
pub trait TryCastTo<T>: Sized {
    /// The error type returned when the conversion fails.
    type Error;

    /// Cast a value to as different type.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    fn try_cast_to(self) -> Result<T, Self::Error>;
}

impl<T, U> TryCastTo<U> for T
where
    U: TryCastFrom<T>,
{
    type Error = U::Error;

    fn try_cast_to(self) -> Result<U, Self::Error> {
        U::try_cast_from(self)
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

    assert_impl_all!(Saturating<u16>: CastTo<u8>);
    assert_impl_all!(Saturating<u16>: CastTo<Wrapping<u32>>);

    #[test]
    const fn compile() {}
}
