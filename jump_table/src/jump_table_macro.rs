macro_rules! jump_table_display {
    (
        $name:ident {
            $( $variant:ident ),*
        }
    ) => {
        use std::fmt::{Error, Formatter, Display};

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                let disp_str = match self {
                    $( $variant => stringify!( $variant ) ),*
                };
                f.write_str(disp_str)
            }
        }
    };
}

macro_rules! jump_table_from {
    (
        $name:ident : $fntype:ty {
            $( $variant:ident = $value:expr ),*
        }
    ) => {
        use std::convert::From;
        use $crate::jump_table_traits::JumpTable;

        impl From< $name > for $fntype {
            fn from( val: $name ) -> Self {
                match val {
                    $( $variant => $value ),*
                }
            }
        }

        impl JumpTable< $fntype > for $name {}
    };
}

macro_rules! jump_table_main {
    (
        $( #[ $mval:meta ] ) *
        ( $( $vis:tt )* ) $name:ident : $fntype:ty {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident = $value:path 
            ) , *
        }
    ) => {
        $( #[ $mval ] ) *
        #[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
        $( $vis:tt )* enum $name {
            $( 
                $( #[ $emval ] )*
                $variant 
            ),*
        }

        jump_table_display!(
            $name {
                $( $variant ),*
            }
        );

        jump_table_from!(
            $name : $fntype {
                $( $variant = $value ),*
            }
        );
    };
}

/// Declarative macro for generating jump tables. This macro expects to 
/// enclose a statement with form similar to the following: 
/// ```ignore
/// #[attribute_0]
/// #[attribute_1]
/// #[attribute_2]
/// enum EnumName: fn (ArgType0, ArgType1, ArgType2) -> RetType {
///     Variant0 = fn_name_0,
///     #[variant_attribute_0]
///     #[variant_attribute_1]
///     #[variant_attribute_2]
///     Variant1 = fn_name_1,
///     Variant2 = fn_name_2
/// }
/// ```
/// All the functions named in the macro must have the type declared after 
/// the enum name. 
/// 
/// From this, the macro will generate a fieldless enum with the given enum 
/// name and enum variants, along with derivations of traits for the enum, 
/// including one which allows conversion of the enum to the named function 
/// type. 
#[macro_export]
macro_rules! jump_table {
    (
        $( #[ $mval:meta ] )*
        enum $name:ident : fn ( $($argtype:ty),* ) -> $rettype:ty {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident = $value:path 
            ),*
        }
    ) => {
        jump_table_main!(
            $( #[ $mval:meta ] )*
            () $name : fn ( $( $argtype ),* ) -> $rettype {
                $( 
                    $( #[ $emval ] )*
                    $variant = $value
                ),*
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        pub enum $name:ident : fn ( $($argtype:ty),* ) -> $rettype:ty {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident = $value:path 
            ),*
        }
    ) => {
        jump_table_main!(
            $( #[ $mval:meta ] )*
            (pub) $name : fn ( $( $argtype ),* ) -> $rettype {
                $( 
                    $( #[ $emval ] )*
                    $variant = $value
                ),*
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        pub ( ( $place:tt )* ) enum $name:ident : fn ( $($argtype:ty),* ) -> $rettype:ty {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident = $value:path 
            ),*
        }
    ) => {
        jump_table_main!(
            $( #[ $mval:meta ] )*
            ( pub ( ( $place )* ) ) $name : fn ( $( $argtype ),* ) -> $rettype {
                $( 
                    $( #[ $emval ] )*
                    $variant = $value
                ),*
            }
        );
    };
}

#[cfg(test)]
mod tests {

    fn one() -> &'static str {
        "one"
    }

    fn two() -> &'static str {
        "two"
    }

    fn three() -> &'static str {
        "three"
    }

    jump_table!(
        enum Thing: fn() -> &'static str {
            One = one,
            Two = two, 
            Three = three
        }
    );

    #[test]
    fn expansion_test() {
        let thing_fn: fn() -> &'static str = Thing::One.into();
        assert!(thing_fn() == "one");
    }
}