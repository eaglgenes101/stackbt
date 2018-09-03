use std::ops::FnMut;

pub trait Automaton<'f, I, A> {
    #[must_use]
    fn transition(&'f mut self, input: &I) -> A;
}

/// Automaton impl for types which implement FnMut(&I) -> A. This is not 
/// recommended for uses other than shimming existing closures into 
/// automata parameters, as this results in a highly opaque automaton. 
impl<'f, I, A, C: FnMut(&I) -> A> Automaton<'f, I, A> for C {
    fn transition(&'f mut self, input: &I) -> A {
        self(&input)
    }
}

#[cfg(test)]
mod tests {
    use automaton::Automaton;

    #[test]
    fn fnmut_automaton_test() {
        let mut i: i64 = 0;
        let mut j: i64 = 1;
        {
            let mut w = |r: &i64| {
                i = j;
                j = *r;
                i
            };
            let k = &mut w;
            assert!(k.transition(&3) == 1);
            assert!(k.transition(&2) == 3);
            assert!(k.transition(&6) == 2);
        }
        assert!(i == 2);
        assert!(j == 6);
    }

}
