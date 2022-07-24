use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use rustyline::hint::{Hint, Hinter};
use rustyline::Context;
use rustyline::{Editor, Result};
use rustyline_derive::{Completer, Helper, Highlighter, Validator};


#[derive(Completer, Helper, Validator, Highlighter)]
pub struct LSPSuggestionHelper {
    pub(crate) hints: Arc<RwLock<HashSet<CommandHint>>>,
}

#[derive(Hash, Debug, PartialEq, Eq)]
pub struct CommandHint {
    pub(crate) display: String,
    complete_up_to: usize,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}


impl CommandHint {
    pub fn new(text: &str, complete_up_to: &str) -> CommandHint {
        assert!(text.starts_with(complete_up_to));
        CommandHint {
            display: text.into(),
            complete_up_to: complete_up_to.len(),
        }
    }

    pub(crate) fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}



impl Hinter for LSPSuggestionHelper {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        //instead of going through the hash set run a function that gets from the receiver and then does it
        self.hints
            .read()
            .unwrap()
            .iter()
            .filter_map(|hint| {

                if hint.display.starts_with(line) {
                    Some(hint.suffix(pos))
                } else {
                    None
                }
            })
            .next()
    }
}
impl LSPSuggestionHelper{
    pub(crate) fn print_hints(&self){
        println!("running hint runner {}", self.hints.read().unwrap().len());
        let a = self.hints.read().unwrap();
        a
            .iter()
            .for_each(|x|{
                println!("\nhere is a hint that we have!:  {}\n", x.display);
            })
    }
}



// pub fn diy_hints() -> HashSet<CommandHint> {
//     let mut set = HashSet::new();
//     set.insert(CommandHint::new("help", "help"));
//     set.insert(CommandHint::new("get key", "get "));
//     set.insert(CommandHint::new("set key value", "set "));
//     set.insert(CommandHint::new("hget key field", "hget "));
//     set.insert(CommandHint::new("hset key field value", "hset key field value"));
//     set
// }