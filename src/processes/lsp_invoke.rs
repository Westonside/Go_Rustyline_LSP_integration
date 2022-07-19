use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use std::{string, thread};
use std::fmt::{Debug, format};
use std::ptr::write;
use tower_lsp::{jsonrpc,Client};
use lsp_types::{ClientCapabilities, CompletionClientCapabilities, CompletionContext, CompletionItemCapability, CompletionParams, DidChangeTextDocumentParams, DidChangeWatchedFilesClientCapabilities, DidOpenTextDocumentParams, InitializedParams, InitializeParams, lsp_request, Position, PublishDiagnosticsClientCapabilities, Range, TextDocumentClientCapabilities, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, TextDocumentSyncClientCapabilities, Url, VersionedTextDocumentIdentifier, WorkDoneProgressParams, WorkspaceClientCapabilities, WorkspaceEditClientCapabilities};
use lsp_types::notification::{DidChangeTextDocument, DidOpenTextDocument, Initialized, Notification, PublishDiagnostics};
use serde_json::{json, json_internal, Value};
use serde_json::Result as Result_Json;


use tower_lsp::jsonrpc::Id::{Null, Number};
// use tower_lsp::jsonrpc::Request;
use std::str;
use std::string::ParseError;
use std::sync::mpsc;
use lsp_types::request::{Completion, Initialize, Request};
use serde_json::value::Serializer;
use tower_lsp::jsonrpc::{Method, RequestBuilder};
use crate::processes::lsp_invoke::LSP_Error::InvalidRequestType;
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

pub fn add_headers(a: String) -> String{
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


pub fn formulate_request (request_type: &str, text: &str)-> Result<String,LSP_Error>{
    match request_type {
        "initialize" => {
            let req: RequestBuilder = jsonrpc::Request::build(Initialize::METHOD).params(serde_json::to_value(InitializeParams {
                process_id: None,
                root_path: None,
                root_uri: Option::from(Url::parse("file:///foo.flux").unwrap()),
                initialization_options: None,
                capabilities: ClientCapabilities{
                    workspace: Some(WorkspaceClientCapabilities{
                        apply_edit: Some(true),
                        workspace_edit: Some(WorkspaceEditClientCapabilities{
                            document_changes: Some(true),
                            resource_operations: None,
                            failure_handling: None,
                            normalizes_line_endings: None,
                            change_annotation_support: None
                        }),
                        did_change_configuration: None,
                        did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities{ dynamic_registration: Some(true) }),
                        symbol: None,
                        execute_command: None,
                        workspace_folders: Some(false),
                        configuration: Some(true),
                        semantic_tokens: None,
                        code_lens: None,
                        file_operations: None
                    }),
                    text_document: Some(TextDocumentClientCapabilities{
                        synchronization: Some(TextDocumentSyncClientCapabilities{
                            dynamic_registration: Some(true),
                            will_save: Some(true),
                            will_save_wait_until: Some(true),
                            did_save: Some(true)
                        }),
                        completion: Some(CompletionClientCapabilities{
                            dynamic_registration: None,
                            completion_item: Some(CompletionItemCapability{
                                snippet_support: Some(true),
                                commit_characters_support: None,
                                documentation_format: None,
                                deprecated_support: None,
                                preselect_support: None,
                                tag_support: None,
                                insert_replace_support: None,
                                resolve_support: None,
                                insert_text_mode_support: None,
                            }),
                            completion_item_kind: None,
                            context_support: None
                        }),
                        hover: None,
                        signature_help: None,
                        references: None,
                        document_highlight: None,
                        document_symbol: None,
                        formatting: None,
                        range_formatting: None,
                        on_type_formatting: None,
                        declaration: None,
                        definition: None,
                        type_definition: None,
                        implementation: None,
                        code_action: None,
                        code_lens: None,
                        document_link: None,
                        color_provider: None,
                        rename: None,
                        publish_diagnostics: None,
                        folding_range: None,
                        selection_range: None,
                        linked_editing_range: None,
                        call_hierarchy: None,
                        semantic_tokens: None,
                        moniker: None
                    }) ,
                    window: None,
                    general: None,
                    experimental: None
                },
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
                        range: Some(Range{ start: Position{ line: 1, character: 0 }, end: Position{ line: 2, character: text.len() as u32 } }),
                        range_length: None,
                        text: text.to_string()
                    }]
                })?);
            let a = serde_json::to_value(req.finish())?;

            let headed = add_headers(serde_json::to_string(&a)?);
            Ok(headed)
        },
        "completion" =>{
            let req: RequestBuilder = jsonrpc::Request::build(Completion::METHOD).params(serde_json::to_value(
                CompletionParams{
                    text_document_position: TextDocumentPositionParams { text_document: TextDocumentIdentifier { uri: (Url::parse("file:///foo.flux").unwrap()) }, position: Position{ line: 1, character: text.len() as u32 } },
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                    context: Default::default()
                })?);
            let a = serde_json::to_value(req.finish())?;
            let headed = add_headers(serde_json::to_string(&a)?);
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


pub fn start_lsp() -> Child{
    //step one: start the process
    let mut child = Command::new("flux-lsp")
        // .arg("")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failure to execute");
    child
}





