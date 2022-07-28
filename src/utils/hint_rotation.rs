use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use crate::CommandHint;

pub struct hint_stack{
    hints: Arc<RwLock<HashSet<CommandHint>>>,
    skip_pos: Option<usize>
}

impl hint_stack{


    fn stack_with_hints(hint: Arc<RwLock<HashSet<CommandHint>>>) -> Self{
        hint_stack{ hints: hint, skip_pos: None }
    }

    fn next_hint(&self){
        let mut lock = self.hints.write().expect("could not get a write lock in hint rotater");
        //remove the first element from the hashet

    }
}