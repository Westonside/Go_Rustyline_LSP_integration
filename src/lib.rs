mod multiLineState;
mod lsp_invoke;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, mpsc};
use std::string::String;
use std::borrow::Cow;
use std::io::Write;
use std::borrow::Cow::{Borrowed, Owned};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
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
        use unicode_width::UnicodeWidthStr;
        if self.masking {
            self.sending.send(line.to_string()).expect("TODO: panic message");
            // println!("test {}", line);
            Owned("*".repeat(line.width()))
        } else {
            Borrowed(line)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {

        self.masking
    }
}


fn newMain() -> Result<()>{
    //spawn the child
    let mut child = start_lsp();
    //send the first responses to the writer thread
    //the writer thread will have a channel with a vec that contains the starting commands
    //TODO: writer thread to the lsp
    //TODO: reader thread for the lsp
    //TODO: reader thread between rustyline that then processes the request and send to the writer thread


    //reading from rustyline input
    let (tx_stdin, rx_read_stdin): (Sender<String>, Receiver<String>) = channel();
    //sending the processed data onwards
    let (tx_processed, rx_processed): (Sender<String>, Receiver<String>) = channel();
    //use a semaphore to tell processor to stop sending until write receives (true meaning that it should send to writer thread)
    let mut processor_send = Arc::new(AtomicBool::new(true));
    //clone so that processor thread can have
    let mut processor_send_writer = Arc::clone(&processor_send);

    //spawning the editor with paste mode
    let mut paste: bool = false;
    let mut paste_state = MultiLineState::new();
    let mut rl = Editor::new();
    //helper
    let h = MaskingHighlighter {masking: false, sending: tx_stdin};
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
                println!("getting from the processed thread");
                let resp = rx_processed.recv().expect("failure getting from proccesor thread");
                write!(&mut child_stdin, "{}", resp).unwrap();
                //tell processor to send more after write
                processor_send.swap(true, Ordering::Relaxed);
            }
            // write!(child_stdin, "{}", headed.trim()).unwrap();
        })
    );


    //read from the LSP thread that will give the suggestions
    let child_stdout =  child.stdout.take().expect("failure getting the stdout");
    let reader = BufReader::new(child_stdout);
    thread_handlers.push(
        thread::spawn(move||{
            for line in reader.lines(){
                //read the response from the lsp and process

                println!("server read {}", line.unwrap());
            }
        })
    );

    //read from the user thread that will send to the processor thread
    thread_handlers.push(
        thread::spawn(move||{
            loop {
                println!("read from user:  {} ", rx_read_stdin.recv().expect("failure reading from user"));
            }
        })
    );

    //processing thread that will send to the writer thread after processing into a request

    //init array
    let init = ["initialize","initialized", "didOpen"];
    let mut res = init.iter().map(|x| formulate_request(x, "").unwrap()).collect::<VecDeque<String>>();
    // println!("new one {:?}",res);
    thread_handlers.push(
        thread::spawn(move||{
            while(res.len() != 0){
                let o = res.pop_front().unwrap();
                println!("popping and then sleep {} {}\n", o, processor_send_writer.load(Ordering::Relaxed) );
                //if false means writer is not ready to receive
                if !processor_send_writer.load(Ordering::Relaxed){
                    //context switch
                    thread::sleep(Duration::from_millis(10));
                }
                tx_processed.send(o).expect("panicked sending processed data");
                processor_send_writer.swap(false, Ordering::Relaxed);
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


fn rusty() -> Result<()> {

    let mut paste: bool = false;
    let mut paste_state = MultiLineState::new();
    // `()` can be used when no completer is required

    // let mut rl:Editor<()> = Editor::<()>::new();
    let mut rl = Editor::new();
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (tx_two, rx_two): (Sender<String>, Receiver<String>) = mpsc:: channel();

    let h = MaskingHighlighter { masking: false, sending: tx };
    // let h = InputValidator {};
    rl.set_helper(Some(h));

    // rl.bind_sequence(
    //     Event::Any,
    //     EventHandler::Conditional(Box::new(FilteringEventHandler)),
    // );
    // let mut help =


    let mut child = lsp_invoke::invoke_lsp();
    // let mut child_stdout =  child.stdout.as_mut().expect("failure number one stdout");
    // let reader = BufReader::new(child_stdout);



    //TODO: Get input from the user in the first thread from the channel receiver convert it to output and write it, that thread will block until it receives a formed request
    //TODO: Reader thread reads from stdout of the process and then parses the request (done concurrently)?
    //TODO: Make a thread that takes the stdin and receives requests then writes because only one place can have a lock on the stdin

    //make a new channel that gets the requests so get the inputted lines in terminal then send to another channel that forms the request

    //open doc notification occurs here
    // send_request(child, formulate_request("didOpen","").unwrap());

    //handlers for reading from the child process all reading happens here=
    let mut handlers = vec![];
    //reading thread
    // handlers.push(thread::spawn(move ||
    //     for line in reader.lines(){
    //         println!("Reading: {}", line.unwrap())
    //     }
    // ));

    //writing thread on update this thread needs to get a lock on the stdin and only this one, it will be the hub for writing every request
    //use channels to pass request and it will write then
    handlers.push(thread::spawn(move ||
        loop {
            println!("testing s asdf {} ", rx.recv().unwrap());
        }
    ));

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                // if paste {
                //     // rl.helper_mut().expect("no helper");
                //     rl.helper_mut().expect("No helper").masking = true;
                //
                // }
                rl.add_history_entry(line.as_str());
                if paste {
                    //add the line to the multiline state
                    rl.helper_mut().expect("No helper").masking = true;

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
    for h in handlers{
        h.join().unwrap();
    }
    rl.save_history("history.txt")

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

