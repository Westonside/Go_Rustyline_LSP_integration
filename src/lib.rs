mod multiLineState;
mod lsp_invoke;

// use std::ffi::CStr;
// use futures::executor::block_on;
use std::io::{BufRead, BufReader, Lines};
// use std::process::Command as Cmd;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use rustyline::InputMode::Command;
use multiLineState::MultiLineState;



fn rusty() -> Result<()> {
    let mut paste: bool = false;
    let mut paste_state = MultiLineState::new();
    // `()` can be used when no completer is required
    lsp_invoke::invoke_lsp();
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
#[no_mangle]
pub extern "C" fn double_input(input: i32) -> i32 {
    input * 2
}


#[no_mangle]
pub extern "C" fn calling_func(){
    call_async();
}

