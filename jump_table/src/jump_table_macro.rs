macro_rules! jump_table_display {
    (
        $name:ident {
            $( $variant:ident ) ,*
        }
    ) => {
        use std::fmt::{Error, Formatter, Display};

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
        enum $name:ident : fn ( $($argtype:ty),* ) -> $rettype:ty {
            $( $variant:ident = $value:path ) , *
        }
    ) => {
        $( #[ $mval ] ) *
        #[derive(Copy, Clone, Hash, Debug, PartialEq, Eq)]
        enum $name {
            $( $variant ) , *
        }

        jump_table_display!(
            $name {
                $( $variant ), *
            }
        );

        jump_table_from!(
            $name : fn ( $($argtype),* ) -> $rettype {
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