macro_rules! all_the_numbers {
    ($name:ident) => {
        $name!(u8);
        $name!(u16);
        $name!(u32);
        $name!(u64);
        $name!(u128);
        $name!(usize);

        $name!(i8);
        $name!(i16);
        $name!(i32);
        $name!(i64);
        $name!(i128);
        $name!(isize);

        $name!(f32);
        $name!(f64);
    };

    (@typed $name:ident) => {
        $name!(@int u8);
        $name!(@int u16);
        $name!(@int u32);
        $name!(@int u64);
        $name!(@int u128);
        $name!(@int usize);

        $name!(@int i8);
        $name!(@int i16);
        $name!(@int i32);
        $name!(@int i64);
        $name!(@int i128);
        $name!(@int isize);

        $name!(@float f32);
        $name!(@float f64);
    };
}

pub(crate) use all_the_numbers;
