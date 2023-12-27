//! Conversion of arbitrary types to primitive types.
//!
//! This module is a collection of traits used to convert arbitrary types to primitive types.
//!
//! You usually don't want to use this, instead you should use [`numi::cast::CastTo`] or
//! [`numi::cast::TryCastTo`] which are more ergonomic and provide the ability to cast between
//! different types that can be converted from primitive types.
use core::{
    convert::Infallible,
    num::{Saturating, Wrapping},
};

#[cfg(feature = "ordered-float-impl")]
use ordered_float::{NotNan, OrderedFloat};

pub trait Primitive: Copy {
    // sadly we cannot have something like `from<T> where T: Primitive` because we cannot limit the
    // implementations. Therefore don't know what we can expect and therefore cannot `as` them.
    // We could in theory due this through sealing of the type, but this is something I'd like to
    // avoid.

    fn into_u8(self) -> u8;
    fn into_u16(self) -> u16;
    fn into_u32(self) -> u32;
    fn into_u64(self) -> u64;
    fn into_u128(self) -> u128;
    fn into_usize(self) -> usize;

    fn into_i8(self) -> i8;
    fn into_i16(self) -> i16;
    fn into_i32(self) -> i32;
    fn into_i64(self) -> i64;
    fn into_i128(self) -> i128;
    fn into_isize(self) -> isize;

    fn into_f32(self) -> f32;
    fn into_f64(self) -> f64;

    // fn into_bool(self) -> bool;
    // fn into_char(self) -> char;
}

struct Proxy<T>(T);

// proxy removes the `into_` prefix, makes it possible to more easily use in macros
impl<T> Proxy<T>
where
    T: Primitive,
{
    fn u8(self) -> u8 {
        self.0.into_u8()
    }

    fn u16(self) -> u16 {
        self.0.into_u16()
    }

    fn u32(self) -> u32 {
        self.0.into_u32()
    }

    fn u64(self) -> u64 {
        self.0.into_u64()
    }

    fn u128(self) -> u128 {
        self.0.into_u128()
    }

    fn usize(self) -> usize {
        self.0.into_usize()
    }

    fn i8(self) -> i8 {
        self.0.into_i8()
    }

    fn i16(self) -> i16 {
        self.0.into_i16()
    }

    fn i32(self) -> i32 {
        self.0.into_i32()
    }

    fn i64(self) -> i64 {
        self.0.into_i64()
    }

    fn i128(self) -> i128 {
        self.0.into_i128()
    }

    fn isize(self) -> isize {
        self.0.into_isize()
    }

    fn f32(self) -> f32 {
        self.0.into_f32()
    }

    fn f64(self) -> f64 {
        self.0.into_f64()
    }
}

macro_rules! impl_primitive {
    ($ty:ident) => {
        impl Primitive for $ty {
            fn into_u8(self) -> u8 {
                self as u8
            }

            fn into_u16(self) -> u16 {
                self as u16
            }

            fn into_u32(self) -> u32 {
                self as u32
            }

            fn into_u64(self) -> u64 {
                self as u64
            }

            fn into_u128(self) -> u128 {
                self as u128
            }

            fn into_usize(self) -> usize {
                self as usize
            }

            fn into_i8(self) -> i8 {
                self as i8
            }

            fn into_i16(self) -> i16 {
                self as i16
            }

            fn into_i32(self) -> i32 {
                self as i32
            }

            fn into_i64(self) -> i64 {
                self as i64
            }

            fn into_i128(self) -> i128 {
                self as i128
            }

            fn into_isize(self) -> isize {
                self as isize
            }

            fn into_f32(self) -> f32 {
                self as f32
            }

            fn into_f64(self) -> f64 {
                self as f64
            }
        }

        impl FromPrimitive for $ty {
            fn from_primitive<T>(value: T) -> Self
            where
                T: ToPrimitive,
            {
                Proxy(value.into_primitive()).$ty()
            }
        }
    };
    ($($ty:ident),*) => {
        $(impl_primitive!($ty);)*
    };
}

impl_primitive!(
    u8, u16, u32, u64, u128, usize, // unsigned integers
    i8, i16, i32, i64, i128, isize, // signed integers
    f32, f64 // floating point numbers
);

pub trait ToPrimitive {
    type Primitive: Primitive;

    fn as_primitive(&self) -> Self::Primitive;
    fn into_primitive(self) -> Self::Primitive;
}

impl<T> ToPrimitive for T
where
    T: Primitive,
{
    type Primitive = T;

    fn as_primitive(&self) -> Self::Primitive {
        *self
    }

    fn into_primitive(self) -> Self::Primitive {
        self
    }
}

pub trait FromPrimitive {
    fn from_primitive<T>(value: T) -> Self
    where
        T: ToPrimitive;
}

pub trait TryFromPrimitive: Sized {
    type Error;

    /// Try to convert a primitive value to `Self`.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    fn try_from_primitive<T>(value: T) -> Result<Self, Self::Error>
    where
        T: ToPrimitive;
}

impl<T> TryFromPrimitive for T
where
    T: FromPrimitive,
{
    // once never type is stable, we can use `!` instead of `Infallible`
    type Error = Infallible;

    fn try_from_primitive<U>(value: U) -> Result<Self, Self::Error>
    where
        U: ToPrimitive,
    {
        Ok(Self::from_primitive(value))
    }
}

macro_rules! impl_newtype {
    ($name:ident<$(|)? $($inner:ident)|*>) => {
        $(
        impl FromPrimitive for $name<$inner>
        {
            fn from_primitive<T>(value: T) -> Self where T: ToPrimitive {
                Self(Proxy(value.into_primitive()).$inner())
            }
        }
        )*

        $(
        impl ToPrimitive for $name<$inner>
        {
            type Primitive = $inner;

            fn as_primitive(&self) -> Self::Primitive {
                self.0
            }

            fn into_primitive(self) -> Self::Primitive {
                self.0
            }
        }
        )*
    };
}

impl_newtype!(Wrapping<
    | u8 | u16 | u32 | u64 | u128 | usize
    | i8 | i16 | i32 | i64 | i128 | isize
    | f32 | f64
    >);

impl_newtype!(Saturating<
    | u8 | u16 | u32 | u64 | u128 | usize
    | i8 | i16 | i32 | i64 | i128 | isize
    | f32 | f64
    >);

#[cfg(feature = "ordered-float-impl")]
impl_newtype!(OrderedFloat<
    | f32 | f64
    >);

#[cfg(feature = "ordered-float-impl")]
impl TryFromPrimitive for NotNan<f32> {
    type Error = ordered_float::FloatIsNan;

    fn try_from_primitive<T>(value: T) -> Result<Self, Self::Error>
    where
        T: ToPrimitive,
    {
        let value = value.into_primitive().into_f32();

        Self::new(value)
    }
}

#[cfg(feature = "ordered-float-impl")]
impl TryFromPrimitive for NotNan<f64> {
    type Error = ordered_float::FloatIsNan;

    fn try_from_primitive<T>(value: T) -> Result<Self, Self::Error>
    where
        T: ToPrimitive,
    {
        let value = value.into_primitive().into_f64();

        Self::new(value)
    }
}
