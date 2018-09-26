#[doc(hidden)]
#[macro_export]
macro_rules! enum_eater {

    (@munch 
        $var:ident ; $name:ident ; ; 
        $( $p1:ident => ( $( $p2:tt )* ) ),* 
    ) => {
        match $var {
            $( $name :: $p1 => $( $p2 )* ),*
        }
    };

    (@munch 
        $var:ident ; $name:ident ; $var1:ident ; 
        $( $p1:ident => ( $( $p2:tt )* ) ),* 
    ) => {
        enum_eater!(@munch 
            $var ; $name ; ;
            $( $p1 => ( $( $p2 )* ) , )* $var1 => ( Option::None )
        );
    };

    (@munch 
        $var:ident ; $name:ident ; $var1:ident , $var2:ident ; 
        $( $p1:ident => ( $( $p2:tt )* ) ),* 
    ) => {
        enum_eater!(@munch 
            $var ; $name ; $var2 ;
            $( $p1 => ( $( $p2 )* ) , )* $var1 => ( Option::Some ( $name :: $var2 ) )
        );
    };

    (@munch 
        $var:ident ; $name:ident ; $var1:ident , $var2:ident , $( $othervar:ident ),* ; 
        $( $p1:ident => ( $( $p2:tt )* ) ),* 
    ) => {
        enum_eater!(@munch 
            $var ; $name ; $var2 , $( $othervar ),* ; 
            $( $p1 => ( $( $p2 )* ) , )* $var1 => ( Option::Some ( $name :: $var2 ) )
        );
    };

    ( $var:ident ; $name:ident ; $( $variant:ident ),+ ) => {
        enum_eater!(@munch $var ; $name ; $( $variant ),+ ; )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_iter_from {
    (
        enum $name:ident : $itername:ident {
            $( $variant:ident ),+
        }
    ) => {
        struct $itername(Option < $name > );

        impl Iterator for $itername {
            type Item = $name;
            
            fn next(&mut self) -> Option<Self::Item> {
                let orig = self.0;
                match orig {
                    Option::None => Option::None,
                    Option::Some(x) => {
                        self.0 = enum_eater!( x; $name ; $( $variant ),+ );
                        orig
                    }
                }
            }
        }

        impl IntoIterator for $name {
            type Item = $name;
            type IntoIter = $itername;
            fn into_iter(self) -> Self::IntoIter {
                $itername(Option::Some(self))
            }
        }
    };

    (
        $visibility:vis enum $name:ident : $itername:ident {
            $( $variant:ident ),+
        }
    ) => {
        $visibility struct $itername(Option < $name > );

        impl Iterator for $itername {
            type Item = $name;
            
            fn next(&mut self) -> Option<Self::Item> {
                let orig = self.0;
                match orig {
                    Option::None => Option::None,
                    Option::Some(x) => {
                        self.0 = enum_eater!(x; $name ; $( $variant ),+ );
                        orig
                    }
                }
            }
        }

        impl IntoIterator for $name {
            type Item = $name;
            type IntoIter = $itername;
            fn into_iter(self) -> Self::IntoIter {
                $itername (Option::Some(self))
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_iter_default {
    (
        $name:ident {
            $firstvariant:ident,
            $( $variant:ident ),*
        }
    ) => {
        impl Default for $name {
            fn default() -> Self {
                $name :: $firstvariant
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_iter_main {
    (
        $( #[ $mval:meta ] )*
        enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident
            ),+
        }
    ) => {
        $( #[ $mval ] )*
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        enum $name {
            $(
                $( #[ $emval ] )*
                $variant
            ),+
        }

        enum_iter_default!(
            $name {
                $( $variant ),+
            }
        );

        enum_iter_from!(
            enum $name : $itername {
                $( $variant ),+
            }
        );
    };


    (
        $( #[ $mval:meta ] )*
        $visibility:vis enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident
            ),+
        }
    ) => {
        $( #[ $mval ] )*
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        $visibility enum $name {
            $(
                $( #[ $emval ] )*
                $variant
            ),+
        }

        enum_iter_default!(
            $name {
                $( $variant ),+
            }
        );

        enum_iter_from!(
            $visibility enum $name : $itername {
                $( $variant ),+
            }
        );
    };
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
                $variant:ident
            ),+
        }
    ) => {
        enum_iter_main!(
            $( #[ $mval ] )*
            enum $name : $itername {
                $( 
                    $( #[ $emval ] )*
                    $variant
                ),+
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        $visibility:vis enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident
            ),+
        }
    ) => {
        enum_iter_main!(
            $( #[ $mval ] )*
            $visibility enum $name : $itername {
                $( 
                    $( #[ $emval ] )*
                    $variant
                ),+
            }
        );
    };
}

#[cfg(test)]
mod tests {
    enum_iter!(
        pub enum Foo: Bar {
            Baz, 
            Quux
        }
    );

    #[test]
    fn bar_iter_test() {
        let mut a = Foo::Baz.into_iter();
        let mut b = Foo::Quux.into_iter();
        assert_eq!(a.next(), Option::Some(Foo::Baz));
        assert_eq!(a.next(), Option::Some(Foo::Quux));
        assert_eq!(a.next(), Option::None);
        assert_eq!(b.next(), Option::Some(Foo::Quux));
        assert_eq!(b.next(), Option::None);
    }
}