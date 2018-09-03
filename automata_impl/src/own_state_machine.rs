use std::ops::FnOnce;
use automaton::Automaton;
use std::marker::PhantomData;
use std::mem::swap;

/// State machine implemented through a boxed consumable closure struct. Each 
/// step, the currently boxed closure is called, returning an action and a 
/// new boxed closure to call the next step. 
pub struct OwnStateMachine<I, A, C: FnOnce(&I) -> (A, Box<C>)> {
    current_state: Option<Box<C>>,
    _i_life_check: PhantomData<I>,
    _a_life_check: PhantomData<A>,
}

impl <I, A, C: FnOnce(&I) -> (A, Box<C>)> OwnStateMachine<I, A, C> {
    pub fn new(init_state: Box<C>) -> OwnStateMachine<I, A, C> {
        OwnStateMachine {
            current_state: Option::Some(init_state),
            _i_life_check: PhantomData,
            _a_life_check: PhantomData
        }
    }

    fn step(&mut self, input: &I) -> A {
        let mut holding_box: Option<Box<C>> = Option::None;
        swap(&mut self.current_state, &mut holding_box);
        let box_ptr = Box::into_raw(holding_box.unwrap());
        let return_tuple;
        //Just a single line of pointerwork. Look at it carefully. 
        unsafe {
            return_tuple = (box_ptr.read())(&input);
            //The closure is now spent, and probably dropped. box_ptr points 
            //to memory which corresponds to where the original current_state 
            //box used to be. 

            //I'm assuming here that now that the thing box_ptr points to is 
            //spent, the box's drop job is done, so dropping the raw pointer 
            //is fine. 

            //If something panicks somewhere in here, the worst that happens 
            //is that a move closure is leaked and this struct is poisoned 
            //by virtue of occupying a None state. I think. 
        }
        let mut return_box = Option::Some(return_tuple.1);
        swap(&mut self.current_state, &mut return_box);
        return_tuple.0
    }
}

impl <I, A, C: FnOnce(&I) -> (A, Box<C>)> Automaton<'static, I, A> for OwnStateMachine<I, A, C> {
    fn transition(&mut self, input: &I) -> A {
        self.step(input)
    }
}