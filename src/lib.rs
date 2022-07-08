mod multiLineState;
// mod invoke_go;
mod processes;

use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc};
use std::string::String;
use std::borrow::Cow;
use std::io::{BufReader, Read, Write};
use std::borrow::Cow::{Borrowed, Owned};
use std::collections::{HashSet, VecDeque};
use crate::processes::{start_go, invoke_go};
use std::io::{self, BufRead};
use std::ops::Add;
use std::str;


// use std::simd::usizex2;

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use regex::{Captures, Regex};
use std::time::Duration;

// use rustyline_derive::{Completer, Helper, Highlighter, Hinter, Validator};
use rustyline::error::ReadlineError;
// use rustyline::{Editor, Event, EventHandler, KeyEvent, Result};
use rustyline::highlight::Highlighter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
// use rustyline::{Editor, , Result};
use rustyline::{Cmd, Config, CompletionType, ConditionalEventHandler, Context, EditMode, Editor, Event, EventContext, EventHandler, KeyCode, KeyEvent, Modifiers, RepeatCount, Result};
use rustyline::completion::Completer;
use rustyline::hint::{Hint, Hinter, HistoryHinter};
use multiLineState::MultiLineState;
use processes::lsp_invoke::{formulate_request, send_request, start_lsp};



// #[derive(Completer, Helper, Hinter, Validator)]
// struct  MaskingHighlighter {
//     masking: bool,
//     sending: Sender<String>
// }
//
//
//
// #[derive(Helper, Completer, Hinter, Validator, Highlighter)]
// struct MyHelper {
//     highlighter: MaskingHighlighter,
//     completer: CompleteHintHandler,
//     third: HistoryHinterTwo
// }
//
//
// #[derive(Clone)]
// struct CompleteHintHandler;
// impl ConditionalEventHandler for CompleteHintHandler {
//     fn handle(&self, evt: &Event, _: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
//         if !ctx.has_hint() {
//             return None; // default
//         }
//         if let Some(k) = evt.get(0) {
//             #[allow(clippy::if_same_then_else)]
//             if *k == KeyEvent::ctrl('E') {
//                 Some(Cmd::CompleteHint)
//             } else if *k == KeyEvent::alt('f') && ctx.line().len() == ctx.pos() {
//                 let text = ctx.hint_text()?;
//                 let mut start = 0;
//                 if let Some(first) = text.chars().next() {
//                     if !first.is_alphanumeric() {
//                         start = text.find(|c: char| c.is_alphanumeric()).unwrap_or_default();
//                     }
//                 }
//
//                 let text = text
//                     .chars()
//                     .enumerate()
//                     .take_while(|(i, c)| *i <= start || c.is_alphanumeric())
//                     .map(|(_, c)| c)
//                     .collect::<String>();
//
//                 Some(Cmd::Insert(1, text))
//             } else {
//                 None
//             }
//         } else {
//             unreachable!()
//         }
//     }
// }
//
// struct TabEventHandler;
// impl ConditionalEventHandler for TabEventHandler {
//     fn handle(&self, evt: &Event, n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
//         debug_assert_eq!(*evt, Event::from(KeyEvent::from('\t')));
//         if ctx.line()[..ctx.pos()]
//             .chars()
//             .rev()
//             .next()
//             .filter(|c| c.is_whitespace())
//             .is_some()
//         {
//             Some(Cmd::SelfInsert(n, '\t'))
//         } else {
//             None // default complete
//         }
//     }
// }
//
//
//
//
//
// impl Highlighter for MaskingHighlighter {
//     fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
//             self.sending.send(line.to_string()).expect("TODO: panic message");
//             Borrowed(line)
//
//     }
//
//     fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
//
//         self.masking
//     }
// }
use rustyline_derive::{Completer, Helper, Validator};


#[derive(Completer, Helper, Validator)]
struct MyHelper(HistoryHinter);

impl Hinter for MyHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.0.hint(line, pos, ctx)
    }
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Owned(format!("\x1b[1;32m{}\x1b[m", prompt))
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("\x1b[1m{}\x1b[m", hint))
    }
}

#[derive(Clone)]
struct CompleteHintHandler;
impl ConditionalEventHandler for CompleteHintHandler {
    fn handle(&self, evt: &Event, _: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        if !ctx.has_hint() {
            return None; // default
        }
        if let Some(k) = evt.get(0) {
            #[allow(clippy::if_same_then_else)]
            if *k == KeyEvent::ctrl('E') {
                Some(Cmd::CompleteHint)
            } else if *k == KeyEvent::alt('f') && ctx.line().len() == ctx.pos() {
                let text = ctx.hint_text()?;
                let mut start = 0;
                if let Some(first) = text.chars().next() {
                    if !first.is_alphanumeric() {
                        start = text.find(|c: char| c.is_alphanumeric()).unwrap_or_default();
                    }
                }

                let text = text
                    .chars()
                    .enumerate()
                    .take_while(|(i, c)| *i <= start || c.is_alphanumeric())
                    .map(|(_, c)| c)
                    .collect::<String>();

                Some(Cmd::Insert(1, text))
            } else {
                None
            }
        } else {
            unreachable!()
        }
    }
}

