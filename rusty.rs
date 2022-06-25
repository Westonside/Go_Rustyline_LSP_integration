mod multiLineState;
// use std::ffi::CStr;
// use futures::executor::block_on;
use std::io::{BufRead, BufReader, Lines};
// use std::process::Command as Cmd;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use rustyline::InputMode::Command;
use async_process::{ChildStdout, Command as Cmd};
use async_process::Stdio;
use multiLineState::MultiLineState;
// use futures_lite::{io::BufReader, prelude::*};

// use async_process::Command as Cmdd;


//PUT BACK INTO GO FILE LATER
//extern int32_t double_input(int32_t input);
// 

pub fn rusty() -> Result<()> {
    //spawn a new process of the lsp
   
    // let mut something = Cmd::new("cargo")
    //     .arg("run")
    //     .arg("../flux-lsp")
    //     .stdout(Stdio::piped())
    //     .spawn()?;
    //
    // let mut lines = BufReader::new(something.stdout.take().unwrap()).lines();
    // testing(lines);


    // let a = something.stdout;
    // let b = String::from_utf8(a).unwrap();
    // println!("starting{}",format!("{:?}",b));

    // println!("result start {}", reverse("testing").unwrap());
    // park_timeout(1000ms);
    let mut paste: bool = false;
    let mut paste_state = MultiLineState::new();
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                
                rl.add_history_entry(line.as_str());
                if paste {
                    //add the line to the multiline state
                    paste_state.addRecord(line.to_string());
                }


                // let string = "line one
                // line two";
                // rl.add_history_entry(string);
                println!("Line: {}", line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                paste = !paste;
                println!("CTRL-D: Paste mode is {}", paste);


                if !paste && paste_state.entries() > 0 {
                    rl.add_history_entry(paste_state.resultString());
                }
                //clear the vec
                if paste == false{
                    paste_state.cleanse();
                }
                continue
                // break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt")
}

fn call_async(){
    rusty();
}

fn main() {
    call_async();
}