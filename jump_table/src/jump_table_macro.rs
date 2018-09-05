macro_rules! jump_table_copy {
    (
        $name:ident {
            $( $variant:ident ) ,*
        }
    ) => {
        use std::marker::{Copy};

        impl Copy for $name {}
        impl Clone for $name {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
}

macro_rules! jump_table_eq {
    (
        $name:ident {
            $( $variant:ident ) ,*
        }
    ) => {
        use std::cmp::{PartialEq, Eq};

        impl Eq for $name {}
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    $( ( & $name::$variant, & $name::$variant ) => true, ) *
                    _ => false
                }
            }
        }
    }
}

macro_rules! jump_table_display {
    (
        $name:ident {
            $( $variant:ident ) ,*
        }
    ) => {
        use std::fmt::{Error, Formatter, Debug, Display};

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                let disp_str = match self {
                    $( $variant => stringify!( $variant )), +
                };
                f.write_str(disp_str)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                let disp_str = match self {
                    $( $variant => stringify!( $variant )), +
                };
                f.write_str(disp_str)
            }
        }
    };
}

macro_rules! jump_table_from {
    (
        $name:ident : $fntype:ty {
            $( $variant:ident = $value:expr ) ,*
        }
    ) => {
        use std::convert::From;

        impl From< $name > for $fntype {
            fn from( val: $name ) -> Self {
                match val {
                    $( $variant => $value ), *
                }
            }
        }
    };
}

#[macro_export]
macro_rules! jump_table {
    (
        $( #[ $mval:meta ] ) *
        enum $name:ident : $fntype:ty {
            $( $variant:ident = $value:expr ) , *
        }
    ) => {
        $( #[ $mval ] ) *
        enum $name {
            $( $variant ), *
        }

        jump_table_copy!(
            $name {
                $( $variant ), *
            }
        );

        jump_table_eq!(
            $name {
                $( $variant ), *
            }
        );

        jump_table_display!(
            $name {
                $( $variant ), *
            }
        );

        jump_table_from!(
            $name : $fntype {
                $( $variant = $value ), *
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