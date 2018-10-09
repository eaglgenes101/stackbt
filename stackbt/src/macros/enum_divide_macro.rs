//! Copypaste of stackbt_macros/enum_divide_macro.rs which exists 
//! as a workaround to the inability to reexport macros. 

#[doc(hidden)]
#[macro_export]
macro_rules! enum_variant_define {
    (
        $oldname:ident {
            $( $( $oldvariant:ident )|* => $variant:ident ),*
        }
    ) => {
        $(
            #[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
            enum $variant {
                $( $oldvariant ),*
            }

            impl From < $variant > for $oldname {
                fn from( this: $variant ) -> $oldname {
                    match this {
                        $( $variant :: $oldvariant => $oldname :: $oldvariant ),*
                    }
                }
            }
        )*
    };

    (
        $visibility:vis $oldname:ident {
            $( $( $oldvariant:ident )|* => $variant:ident ),*
        }
    ) => {
        $(
            #[derive(Copy, Clone, Hash, Debug, Eq, PartialEq)]
            $visibility enum $variant {
                $( $oldvariant ),*
            }

            impl From < $variant > for $oldname {
                fn from( this: $variant ) -> $oldname {
                    match this {
                        $( $variant :: $oldvariant => $oldname :: $oldvariant ),*
                    }
                }
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_divide_from_body {
    (@munch 
        $var:ident ; $name:ident : $oldname:ident ; ; 
        $( ( $( $from:tt )* ) => ( $( $to:tt )* ) ),*
    ) => {
        match $var {
            $( $( $from )* => $( $to )* ),*
        }
    };

    (@munch 
        $var:ident ; $name:ident : $oldname:ident ;
        $( $firstoldvariant:ident )|* => $firstvariant:ident ; 
        $( ( $( $from:tt )* ) => ( $( $to:tt )* ) ),*
    ) => {
        enum_divide_from_body!(@munch
            $var ; $name : $oldname ; ; 
            $( ( $( $from )* ) => ( $( $to )* ) , )*
            $(
                ( $name :: $firstvariant (
                    $firstvariant :: $firstoldvariant
                ) ) => ( $oldname :: $firstoldvariant ) 
            ),*
        );
    };

    (@munch 
        $var:ident ; $name:ident : $oldname:ident ;
        $( $firstoldvariant:ident )|* => $firstvariant:ident ,
        $( $( $oldvariant:ident )|* => $variant:ident ),* ; 
        $( ( $( $from:tt )* ) => ( $( $to:tt )* ) ),*
    ) => {
        enum_divide_from_body!(@munch
            $var ; $name : $oldname ;
            $( $( $oldvariant )|* => $variant ),* ; 
            $( ( $( $from )* ) => ( $( $to )* ) , )*
            $(
                ( $name :: $firstvariant (
                    $firstvariant :: $firstoldvariant
                ) ) => ( $oldname :: $firstoldvariant ) 
            ),*
        );
    };

    (
        $var:ident ; $name:ident : $oldname:ident {
            $( $( $oldvariant:ident )|* => $variant:ident ),*
        }
    ) => {
        enum_divide_from_body!(@munch
            $var ; $name : $oldname ;
            $( $( $oldvariant )|* => $variant ),* ;
        );
    }
}

#[doc(hidden)]
#[macro_export]
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
                        $( 
                            $oldname :: $oldvariant  => 
                            $name :: $variant ( $variant :: $oldvariant ) 
                        ),*
                    ),*
                }
            }
        }

        impl From< $name > for $oldname {
            fn from( this: $name ) -> $oldname {
                enum_divide_from_body!(
                    this ; $name : $oldname {
                        $( $( $oldvariant )|* => $variant ),*
                    }
                )
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_divide_main {
    (
        $( #[ $mval:meta ] )*
        $name:ident : $oldname:ident {
            $( 
                $( #[ $emval:meta ] )*
                $( $oldvariant:ident )|* => $variant:ident
            ),*
        }
    ) => {

        $( #[ $mval ] )*
        #[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
        enum $name {
            $( 
                $( #[ $emval ] )*
                $variant ( $variant )
            ) , *
        }

        enum_variant_define!(
            $oldname {
                $( $( $oldvariant )|* => $variant ),*
            }
        );

        enum_divide_from!(
            $name : $oldname {
                $( $( $oldvariant )|* => $variant ),*
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        $visibility:vis $name:ident : $oldname:ident {
            $( 
                $( #[ $emval:meta ] )*
                $( $oldvariant:ident )|* => $variant:ident
            ),*
        }
    ) => {

        $( #[ $mval ] )*
        #[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
        $visibility enum $name {
            $( 
                $( #[ $emval ] )*
                $variant ( $variant )
            ) , *
        }

        enum_variant_define!(
            $visibility $oldname {
                $( $( $oldvariant )|* => $variant ),*
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
/// of existing ones. 
/// 
/// The new enum's name must be a legal, unused enum name, and its variants 
/// must all be valid enum variant names. In addition, the old enum must be 
/// fieldless, and the declared old enum's variants must actually be an  
/// exhausive enumeration of that old enum's variants. 
/// 
/// From this, the macro will expand to a new enum with the listed new 
/// variants, as well as corresponding traits, including one for conversion 
/// from the old enum to the new one according to the specified mappings 
/// between old and new enum variants. 
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
            $name : $oldname {
                $( 
                    $( #[ $emval ] )*
                    $( $oldvariant )|* => $variant
                ),*
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        $visibility:vis enum $name:ident : $oldname:ident {
            $( 
                $( #[ $emval:meta ] )*
                $( $oldvariant:ident )|* => $variant:ident
            ),*
        }
    ) => {
        enum_divide_main!(
            $( #[ $mval ] )*
            $visibility $name : $oldname {
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



