use std::collections::HashSet;
use std::process::{Child, Command, Stdio};
use serde_json::{json, json_internal, Value};
use crate::CommandHint;


pub fn process_completions_response(resp: &str) {
    //parse the response to a value using serde then enumerate the items adding each to the new set
    let json_bit: Value = serde_json::from_str::<Value>(resp).expect("failed to change");
    println!("here is the jsson version{:?}", json_bit);

    if let Some(completions) = json_bit["result"]["items"].as_array(){
        //create the set of completions
        let mut set:HashSet<CommandHint> = HashSet::new();

        completions
            .iter()
            .for_each(|x|{
                let val = x["filterText"].as_str().unwrap();
                set.insert(CommandHint::new(val,val));
            });
        println!("{:?} here are the completipons", set);

        // println!(" going deeper {:?} ", completions);
    }
    else{
        println!("failed !");
    }

}

pub fn add_completions() {

}



// pub struct LSPCompletionContainer{
//     child_stdin: ChildStdin,
//     helper: Helper
// }