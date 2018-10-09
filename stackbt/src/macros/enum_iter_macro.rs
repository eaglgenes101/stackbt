//! Copypaste of stackbt_macros/enum_iter_macro.rs which exists 
//! as a workaround to the inability to reexport macros. #[doc(hidden)]
#[macro_export]
macro_rules! first {
    (
        $name:ident ; $variant:ident 
    ) => {
        $name :: $variant
    };

    (
        $name:ident ; $variant:ident , $( $othervariants:ident ),*
    ) => {
        $name :: $variant
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_iter_from {
    (
        enum $itername:ident {
            $( $variant:ident ),*
        }
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[derive(::num_derive::ToPrimitive, ::num_derive::FromPrimitive)]
        enum $itername {
            $( $variant ),*
        }
    };

    (
        $visibility:vis enum $itername:ident {
            $( $variant:ident ),*
        }
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[derive(::num_derive::ToPrimitive, ::num_derive::FromPrimitive)]
        $visibility enum $itername {
            $( $variant ),*
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_iter_debug {
    (
        enum $name:ident {
            $( $variant:ident ),*
        }
    ) => {
        impl ::std::fmt::Debug for $name {
            fn fmt(&self, fmter: &mut ::std::fmt::Formatter) -> Result<(), 
                ::std::fmt::Error> 
            {
                match self {
                    $(
                        $name :: $variant (a) => fmter.write_fmt(
                            format_args!(
                            "{:?}({:?})", 
                                self.discriminant_of(),
                                a
                            )
                        )
                    ),*
                }
            }
        }
    }
}

/// Declarative macro for quickly and easily declaring an enum with an 
/// associated exhaustively enumerable discriminant type. 
/// 
/// The name for both the new fielded enum and its associated discriminant 
/// enumerator type must be legal, unused enum names, and its variants must 
/// all be valid enum variant names. 
/// 
/// From this, the macro will expand to the definition of two enums, the first
/// which is the enum to be made enumerable, and the second one, which is 
/// defined to be a fieldless enum with the same discriminant names and a 
/// trait impl which allows for it to be exhaustively enumerated. 
#[macro_export]
macro_rules! enum_iter {
    (
        $( #[ $mval:meta ] )*
        enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident ( $( $types:tt )* )
            ),*
        }
    ) => {
        $( #[ $mval ] )*
        enum $name {
            $(
                $( #[ $emval ] )*
                $variant ( $( $types )* )
            ),*
        }

        impl $name {
            fn discriminant_of(&self) -> $itername {
                match self {
                    $( $name :: $variant (_) => $itername :: $variant ),*
                }
            }
        }

        enum_iter_debug!(
            enum $name {
                $( $variant ),*
            }
        );

        enum_iter_from!(
            enum $itername {
                $( $variant ),*
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        $visibility:vis enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident ( $( $types:tt )* )
            ),*
        }
    ) => {
        $( #[ $mval ] )*
        $visibility enum $name {
            $(
                $( #[ $emval ] )*
                $variant ( $( $types )* )
            ),*
        }

        impl $name {
            $visibility fn discriminant_of(&self) -> $itername {
                match self {
                    $( $name :: $variant (_) => $itername :: $variant ),*
                }
            }
        }

        enum_iter_debug!(
            enum $name {
                $( $variant ),*
            }
        );

        enum_iter_from!(
            $visibility enum $itername {
                $( $variant ),*
            }
        );
    };
}

#[cfg(test)]
mod tests {

    use num_traits::FromPrimitive;

    enum_iter!(
        pub enum Foo: Bar {
            Baz(i64), 
            Quux(i64)
        }
    );

    #[test]
    fn bar_iter_test() {
        let a = Foo::Baz(0);
        let b = Foo::Quux(1);
        assert_eq!(Bar::from_u64(0), Option::Some(Bar::Baz));
        assert_eq!(Bar::from_u64(1), Option::Some(Bar::Quux));
        assert_eq!(Bar::from_u64(2), Option::None);
        assert_eq!(a.discriminant_of(), Bar::Baz);
        assert_eq!(b.discriminant_of(), Bar::Quux);
    }
}