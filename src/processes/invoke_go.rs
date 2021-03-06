use std::env::consts::OS;
use std::fmt::format;
use std::io::{Read, stdin};
use std::process::{Child, ChildStdout, Command, Stdio};
use regex::Regex;
use std::string::String;
use std::str::from_utf8;
use std::str;
use lsp_types::{DidChangeTextDocumentParams, TextDocumentContentChangeEvent, Url, VersionedTextDocumentIdentifier};
use serde_json::Value;
use tower_lsp::jsonrpc;
use tower_lsp::jsonrpc::RequestBuilder;
use crate::invoke_go::OutputError::InvalidMethod;
use crate::processes::lsp_invoke::add_headers;

// use std::simd::usizex2;

// pub struct FluxExecutor {
//     child: Child,
// }



    pub fn start_go() -> Child{
        let mut child = Command::new("./current_repl/flux_headlessrepl/flux")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failure to execute");
        child
    }

#[derive(Debug)]
    pub enum OutputError{
        InvalidMethod
    }


impl From<serde_json::Error> for OutputError{
    fn from(_: serde_json::Error) -> Self {
        InvalidMethod
    }
}

    pub fn form_output(request_type: &str, text: &str) -> Result<String, OutputError>{
        match request_type {
            "Service.DidOutput" =>{
                let mut cleaned = text.to_string().replace("\"","\\\"");
                let mut param = (r#"[{"input": "#).to_string();
                let mut other_side = format!(r#""{}""#, cleaned);
                other_side.push_str("}]");
                param.push_str(other_side.as_str());
                let paramm: Value = serde_json::from_str(param.as_str()).expect("failure going to value");




                let req: RequestBuilder = jsonrpc::Request::build("Service.DidOutput").params(paramm);
                let a = serde_json::to_value(req.finish())?;
                let res = serde_json::to_string(&a).unwrap();
                // println!("{}", res);
                // {"jsonrpc":"2.0", "method": "Service.DidOutput", "id": "1", "title":"testing","body":"dog", "params":[{"input":"x=1"}]}
                Ok(res)
            }
            _ =>{Err(InvalidMethod)}
        }
    }


    pub fn read_json_rpc(child_stdout: ChildStdout) {


        let re = Regex::new(r"Content-Length: ").unwrap();
        let num = Regex::new(r"\d").unwrap();
        let mut buf: Vec<u8> = vec![];
        let mut num_buf: Vec<u8> = vec![];
        let mut x = 0;
        let mut y = 0;
        //indicate when to start and stop capturing numbers in the content length
        let mut num_cap = false;
        let mut read_exact = (false, 0);
        for i in child_stdout.bytes() {
            let val = i.unwrap();
            let single = [val];
            if read_exact.0 {
                buf.insert(buf.len(), val);
                read_exact.1 = read_exact.1 - 1;
                if read_exact.1 == 0 {
                    //final result
                    println!("{}", str::from_utf8(&buf).unwrap());
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
                    //now read that many characters
                    let mut my_int: u16 = read.parse().unwrap();
                    //3 being the \r\n\n in the header
                    my_int = my_int + 3;
                    read_exact.0 = true;
                    read_exact.1 = my_int;
                    num_buf.clear();
                }
                buf.insert(buf.len(), val);
            }
            let cur = str::from_utf8(&buf).unwrap();
            let cl = str::from_utf8(&num_buf).unwrap();
            x = x + 1;
            y = y + 1;
            if !re.captures(cur).is_none() {
                num_cap = true;
            }
        }
    }



