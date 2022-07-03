use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use std::{string, thread};
use std::fmt::{Debug, format};
use std::ptr::write;
use tower_lsp::{jsonrpc,Client};
use lsp_types::{DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializedParams, InitializeParams, lsp_request, TextDocumentContentChangeEvent, TextDocumentItem, Url, VersionedTextDocumentIdentifier};
use lsp_types::notification::{DidChangeTextDocument, DidOpenTextDocument, Initialized, Notification};
use serde_json::{json, json_internal, Value};
use serde_json::Result as Result_Json;


use tower_lsp::jsonrpc::Id::{Null, Number};
// use tower_lsp::jsonrpc::Request;
use std::str;
use std::string::ParseError;
use std::sync::mpsc;
use lsp_types::request::{Initialize, Request};
use serde_json::value::Serializer;
use tower_lsp::jsonrpc::{Method, RequestBuilder};
use crate::lsp_invoke::LSP_Error::InvalidRequestType;
// use async_process::{Command, Stdio};
// use futures_lite::{io::BufReader, prelude::*};

// cmd := exec.Command("cargo", "run", "--manifest-path", "./flux-lsp/Cargo.toml")



//LOOK at Rustyline event handler
//reading input as it inputted get a response back and get back into rustyline as to make a suggestion
//implement with channels
//experiment for each key stroke send to lsp bit

//getting back to Go stdin

//real messages to the lps
//document state

fn add_headers(a: String) -> String{
    format!("Content-Length: {}\r\n\r\n{}", a.len(), a)
}

fn process_response(a: &str) {
    // println!("testing 123 {} test", a);
    let js: Value = serde_json::to_value(a).unwrap();
    let result = js.get("result");
    // println!("{:?} result ", result);
}

#[derive(Debug)]
pub enum LSP_Error{
    Init_Error,
    InvalidRequestType,
    InvalidFormatting,
    InternalError
}

impl From<serde_json::Error> for LSP_Error{
    fn from(_: serde_json::Error) -> Self {
        Self::InvalidFormatting
    }

}
// impl From<url::parser::ParseError> for LSP_Error{
//     fn from(_: ParseError) -> Self {Self::InternalError}
// }



pub fn formulate_request (request_type: &str, text: &str)-> Result<String,LSP_Error>{
    match request_type {
        "initialize" => {
            let req: RequestBuilder = jsonrpc::Request::build(Initialize::METHOD).params(serde_json::to_value(InitializeParams {
                process_id: None,
                root_path: None,
                root_uri: None,
                initialization_options: None,
                capabilities: Default::default(),
                trace: None,
                workspace_folders: None,
                client_info: None,
                locale: None
            }).unwrap()).id(Number(1));
            // let fin =  req.finish();

            Ok(add_headers(serde_json::to_string(&req.finish())?))
        },
        "initialized" =>{
            let req: RequestBuilder = jsonrpc::Request::build(Initialized::METHOD).params(
                serde_json::to_value(InitializedParams{

                }).unwrap());
            Ok(add_headers(req.finish().to_string()))

        }

        "didOpen" => {
            // println!("getting here ");
            let req: RequestBuilder = jsonrpc::Request::build(DidOpenTextDocument::METHOD).params(serde_json::to_value(
                DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: Url::parse("file:///foo.flux").unwrap(),
                        language_id: "flux".to_string(),
                        version: 0,
                        text: "".to_string()
                    }
                })?);

            let a = serde_json::to_value(req.finish())?;

            let headed = add_headers(serde_json::to_string(&a)?);

            Ok(headed)
        },
        "didChange" =>{
            let req: RequestBuilder = jsonrpc::Request::build(DidChangeTextDocument::METHOD).params(serde_json::to_value(
                DidChangeTextDocumentParams{
                    text_document: VersionedTextDocumentIdentifier { uri: (Url::parse("file:///foo.flux").unwrap()), version: 0 },
                    content_changes: vec![TextDocumentContentChangeEvent{
                        range: None,
                        range_length: None,
                        text: text.to_string()
                    }]
                })?);
            // println!("i am reaching gthe line {}",text );
            let a = serde_json::to_value(req.finish())?;

            let headed = add_headers(serde_json::to_string(&a)?);
            // println!("headed for the didchange {}", headed);
            Ok(headed)
        }
        _ => {
            Err(InvalidRequestType)
        }
    }
}

