macro_rules! id_newtype {
    ($name:ident) => {
        use core::num::NonZeroU64;

        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Debug)]
        pub struct $name(NonZeroU64);
        impl $name {
            pub fn new(raw: u64) -> Option<Self> {
                NonZeroU64::new(raw).map(Self)
            }
            pub fn get(self) -> u64 {
                self.0.get()
            }
        }
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                // base36 for compact human form
                write!(f, "{}", radix_fmt::radix(self.get(), 36))
            }
        }
        impl core::str::FromStr for $name {
            type Err = core::num::ParseIntError;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                let v = u64::from_str_radix(s, 36)?;
                $name::new(v).ok_or_else(|| "zero id".parse::<core::num::NonZeroU64>().unwrap_err())
            }
        }
    };
}

pub(crate) use id_newtype;
