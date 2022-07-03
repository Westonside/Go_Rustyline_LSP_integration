mod multiLineState;
mod lsp_invoke;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, mpsc};
use std::string::String;
use std::borrow::Cow;
use std::io::{Read, Write};
use std::borrow::Cow::{Borrowed, Owned};
use std::collections::VecDeque;
// use std::intrinsics::atomic_and;
// use std::io::{BufRead, BufReader};
use std::io::{self, BufRead};
use std::ops::Add;
use std::str;

// use std::simd::usizex2;

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use regex::{Captures, Regex};
use std::time::Duration;
// use lsp_types::MarkedString::String;
// use std::si md::Mask;
use rustyline_derive::{Completer, Helper, Highlighter, Hinter, Validator};
use rustyline::error::ReadlineError;
// use rustyline::{Editor, Event, EventHandler, KeyEvent, Result};
use rustyline::highlight::Highlighter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
// use rustyline::{Editor, , Result};
use rustyline::{
    Cmd, ConditionalEventHandler, Editor, Event, EventContext, EventHandler, KeyCode, KeyEvent,
    Modifiers, RepeatCount, Result,
};
use multiLineState::MultiLineState;
use crate::lsp_invoke::{formulate_request, generate_chores, send_request, start_lsp};


// struct FilteringEventHandler;
// impl ConditionalEventHandler for FilteringEventHandler {
//     fn handle(&self, evt: &Event, _: RepeatCount, _: bool, _: &EventContext) -> Option<Cmd> {
//         if let Some(KeyEvent(KeyCode::Char(c), m)) = evt.get(0) {
//             if m.contains(Modifiers::CTRL) || m.contains(Modifiers::ALT) || c.is_ascii_digit() {
//                 None
//             } else {
//                 Some(Cmd::Noop) // filter out invalid input
//             }
//         } else {
//             None
//         }
//     }
// }


#[derive(Completer, Helper, Hinter, Validator)]
struct  MaskingHighlighter {
    masking: bool,
    sending: Sender<String>
}

impl Highlighter for MaskingHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
            self.sending.send(line.to_string()).expect("TODO: panic message");
            Borrowed(line)

    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {

        self.masking
    }
}


