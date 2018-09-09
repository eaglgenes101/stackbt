macro_rules! enum_variant_define {
    (
        ( $( $vis:tt )* ) $oldname:ident {
            $( $( $oldvariant:ident )|* => $variant:ident ),*
        }
    ) => {
        $(
            #[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
            enum $variant {
                $( $oldvariant ),*
            }

            impl From < $oldname > for $variant {
                fn from(this: $oldname ) -> $variant {
                    match this {
                        $( $oldname :: $oldvariant => $variant :: $oldvariant , )*
                        _ => unreachable!("Your macro has failed you!")
                    }
                }
            }

            impl Display for $variant {
                fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                    let disp_str = match self {
                        $( $variant :: $oldvariant => stringify!( $oldvariant ) ),*
                    };
                    f.write_str(disp_str)
                }
            }
        )*
    }
}

macro_rules! enum_divide_display {
    (
        $name:ident {
            $( $variant:ident ),*
        }
    ) => {
        impl Display for $name {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                let disp_str = match self {
                    $( $name :: $variant (_) => stringify!( $variant ) ),*
                };
                f.write_str(disp_str)
            }
        }
    };
}

macro_rules! enum_divide_from {
    (
        $name:ident : $oldname:ident {
            $( $( $oldvariant:ident )|* => $variant:ident ),*
        }
    ) => {

        impl From< $oldname > for $name {
            fn from(old: $oldname) -> $name {
                match old {
                    $( 
                        $( $oldname :: $oldvariant  => 
                        $name :: $variant ( $variant :: $oldvariant ) ),*
                    ),*
                }
            }
        }
    }
}

macro_rules! enum_divide_main {
    (
        $( #[ $mval:meta ] )*
        ( $( $vis:tt )* ) enum $name:ident : $oldname:ident {
            $( 
                $( #[ $emval:meta ] )*
                $( $oldvariant:ident )|* => $variant:ident
            ),*
        }
    ) => {
        use std::fmt::{Error, Formatter, Display};
        use std::convert::From;

        $( #[ $mval ] )*
        #[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
        $( $vis )* enum $name {
            $( 
                $( #[ $emval ] )*
                $variant ( $variant )
            ) , *
        }

        enum_variant_define!(
            ( $( $vis )* ) $oldname {
                $( $( $oldvariant )|* => $variant ),*
            }
        );

        enum_divide_display!(
            $name {
                $( $variant ),*
            }
        );

        enum_divide_from!(
            $name : $oldname {
                $( $( $oldvariant )|* => $variant ),*
            }
        );
    };
}

/// Declarative macro for quickly and easily declaring enums as partitions 
/// of existing ones. This macro expects to enclose a statement with form 
/// similar to the following: 
/// ```ignore
/// #[attribute_0]
/// #[attribute_1]
/// #[attribute_2]
/// enum NewName: match OldName {
///     OldVariant00 | OldVariant01 => NewVariant0,
///     #[field_attribute_0]
///     #[field_attribute_1]
///     #[field_attribute_2]
///     OldVariant10 => NewVariant1,
///     OldVariant11 => NewVariant2
/// }
/// ```
/// The new enum's name must be a legal, unused enum name, and its variants 
/// must all be valid enum variant names. In addition, the old enum must be 
/// fieldless, and the declared old enum's variants must actually be an  
/// exhausive enumeration of that old enum's variants. 
/// 
/// From this, the macro will expand to a new enum with the listed new 
/// variants, as well as corresponding traits, including one for conversion 
/// from the old enum to the new one according to the specified mappings 
/// between old and new enum variants. The generated conversion is 
/// irreversible, and does not preserve information about which old enum 
/// variant corresponds to each new one. 
#[macro_export]
macro_rules! enum_divide {
    (
        $( #[ $mval:meta ] )*
        enum $name:ident : $oldname:ident {
            $( 
                $( #[ $emval:meta ] )*
                $( $oldvariant:ident )|* => $variant:ident
            ),*
        }
    ) => {
        enum_divide_main!(
            $( #[ $mval ] )*
            () enum $name : $oldname {
                $( 
                    $( #[ $emval ] )*
                    $( $oldvariant )|* => $variant
                ),*
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        pub enum $name:ident : $oldname:ident {
            $( 
                $( #[ $emval:meta ] )*
                $( $oldvariant:ident )|* => $variant:ident
            ),*
        }
    ) => {
        enum_divide_main!(
            $( #[ $mval ] )*
            ( pub ) enum $name : $oldname {
                $( 
                    $( #[ $emval ] )*
                    $( $oldvariant )|* => $variant
                ),*
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        pub $place:tt enum $name:ident : $oldname:ident {
            $( 
                $( #[ $emval:meta ] )*
                $( $oldvariant:ident )|* => $variant:ident
            ),*
        }
    ) => {
        enum_divide_main!(
            $( #[ $mval ] )*
            ( pub $place ) enum $name : $oldname {
                $( 
                    $( #[ $emval ] )*
                    $( $oldvariant )|* => $variant
                ),*
            }
        );
    };
}

mod tests {
    enum Foo {
        Foo0,
        Foo1,
        Foo2,
        Foo3
    }

    enum_divide!(
        enum Bar : Foo {
            Foo0 | Foo3 => Bar0,
            Foo1 | Foo2 => Bar1
        }
    );

    #[test]
    fn expansion_test() {
        let thing: Bar = Foo::Foo1.into();
        match thing {
            Bar::Bar0(Bar0::Foo0) => unreachable!(),
            Bar::Bar0(Bar0::Foo3) => unreachable!(),
            Bar::Bar1(Bar1::Foo1) => (),
            Bar::Bar1(Bar1::Foo2) => unreachable!()
        }
    }
}



