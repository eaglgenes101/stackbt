#[doc(hidden)]
#[macro_export]
macro_rules! from_proxy_main {
    (
        $name:ident;
        $( $vis:ident )* ; $rettype:ty $body:expr
    ) => {

        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        $( $vis )* $( ( $( $where )* ) )* struct $name ;

        impl Default for $name {
            fn default() -> $name {
                $name
            }
        }

        impl From < $name > for $rettype {
            fn from ( _this : $name ) -> $rettype { $expr }
        }
    };
}

#[macro_export]
macro_rules! from_proxy {
    (
        $name:ident : $ret:ty = $block:expr
    ) => {
    	from_proxy_main!(
            $name ;
    		;$ret
    		$block
    	);
    };

    (
        pub $name:ident : $ret:ty = $block:expr
    ) => {
    	from_proxy_main!(
            $name ;
    		pub ; $ret
    		$block
    	);
    };

    (
        pub ( $( place:tt )* ) $name:ident: $ret:ty = $block:expr
    ) => {
    	from_proxy_main!(
            $name ;
    		pub ( $( place:tt )* ) ; $ret
    		$block
    	);
    };
}

#[cfg(test)]
mod tests {
    from_proxy!(
        Foo : i64 = 5
    );

    #[test]
    fn expansion_test() {
        assert!(Foo::default().into() == 5);
    }
}