fn newMain() -> Result<()>{
    //ISSUE AT THE MOMENT: reader is not reading the full response of the lsp response so it is behind somewhat
    //spawn the child
    let mut child = start_lsp();
    //send the first responses to the writer thread
    //the writer thread will have a channel with a vec that contains the starting commands
    //TODO: writer thread to the lsp
    //TODO: reader thread for the lsp
    //TODO: reader thread between rusty-line that then processes the request and send to the writer thread


    //reading from rustyline input
    let (tx_stdin, rx_read_stdin): (Sender<String>, Receiver<String>) = channel();
    //sending the processed data onwards
    let (tx_processed, rx_processed): (Sender<String>, Receiver<String>) = channel();


    let mut reader_block = Arc::new(AtomicBool::new(false));
    let mut reader_block_w = Arc::clone(&reader_block);
    let mut reader_block_p = Arc::clone(&reader_block);







    //spawning the editor with paste mode
    let mut paste: bool = false;
    let mut paste_state = MultiLineState::new();
    let mut rl = Editor::new();
    //helper
    let h = MaskingHighlighter {masking: true, sending: tx_stdin};
    rl.set_helper(Some(h));

    //spawn the lsp
    let mut child = start_lsp();

    //thread handler
    let mut thread_handlers = vec![];

    //first spawn the writing thread nothing else can access the stdin if you take
    //reads from the processed thread
    let mut child_stdin = child.stdin.take().unwrap();
    thread_handlers.push(
        thread::spawn(move||{
            //read the processed request then write the request to the LSP
            loop{
                //block if just sent
                if reader_block_w.load(Ordering::Relaxed){
                    thread::sleep(Duration::from_millis(10));
                }
                let resp = rx_processed.recv().expect("failure getting from processor thread");
                write!(&mut child_stdin, "{}", resp).unwrap();
                reader_block_w.swap(true, Ordering::Relaxed);

            }
        })
    );


    //(b"<string>")
    //read from the LSP thread that will give the suggestions
    let mut child_stdout =  child.stdout.take().expect("failure getting the stdout");
    // let mut cursor = io::Cursor::new(child_stdout.);
    thread_handlers.push(
        thread::spawn(move||{
            let re = Regex::new(r"Content-Length: ").unwrap();
            let num = Regex::new(r"\d").unwrap();
                    let mut buf: Vec<u8> = vec![];
                    let mut num_buf: Vec<u8> = vec![];
                    let mut x = 0;
                    let mut y = 0;
                    //indicate when to start and stop capturing numbers in the content length
                    let mut num_cap = false;
                    let mut read_exact = (false,0);
                    for i in child_stdout.bytes() {
                        let val = i.unwrap();
                        let single = [val];
                        if read_exact.0{
                            buf.insert(buf.len(),val);
                            read_exact.1 = read_exact.1 - 1;
                            if read_exact.1 == 0{
                                println!("full response {}", str::from_utf8(&buf).unwrap());
                                read_exact.0 = false;
                                // break;
                            }
                            continue;
                        }

                        let a = str::from_utf8(&single).unwrap();
                        //if capturing numbers and the value is numeric add to number buffer
                        if num_cap && num.is_match(a) {
                            num_buf.insert(num_buf.len(), val);
                        } else {
                            if num_cap {
                                //indicate you need to take that number and read that many bytes
                                num_cap = false;
                                buf.clear();
                                let read = str::from_utf8(&num_buf).unwrap();
                                println!("that is the number read! !! {}a", read);
                                //now read that many characters
                                let mut my_int: u16 = read.parse().unwrap();
                                //3 being the \r\n\n in the header
                                my_int = my_int+3;
                                read_exact.0 = true;
                                read_exact.1 = my_int;
                                num_buf.clear();

                                //read that many bytes and go again
                                // let mut resp = Vec::with_capacity(my_int as usize);

                                // break;
                            }
                            buf.insert(buf.len(), val);
                        }
                        let cur = str::from_utf8(&buf).unwrap();
                        let cl = str::from_utf8(&num_buf).unwrap();
                        x = x + 1;
                        y = y + 1;
                        if !re.captures(cur).is_none(){
                            num_cap = true;
                        }

                    }
        })
    );


    //processing thread that will send to the writer thread after processing into a request
    //init array
    let init = ["initialize","initialized", "didOpen"];
    let mut res = init.iter().map(|x| formulate_request(x, "").unwrap()).collect::<VecDeque<String>>();
    thread_handlers.push(
        thread::spawn(move||{



            //inintalize
            while(res.len() != 0){
                if reader_block_p.load(Ordering::Relaxed){
                    thread::sleep(Duration::from_millis(1));
                }
                let o = res.pop_front().unwrap();



                tx_processed.send(o).expect("panicked sending processed data to writer thread");
            }
            //getting data from the user thread read from the reading
            loop{
                // if reader_block_p.load(Ordering::Relaxed){
                //     thread::sleep(Duration::from_millis(10));
                // }
                // let input = rx_user.recv().expect("failure receiving from the user input thread");
                let input = rx_read_stdin.recv().expect("failure reading from the user");
                //create a document change request from the input and capture the line
                tx_processed.send(formulate_request("didChange", &input).expect("incorrect request type in processor")).expect("failure sending to writer thread pt 2");
                // println!("i am here processor");
                // thread::sleep(Duration::from_millis(1));

            }

        })
    );


    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {

                rl.add_history_entry(line.as_str());
                if paste {
                    //add the line to the multiline state
                    rl.helper_mut().expect("No helper").masking = true;

                    paste_state.addRecord(line.to_string());

                }

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
                    // rl.helper_mut().expect("No helper").masking = true;
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
    for h in thread_handlers{
        h.join().expect("joining failed");
    }
    rl.save_history("history.txt")


    // Ok(())
}




fn call_async(){
    newMain();
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

