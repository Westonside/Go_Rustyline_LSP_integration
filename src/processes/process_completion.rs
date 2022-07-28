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

    return if let Some(completions) = json_bit["result"]["items"].as_array() {
        //create the set of completions
        // println!("there are completions in here!");
        let mut set: HashSet<CommandHint> = HashSet::new();

        completions
            .iter()
            .for_each(|x| {

                    let val = match  x["insertText"].as_str(){
                        None => {
                            x["label"].as_str().unwrap()
                        }
                        Some(val) => {val}
                    };

                    let kind = x["kind"].as_u64().unwrap();
                    if let Some(detail)= x["detail"].as_str(){
                        // println!("vals {} {} {}", kind,detail, val);

                    }
                    else{
                        // println!("vals {}  {}", kind, val);
                    }
                    // let detail = x["detail"].as_str().unwrap();
                    set.insert(CommandHint::new(val,val,0,None));

            });
        Some(set)
    } else {

        None
    }

}

pub fn add_completions() {

}



// pub struct LSPCompletionContainer{
//     child_stdin: ChildStdin,
//     helper: Helper
// }