pub fn send_request(mut child: Child, request: String){
    let mut child_stdin = child.stdin.as_mut().unwrap();

    write!(child_stdin, "{}", request).unwrap();
}

fn initial_handshake(mut child: Child, req: String) -> Result<(),LSP_Error>{
    //get the stdin and stdout
    let mut child_stdout = child.stdout.take().expect("epic fail");
    let mut child_stdin = child.stdin.take().expect( "failure getting the stdin");
    //get the start request
    let a = formulate_request("initialize", "")?;
    // let req = a.finish();


    // write!(&mut child_stdin, "{}", headed.trim()).unwrap();

    // thread::spawn(move || {
    //     for line in reader.lines() {
    //
    //         //response from the lsp
    //         println!("{}", line.unwrap());
    //     }
    // });
    // write!(&mut child_stdin, "{}", fin_two.trim()).unwrap();


    // for handling the resutl

    Ok(())
}

//adds a list of chores to the scheduler
pub fn generate_chores(chores: &mut [u8]){

}

pub fn start_lsp() -> Child{
    //step one: start the process
    let mut child = Command::new("flux-lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failure to execute");
    child
}



pub fn invoke_lsp() -> Child {
    let mut child = Command::new("flux-lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failure to execute");

    let method_name = Initialize::METHOD;
    // params
    let params = InitializeParams {
        process_id: None,
        root_path: None,
        root_uri: None,
        initialization_options: None,
        capabilities: Default::default(),
        trace: None,
        workspace_folders: None,
        client_info: None,
        locale: None
    };

    // lsp_request!("initialize");
    let mut req: RequestBuilder = jsonrpc::Request::build(method_name);
    let a = req.params(serde_json::to_value(&params).unwrap());
    let b  = a.id(Number(1));

    let mut fin =  b.finish();
    let fin_j = serde_json::to_value(fin).unwrap();
    let headed = add_headers(serde_json::to_string(&fin_j).unwrap());
    // println!("{}", lsp_request!("initialize"));

    //need to add headers




    // req.id()
    // println!("{}", headed);

    let params_two = InitializedParams{

    };
    let mut reqs  = jsonrpc::Request::build(Initialized::METHOD);
    let parammed = reqs.params(serde_json::to_value(&params_two).unwrap());
    let asdf = parammed.finish();
    let fin_two = add_headers(asdf.to_string());
    // println!("\n{}", fin_two);


    // let mut child_stdout = child.stdout.take().expect("epic fail");
    let mut child_stdin = child.stdin.as_mut().unwrap();

    write!(child_stdin, "{}", headed.trim()).unwrap();
    // let reader = BufReader::new(child_stdout);


    // thread::spawn(move || {
    //     for line in reader.lines() {
    //         //TODO: Need to handle response from the lsp
    //         println!("{}", line.unwrap());
    //     }
    // });
    write!(child_stdin, "{}", fin_two.trim()).unwrap();

    // need to indicate that a document has been opened
    //create a url
    child


    // loop {
    //     let mut r = [0;1024];
    //     child_stdout.read(&r);
    //     println!("reading {} ", String::from_utf8(Vec::from(r)).unwrap());
    // }












    // let mut pool = scoped_threadpool::Pool::new(1);

    // pool.scoped(|scope| {
    //     // read all output from the subprocess
    //     scope.execute(move || {
    //         use std::io::BufRead;
    //         let reader = BufReader::new(child_stdout);
    //         for line in reader.lines() {
    //             println!("{}", line.unwrap());
    //         }
    //     });
    //
    //     // write to the subprocess
    //
    //     // scope.execute(move ||
    //     //
    //     //     writeln!(&mut child_stdin, "{}", headed).unwrap()
    //     //
    //     // );
    // });


}