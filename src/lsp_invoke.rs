use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::{string, thread};
use std::fmt::format;
use std::ptr::write;
use tower_lsp::{jsonrpc,Client};
use lsp_types::{DidOpenTextDocumentParams, InitializedParams, InitializeParams, lsp_request, TextDocumentItem, Url};
use lsp_types::notification::{DidOpenTextDocument, Initialized, Notification};
use serde_json::{json, Result, Value};
use tower_lsp::jsonrpc::Id::{Null, Number};
// use tower_lsp::jsonrpc::Request;
use std::str;
use std::sync::mpsc;
use lsp_types::request::{Initialize, Request};
use serde_json::value::Serializer;
use tower_lsp::jsonrpc::RequestBuilder;
// use async_process::{Command, Stdio};
// use futures_lite::{io::BufReader, prelude::*};

// cmd := exec.Command("cargo", "run", "--manifest-path", "./flux-lsp/Cargo.toml")

fn add_headers(a: String) -> String{
    format!("Content-Length: {}\r\n\r\n{}", a.len(), a)
}

fn process_response(a: &str) {
    // println!("testing 123 {} test", a);
    let js: Value = serde_json::to_value(a).unwrap();
    let result = js.get("result");
    // println!("{:?} result ", result);
}
pub fn invoke_lsp() {
    let mut child = Command::new("../../../ishell/ishellDevelop/flux-lsp/target/debug/flux-lsp")
        // .arg("run")
        // .arg("--manifest-path")
        // .arg()
        // .arg("--")
        // .arg("--log-file")
        // .arg("~/Documents/Influx/ffi_test/test_two/")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failure to execute");



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
    let a = req.params(serde_json::to_value(&params).unwrap());
    let b  = a.id(Number(1));

    let mut fin =  b.finish();
    let fin_j = serde_json::to_value(fin).unwrap();
    let headed = add_headers(serde_json::to_string(&fin_j).unwrap());


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


    let mut child_stdout = child.stdout.take().expect("epic fail");
    let mut child_stdin = child.stdin.take().expect("failure getting the stdin");

    write!(&mut child_stdin, "{}", headed.trim()).unwrap();
    // writeln!(&mut child_stdin, "{}", fin_two).unwrap();
    let reader = BufReader::new(child_stdout);

    thread::spawn(move || {
        for line in reader.lines() {
            println!("{}", line.unwrap());
        }
    });
    write!(&mut child_stdin, "{}", fin_two.trim()).unwrap();



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