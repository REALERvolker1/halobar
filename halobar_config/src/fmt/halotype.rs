/// Methods that a type can use to expedite formatting.
pub trait Truthy: std::fmt::Display {
    /// Determine if this variable is "truthy" or "falsy"
    fn is_truthy(&self) -> bool;
}

macro_rules! mass_impl {
    (@int $( $ty:ty ),+$(,)?) => {
        $(
            impl Truthy for $ty {
                #[inline(always)]
                fn is_truthy(&self) -> bool {
                    *self != 0
                }
            }
        )+
    };
    (@float $( $ty:ty ),+$(,)?) => {
        $(
            impl Truthy for $ty {
                #[inline(always)]
                fn is_truthy(&self) -> bool {
                    *self != 0.0
                }
            }
        )+
    };
    (@str $( $ty:ty ),+$(,)?) => {
        $(
            impl Truthy for $ty {
                #[inline(always)]
                fn is_truthy(&self) -> bool {
                    !self.is_empty()
                }
            }
        )+
    };
}

mass_impl![@int i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize];
mass_impl![@float f32, f64];
mass_impl![@str String, str];

impl Truthy for bool {
    #[inline(always)]
    fn is_truthy(&self) -> bool {
        *self
    }
}
