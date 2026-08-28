/// A macro that wraps a shared asset enum type and implements the asset trait for each variant.
#[macro_export]
macro_rules! define_asset_type {
    (
        $(#[$meta:meta])*
        $vis:vis enum $enum_name:ident {
            $($variant:ident($inner:ty)),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $enum_name {
            $($variant($inner),)*
        }

        impl $crate::Asset for $enum_name {
            type Hash = u64;

            fn name(&self) -> String {
                match self {
                    $($enum_name::$variant(inner) => inner.name(),)*
                }
            }

            fn type_name(&self) -> &'static str {
                match self {
                    $($enum_name::$variant(inner) => inner.type_name(),)*
                }
            }

            fn color(&self) -> $crate::Color {
                match self {
                    $($enum_name::$variant(inner) => inner.color(),)*
                }
            }

            fn status(&self) -> &$crate::AssetStatus {
                match self {
                    $($enum_name::$variant(inner) => inner.status(),)*
                }
            }

            fn hash(&self) -> Self::Hash {
                match self {
                    $($enum_name::$variant(inner) => inner.hash(),)*
                }
            }

            fn info(&self) -> String {
                match self {
                    $($enum_name::$variant(inner) => inner.info(),)*
                }
            }

            fn search(&self) -> $crate::SearchAsset {
                match self {
                    $($enum_name::$variant(inner) => inner.search(),)*
                }
            }

            fn is_newer_than(&self, other: &Self) -> bool {
                match (self, other) {
                    $(($enum_name::$variant(inner), $enum_name::$variant(other)) => inner.is_newer_than(other),)*
                    _ => {
                        #[cfg(debug_assertions)]
                        {
                            let name = self.name();
                            let type_name = self.type_name();
                            let name_other = other.name();
                            let type_name_other = other.type_name();

                            println!("Tried to compare two different asset types: {name:?}:{type_name:?} == {name_other:?}:{type_name_other:?}");
                        }
                        false
                    }
                }
            }
        }

        $(
            impl From<$inner> for $enum_name {
                fn from(value: $inner) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}
