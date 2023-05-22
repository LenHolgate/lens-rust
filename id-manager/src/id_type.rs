pub trait IdType where Self: Ord + std::ops::Add<Self, Output=Self> + std::ops::Sub<Self, Output=Self> + Sized + num::One + std::fmt::Display + Copy
{
    const MAX: Self;
    const MIN: Self;
}

macro_rules! id_type_trait_impl {
    ($name:ident for $($t:ty)*) => ($(
    impl $name for $t {
        const MAX : $t = <$t>::MAX;
        const MIN : $t = <$t>::MIN;
    }
    )*)
}

id_type_trait_impl!(IdType for u8 u16 u32 u64 u128 usize);
