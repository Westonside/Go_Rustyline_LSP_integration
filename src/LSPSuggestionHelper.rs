use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use rustyline::hint::{Hint, Hinter};
use rustyline::Context;
use rustyline::{Editor, Result};
use rustyline::KeyCode::PageUp;
use rustyline_derive::{Completer, Helper, Highlighter, Validator};


#[derive(Completer, Helper, Validator, Highlighter)]
pub struct LSPSuggestionHelper {
    pub(crate) hints: Arc<RwLock<HashSet<CommandHint>>>,
    pub (crate) skip_pos: Arc<RwLock<Option<usize>>>
}

// pub struct CommandHint2{
//     pub(crate) display: String,
//     complete_up_to: usize,
//
// }


#[derive(Hash, Debug, PartialEq, Eq)]
pub struct CommandHint {
    pub(crate) display: String,
    complete_up_to: usize,
    hint_type: u8,
    hint_signature: Option<String>
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
    pub fn new(text: &str, complete_up_to: &str, hint_type: u8, sig: Option<String>) -> CommandHint {
        assert!(text.starts_with(complete_up_to));
        CommandHint {
            display: text.into(),
            complete_up_to: complete_up_to.len(),
            hint_type,
            hint_signature: sig
        }
    }

    pub(crate) fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
            hint_type: 0,
            hint_signature: None
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



    //needs fixing
    pub(crate) fn get_best_hint(&self, line: &str) -> Option<CommandHint>{
        let lock = self.hints.read().unwrap();
        // println!("going through the hints atm!");

        // let mut s = CommandHint::new("", "", 0, None);
        // let mut cur_best = i32::MAX;
        // let mut overlap = 0;
        self.print_hints();
        let mut state = (&CommandHint::new("", "", 0, None),i32::MAX, 0 as usize );
        //go through all the hints and find the best hint
        for hint in lock.iter(){
            // println!("current hint! {}", hint.display);
            if let Some((hint_len,overlap)) = current_line_ends_with(line, &hint.display){
                let mut abs_val = hint_len as i32 - overlap as i32;
                // println!("something is ending with this {} {} {} and ", hint.display, abs_val, state.1);

                abs_val = i32::abs(abs_val);
                if state.1 > abs_val{
                    // println!("switching {} and {}", hint.display, state.1);
                    state.1 = abs_val;
                    state.0 = hint;
                    state.2 = overlap;
                }
            }
        }

        if state.1 == i32::MAX{
            // println!("the biggest failure");
            println!("no change");
            return None
        }
        // println!("this is what is being returned {}", state.0.display);
        return Some(state.0.suffix(state.2));
    }


    pub(crate) fn best_hint_get_new(&self, line: &str) -> Option<CommandHint>{
        self.print_hints();
        let mut best = usize::MAX;
        let mut best_overlap: &str = "";
        let mut best_hint = &CommandHint::new("", "", 0, None);
        let lock = self.hints.read().unwrap();
        for hint in lock.iter(){
            //for each hint find the biggest overlap compared to size
            if let Some(overlap) = better_overlap(line, &hint.display){
                let diff = overlap.len().abs_diff(hint.display.len());
                if best > diff {
                    best = diff;
                    best_hint = hint;
                    best_overlap = overlap;
                }
            }
        }

        return match best {
            usize::MAX =>{None}
            _=>{Some(best_hint.suffix(best_overlap.len()))}
        };
        None
    }




}




//needs a lot of fixing
pub(crate) fn current_line_ends_with(line: &str, comp: &str) -> Option<(usize, usize)>{
    let mut i: i8 = (line.len()-1) as i8;
    while i > -1{
        let up_to = &line[i as usize..];
        // println!("{}", up_to);
        if comp.starts_with(up_to){

            return Some((comp.len(),up_to.len()))
        }
        i = i-1;
    }
    None
}

#[cfg(test)]
mod tests_overlap {
    use std::collections::HashSet;
    use std::sync::{Arc, RwLock};
    use crate::LSPSuggestionHelper::{better_overlap, LSPSuggestionHelper, valid_checker};

    #[test]
    fn overlap_test_one() {
        assert_eq!(better_overlap("import \"dat", "date"), Some("dat"));
    }
    #[test]
    fn overlap_import(){
        let out = better_overlap("imp", "import");
        println!("{:?}", out);
        assert_eq!(out, Some("imp"));
    }
    #[test]
    fn from_test(){
        let out = better_overlap("fr", "from");
        println!("{:?}", out);
        assert_eq!(out, Some("fr"));
    }

    #[test]
    fn import_test_two(){
        let out = better_overlap("import", "truncate");
        println!("{:?}", out);
        assert_eq!(out, None);
    }

    #[test]
    fn import_test_three(){
        let out = better_overlap("import", "import");
        println!("{:?}", out);
        assert_eq!(out, Some("import"));
    }

    #[test]
    fn test_valid_checker(){
        let a = "de";
        let cur = "e";
        let goal = "elapsed";
        assert_eq!(valid_checker(a,cur,goal), false)
    }

    #[test]
    fn test_valid_checker_two(){
        let a = "de";
        let cur = "de";
        let goal = "derive";
        assert_eq!(valid_checker(a,cur,goal), true)
    }


    // #[test]
    // fn pick_best_hint_easy(){
    //     let a = LSPSuggestionHelper{ hints: Arc::new(RwLock::new(HashSet::new())), skip_pos: Arc::new(Default::default()) };
    //     let lock = a.hints.write();
    //
    //     assert_eq!(out, Some("import"));
    // }


}

fn better_overlap<'a>(line: &'a str, comp: &'a str) -> Option<&'a str>{

    for (i,ch) in comp.chars().enumerate(){
        let (l,r) = comp.split_at(i);
        if valid_checker(line,l,comp){
            // if line.ends_with(l)  && !l.is_empty() && !comp.ends_with(l){
            //     println!("ret this: {:?}",l);
                return Some(l);
        }
        else if valid_checker(line,r,comp){
            // line.ends_with(r) && !r.is_empty() && !comp.ends_with(r)
            // println!("ret this: {:?}",l);
            return Some(r);
        }
    }
    None
}

fn valid_checker(line: &str, overlap: &str, goal: &str) -> bool {
        let first_valid = line.ends_with(overlap) && !overlap.is_empty() && !goal.ends_with(overlap);
        let line_split = line.split(" ").collect::<Vec<&str>>();
        //get the last item
        let last_ref = line_split[line_split.len()-1];
        //remove the overlap from the line and add together
        let mut newer = last_ref.to_string();
        let clean_goal = goal.replacen(overlap, "", 1);
        newer.push_str(&clean_goal);
    //combine the removed with the overlap and you should get the goal

    return first_valid && newer == goal;
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