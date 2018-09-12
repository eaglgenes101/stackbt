#[doc(hidden)]
#[macro_export]
macro_rules! fn_proxy_main {
    (
        $name:ident;
        $( #[ $mval:meta ] )*
        $( $vis:ident )* $( ( $( $where:tt )* ) )* ; $fnname:ident ( 
            $( $argname:ident : $argtype:ty ),* 
        ) -> $rettype:ty $body:block
    ) => {
        $( #[ $mval ] )*
        $( $vis )* $( ( $( $where )* ) )* fn $fnname ( $( $argname : $argtype ),* ) 
        -> $rettype $body

        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        $( $vis )* $( ( $( $where )* ) )* struct $name ;

        impl Default for $name {
            fn default() -> $name {
                $name
            }
        }

        impl From < $name > for fn ( $( $argtype ),* ) -> $rettype {
            fn from ( _this : $name ) -> fn ( $( $argtype ),* ) 
            -> $rettype { $fnname }
        }
    };
}

#[macro_export]
macro_rules! fn_proxy {
    (
        $( #[ $mval:meta ] )* 
        $name:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) $block:block
    ) => {
    	fn_proxy_main!(
            $name ;
    		$( #[$mval] )*
    		; $fn ( $($arg : $ty),* ) -> ()
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        pub $name:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) $block:block
    ) => {
    	fn_proxy_main!(
            $name ;
    	    $( #[ $mval ] )*
    		pub ; $fn ( $($arg : $ty),* ) -> ()
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        pub ( $( place:tt )* ) $name:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) $block:block
    ) => {
    	fn_proxy_main!(
            $name ;
    		$( #[ $mval ] )*
    		pub ( $( place:tt )* ) ; $fn ( $($arg : $ty),* ) -> ()
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        $name:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) -> $ret:ty $block:block
    ) => {
    	fn_proxy_main!(
            $name ;
    		$( #[ $mval ] )*
    		; $fn ( $($arg : $ty),* ) -> $ret
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        pub $name:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) -> $ret:ty $block:block
    ) => {
    	fn_proxy_main!(
            $name ;
    		$( #[ $mval ] )*
    		pub ; $fn ( $($arg : $ty),* ) -> $ret
    		$block
    	);
    };

    (
        $( #[ $mval:meta ] )* 
        pub ( $( place:tt )* ) $name:ident =
        fn $fn:ident( $($arg:ident : $ty:ty),* ) -> $ret:ty $block:block
    ) => {
    	fn_proxy_main!(
            $name ;
    		$( #[ $mval ] )*
    		pub ( $( place:tt )* ) ; $fn ( $($arg : $ty),* ) -> $ret
    		$block
    	);
    };
}

mod tests {

    fn_proxy!(
        Foo = fn foo(a: i64, b: i64) -> i64 {
            a + b
        }
    );
}