use std::collections::HashSet;
use std::hash::Hash;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;
use rustyline::Helper;
use serde_json::{json, json_internal, Value};
use crate::{CommandHint, MyHelper};
use crate::LSPSuggestionHelper::LSPSuggestionHelper;


pub fn process_completions_response(resp: &str) -> Option<HashSet<CommandHint>> {
    //parse the response to a value using serde then enumerate the items adding each to the new set
    let json_bit: Value = serde_json::from_str::<Value>(resp).expect("failed to change");
    // println!("here is the jsson version{:?}", json_bit);

    if let Some(completions) = json_bit["result"]["items"].as_array(){
        //create the set of completions
        let mut set:HashSet<CommandHint> = HashSet::new();

        completions
            .iter()
            .for_each(|x|{
                if set.len() < 10{
                    let val = x["filterText"].as_str();
                    if val.is_some(){
                        let add = val.unwrap();
                        set.insert(CommandHint::new(add,add));
                    }
                }

            });

        return Some(set);
        // println!("{:?} here are the completions", set);
        // helper.0 = LSPSuggestionHelper{ hints: set};

        // println!(" going deeper {:?} ", completions);
    }
    else{
        println!("failed !");
        return None
    }

}

pub fn add_completions() {

}



// pub struct LSPCompletionContainer{
//     child_stdin: ChildStdin,
//     helper: Helper
// }