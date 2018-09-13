#[doc(hidden)]
#[macro_export]
macro_rules! enum_eater {

    (@munch 
        $var:ident ; 
        $name:ident ; ; 
        $( $p1:ident => ( $( $p2:tt )* ) ),* 
    ) => {
        match $var {
            $( $name :: $p1 => $( $p2 )* ),*
        }
    };

    (@munch 
        $var:ident ;
        $name:ident ; 
        $var1:ident ; 
        $( $p1:ident => ( $( $p2:tt )* ) ),* 
    ) => {
        enum_eater!(@munch 
            $var ;
            $name ; ;
            $( $p1 => ( $( $p2 )* ) , )* $var1 => ( Option::None )
        );
    };

    (@munch 
        $var:ident ;
        $name:ident ; 
        $var1:ident , $var2:ident ; 
        $( $p1:ident => ( $( $p2:tt )* ) ),* 
    ) => {
        enum_eater!(@munch 
            $var ;
            $name ;
            $var2 ;
            $( $p1 => ( $( $p2 )* ) , )* $var1 => ( Option::Some ( $name :: $var2 ) )
        );
    };

    (@munch 
        $var:ident ;
        $name:ident ; 
        $var1:ident , $var2:ident , $( $othervar:ident ),* ; 
        $( $p1:ident => ( $( $p2:tt )* ) ),* 
    ) => {
        enum_eater!(@munch 
            $var ;
            $name ;
            $var2 , $( $othervar ),* ; 
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
        $( ( $( $vis:tt )* ) )* ; enum $name:ident : $itername:ident {
            $( $variant:ident ),+
        }
    ) => {
        
        $( $( $vis )* )* struct $itername {
            f: $name
        }

        impl Iterator for $itername {
            type Item = $name;
            
            fn next(&mut self) -> Option<Self::Item> {
                let t = self.f;
                let next = enum_eater!(
                    t ;
                    $name ; $( $variant ),+
                );
                match next {
                    Option::Some(t) => {
                        self.f = t;
                        Option::Some(t)
                    }
                    Option::None => Option::None
                }
            }
        }

        impl IntoIterator for $name {
            type Item = $name;
            type IntoIter = $itername;

            fn into_iter(self) -> Self::IntoIter {
                $itername {
                    f: self
                }
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! enum_iter_default {
    (
        enum $name:ident {
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
        $( ( $( $vis:tt )* ) )* ; enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident
            ),+
        }
    ) => {
        $( #[ $mval ] )*
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        $( $( $vis )* )* enum $name {
            $(
                $( #[ $emval ] )*
                $variant
            ),+
        }

        enum_iter_default!(
            enum $name {
                $( $variant ),+
            }
        );

        enum_iter_from!(
            $( ( $( $vis:tt )* ) )* ; enum $name : $itername {
                $( $variant ),+
            }
        );
    };
}

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
            ; enum $name : $itername {
                $( 
                    $( #[ $emval ] )*
                    $variant
                ),+
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        pub enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident
            ),+
        }
    ) => {
        enum_iter_main!(
            $( #[ $mval ] )*
            ( pub ) ; enum $name : $itername {
                $( 
                    $( #[ $emval ] )*
                    $variant
                ),+
            }
        );
    };

    (
        $( #[ $mval:meta ] )*
        pub $place:tt enum $name:ident : $itername:ident {
            $( 
                $( #[ $emval:meta ] )*
                $variant:ident
            ),+
        }
    ) => {
        enum_iter_main!(
            $( #[ $mval ] )*
            ( pub $place ) ; enum $name : $itername {
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
        enum Foo: Bar {
            Baz, 
            Quux
        }
    );
}