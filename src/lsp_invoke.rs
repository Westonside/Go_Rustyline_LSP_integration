use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::{string, thread};
use tower_lsp::{jsonrpc,Client};
use lsp_types::{DidOpenTextDocumentParams, InitializeParams, lsp_request, TextDocumentItem, Url};
use lsp_types::notification::{DidOpenTextDocument, Initialized};
use serde_json::{json, Result, Value};
use tower_lsp::jsonrpc::Id::{Number, String};
// use tower_lsp::jsonrpc::Request;
use std::str;
use lsp_types::request::{Initialize, Request};
use serde_json::value::Serializer;
use tower_lsp::jsonrpc::RequestBuilder;
// use async_process::{Command, Stdio};
// use futures_lite::{io::BufReader, prelude::*};

// cmd := exec.Command("cargo", "run", "--manifest-path", "./flux-lsp/Cargo.toml")


pub fn invoke_lsp() {
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg("../../../ishell/ishellDevelop/flux-lsp/Cargo.toml")
        .stdin(Stdio::piped())
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failure to execute");

    let child_stdout = child.stdout.as_mut().unwrap();
    let mut child_stdin = child.stdin.unwrap();

    // let  v = json!({"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":0});

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


    let mut req: RequestBuilder = jsonrpc::Request::build(method_name);
    //maybe add id
    //call finish to get a Result


    let a = req.params(serde_json::to_value(&params).unwrap());
    let mut fin =  a.finish();



    // req.id()
    println!("{:?} serde", serde_json::to_string(&fin).unwrap());





    //generates the body
    println!("slmetning test {}", serde_json::to_string(&params).unwrap());
    //add the header



    let mut pool = scoped_threadpool::Pool::new(2);
    pool.scoped(|scope| {
        // read all output from the subprocess
        scope.execute(move || {
            use std::io::BufRead;
            let reader = BufReader::new(child_stdout);
            for line in reader.lines() {
                println!("tes {}", line.unwrap());
            }
        });

        // write to the subprocess

        scope.execute(move ||
            // println!("test")
            //               r#"{"jsonrpc": "2.0", "method": "say_hello", "params": [42, 23], "id": 1}"#;
            writeln!(&mut child_stdin, "{}", serde_json::to_string(&params).unwrap()).unwrap()
            //           writeln!(&mut child_stdin, "{}", r#"{"jsonrpc": "2.0", "method": "say_hello", "params": [42, 23], "id": 1}"#).unwrap()

                      // writeln!(&mut child_stdin, "{}", a).unwrap();

                      // child_stdin has been moved into this closure and is now
                      // dropped, closing it.
        );
    });


}