struct TabEventHandler;
impl ConditionalEventHandler for TabEventHandler {
    fn handle(&self, evt: &Event, n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        debug_assert_eq!(*evt, Event::from(KeyEvent::from('\t')));
        if ctx.line()[..ctx.pos()]
            .chars()
            .rev()
            .next()
            .filter(|c| c.is_whitespace())
            .is_some()
        {
            Some(Cmd::SelfInsert(n, '\t'))
        } else {
            None // default complete
        }
    }
}




fn newMain() -> Result<()>{
    let config = rustyline::Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        // .edit_mode(EditMode::Emacs)
        .build();


    //reading from rustyline input
    let (tx_stdin, rx_read_stdin): (Sender<String>, Receiver<String>) = channel();
    //sending the processed data onwards
    let (tx_processed, rx_processed): (Sender<String>, Receiver<String>) = channel();
    //sending from when user presses enter
    let (tx_user, rx_user): (Sender<String>, Receiver<String>) = channel();

    //TODO: ADD BACK IN
    // let ha = MyHelper{
    //     highlighter: MaskingHighlighter {masking: true, sending: tx_stdin},
    //     completer: CompleteHintHandler{},
    //     third: MyHelperTwo
    // };





    let mut reader_block = Arc::new(AtomicBool::new(false));
    let mut reader_block_w = Arc::clone(&reader_block);
    let mut reader_block_p = Arc::clone(&reader_block);


    //spawning the editor with paste mode
    let mut paste: bool = false;
    let mut paste_state = MultiLineState::new();
    // let mut rl = Editor::new();
    //TODO: ADD BACK
    // let mut rl = Editor::with_config(config);

    let mut rl = Editor::<MyHelper>::new();
    rl.set_helper(Some(MyHelper(HistoryHinter {})));
    let ceh = Box::new(CompleteHintHandler);
    rl.bind_sequence(KeyEvent::ctrl('E'), EventHandler::Conditional(ceh.clone()));
    rl.bind_sequence(KeyEvent::alt('f'), EventHandler::Conditional(ceh));
    rl.bind_sequence(
        KeyEvent::from('\t'),
        EventHandler::Conditional(Box::new(TabEventHandler)),
    );


    // let h = DIYHinter {hints: diy_hints()};
    // let mut rl: Editor<DIYHinter> = Editor::new();
    // rl.set_helper(Some(h));



    //TODO: ADD BACK
    // rl.set_helper(Some(ha));
    // rl.set_helper(Some(MyHelper(HistoryHinter{})));
    //helper
    // let h = MaskingHighlighter {masking: true, sending: tx_stdin};
    // rl.set_helper(Some(h));

    //spawn the lsp
    let mut child = start_lsp();
    //spawn the flux runner
    let mut flux_child = start_go();


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
                // println!("{}", resp);
                write!(&mut child_stdin, "{}", resp).unwrap();
                reader_block_w.swap(true, Ordering::Relaxed);

            }
        })
    );


    //read from the LSP thread that will give the suggestions
    thread_handlers.push(
        thread::spawn(move||{
            invoke_go::read_json_rpc(child.stdout.take().expect("failure getting the stdout"));
        })
    );

    // getting when the user presses enter to send to the flux runner
    let mut flux_stdin = flux_child.stdin.take().expect("failure getting the stdin of the flux");
    thread_handlers.push(
        thread::spawn(move||{
            loop {
                let resp = rx_user.recv().expect("Failure receiving the user's input");
                //format what is received
                let message = invoke_go::form_output("Service.DidOutput", &resp).expect("failure making message for flux");
                write!(flux_stdin, "{}" ,message).expect("failed to write to the flux run time");
            }
        })
    );

    let mut flux_stdout = flux_child.stdout.take().expect("failure getting the stoud of the flux");
    let reader = BufReader::new(flux_stdout);
    thread_handlers.push(
        thread::spawn(move||{
            for line in reader.lines(){
                println!("{}", line.unwrap())
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
            while res.len() != 0 {
                if reader_block_p.load(Ordering::Relaxed){
                    thread::sleep(Duration::from_millis(1));
                }
                let o = res.pop_front().unwrap();



                tx_processed.send(o).expect("panicked sending processed data to writer thread");
            }
            //getting data from the user thread read from the reading
            loop{
                let input = rx_read_stdin.recv().expect("failure reading from the user");
                tx_processed.send(formulate_request("didChange", &input).expect("incorrect request type in processor")).expect("failure sending to writer thread pt 2");
            }

        })
    );


    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {

                // rl.add_history_entry(line.as_str());
                if paste {
                    //add the line to the multiline state
                    //TODO: ADD BACK IN
                    // rl.helper_mut().unwrap().highlighter.masking = true;


                    paste_state.addRecord(line.to_string());

                }

                println!("Line: {}", line);
                rl.add_history_entry(line.as_str());
                if !paste{
                    tx_user.send(line).expect("Failure getting user input!");
                }

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
                    // rl.add_history_entry(paste_state.resultString());
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

    // rl.save_history("history.txt")

    Ok(())

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

