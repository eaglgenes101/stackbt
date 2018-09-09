macro_rules! fn_singleton_main {
    (
        $enumname:ident;
        $( #[ $mval:meta ] )*
        $( $vis:ident )* $( ( $( $where:tt )* ) )* ; $fnname:ident ( 
            $( $argname:ident : $argtype:ty ),* 
        ) -> $rettype:ty $body:block
    ) => {
        $( #[ $mval ] )*
        $( $vis )* $( ( $( $where )* ) )* fn $fnname ( $( $argname : $argtype ),* ) 
        -> $rettype $body

        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        $( $vis )* $( ( $( $where )* ) )* enum $enumname {
            $enumname
        }

        impl Default for $enumname {
            fn default() -> $enumname {
                $enumname :: $enumname
            }
        }

        use std::fmt::{Error, Formatter, Display};

        impl Display for $enumname {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                f.write_str( stringify!( $enumname ) )
            }
        }

        impl From < $enumname > for fn ( $( $argtype ),* ) -> $rettype {
            fn from ( _this : $enumname ) -> fn ( $( $argtype ),* ) 
            -> $rettype { $fnname }
        }
    };
}

#[macro_export]
macro_rules! fn_singleton {
    (
        $( #[ $mval:meta ] )* 
        $enumname:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) $block:block
    ) => {
    	fn_singleton_main!(
            $enumname ;
    		$( #[$mval] )*
    		; $fn ( $($arg : $ty),* ) -> ()
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        pub $enumname:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) $block:block
    ) => {
    	fn_singleton_main!(
            $enumname ;
    	    $( #[ $mval ] )*
    		pub ; $fn ( $($arg : $ty),* ) -> ()
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        pub ( $( place:tt )* ) $enumname:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) $block:block
    ) => {
    	fn_singleton_main!(
            $enumname ;
    		$( #[ $mval ] )*
    		pub ( $( place:tt )* ) ; $fn ( $($arg : $ty),* ) -> ()
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        $enumname:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) -> $ret:ty $block:block
    ) => {
    	fn_singleton_main!(
            $enumname ;
    		$( #[ $mval ] )*
    		; $fn ( $($arg : $ty),* ) -> $ret
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        pub $enumname:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) -> $ret:ty $block:block
    ) => {
    	fn_singleton_main!(
            $enumname ;
    		$( #[ $mval ] )*
    		pub ; $fn ( $($arg : $ty),* ) -> $ret
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        pub ( $( place:tt )* ) $enumname:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) -> $ret:ty $block:block
    ) => {
    	fn_singleton_main!(
            $enumname ;
    		$( #[ $mval ] )*
    		pub ( $( place:tt )* ) ; $fn ( $($arg : $ty),* ) -> $ret
    		$block
    	);
    };
}

mod tests {
    fn_singleton!(
        Foo = fn foo(a: i64, b: i64) -> i64 {
            a + b
        }
    );
}