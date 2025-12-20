use std::io;
use std::path::Path;

use lsp_types::{
    ClientCapabilities, DidOpenTextDocumentParams, InitializeParams, Position,
    TextDocumentIdentifier, TextDocumentItem, Url, WorkspaceFolder,
    notification::{DidOpenTextDocument, Notification},
    request::{Initialize, Request},
};
use serde_json::value::to_raw_value;

fn make_payload(data: &str) -> String {
    let len = data.len();
    format!("Content-Length: {len}\r\n\r\n{data}")
}

fn to_uri(path: &Path) -> Url {
    Url::from_file_path(path).unwrap()
}

pub fn init(id: i32, cwd: &Path) -> String {
    let client = ClientCapabilities {
        text_document: Some(lsp_types::TextDocumentClientCapabilities {
            synchronization: Some(lsp_types::TextDocumentSyncClientCapabilities {
                dynamic_registration: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    let params = InitializeParams {
        //workspace_folders: Some(vec![WorkspaceFolder { uri, name }]),
        capabilities: client,
        ..Default::default()
    };
    let req = jsonrpc::Request {
        jsonrpc: Some("2.0"),
        id: id.into(),
        method: Initialize::METHOD,
        params: Some(&to_raw_value(&params).unwrap()),
    };
    make_payload(&serde_json::to_string(&req).unwrap())
}

#[derive(serde::Serialize)]
struct NotificationRequest<'a> {
    jsonrpc: Option<&'a str>,
    method: &'a str,
    params: Option<serde_json::Value>,
}

pub fn did_open(path: &Path, source: String) -> String {
    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: to_uri(path),
            language_id: "rust".to_string(),
            version: 1,
            text: source,
        },
    };
    let req = NotificationRequest {
        jsonrpc: Some("2.0"),
        method: DidOpenTextDocument::METHOD,
        params: Some(serde_json::to_value(&params).unwrap()),
    };
    make_payload(&serde_json::to_string(&req).unwrap())
}

/// RustOwl does not implement [`serde::Serialize`] for `CursorRequest`
/// We will fix it
#[derive(serde::Serialize)]
pub struct CursorRequest {
    pub position: lsp_types::Position,
    pub document: lsp_types::TextDocumentIdentifier,
}
#[derive(serde::Serialize)]
pub struct AnalyzeRequest {}

pub fn analyze(id: i32) -> String {
    let req = jsonrpc::Request {
        jsonrpc: Some("2.0"),
        id: id.into(),
        method: "rustowl/analyze",
        params: Some(&to_raw_value(&AnalyzeRequest {}).unwrap()),
    };
    make_payload(&serde_json::to_string(&req).unwrap())
}

pub fn cursor(id: i32, path: &Path, line: u32, character: u32) -> String {
    let cursor = CursorRequest {
        position: Position { line, character },
        document: TextDocumentIdentifier { uri: to_uri(path) },
    };

    let req = jsonrpc::Request {
        jsonrpc: Some("2.0"),
        id: id.into(),
        method: "rustowl/cursor",
        params: Some(&to_raw_value(&cursor).unwrap()),
    };
    make_payload(&serde_json::to_string(&req).unwrap())
}

pub fn read(
    mut payload: Vec<u8>,
    mut read: impl io::Read,
) -> io::Result<(Vec<u8>, serde_json::Value)> {
    let content_length: usize;
    let body_starts: usize;
    loop {
        let mut headers = [httparse::EMPTY_HEADER; 1];
        match httparse::parse_headers(&payload, &mut headers)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "parse error"))?
        {
            httparse::Status::Complete((i, h)) => {
                content_length = h
                    .get(0)
                    .map(|v| String::from_utf8_lossy(v.value).trim().parse().ok())
                    .flatten()
                    .map_or(Err(io::Error::new(io::ErrorKind::Other, "parse error")), Ok)?;
                body_starts = i;
                break;
            }
            _ => {}
        }

        let mut buf = vec![0u8; 1024];
        let r = read.read(&mut buf)?;
        payload.extend_from_slice(&buf[0..r]);
    }
    while payload.len() - body_starts < content_length {
        let mut buf = vec![0u8; 1024];
        let r = read.read(&mut buf)?;
        payload.extend_from_slice(&buf[0..r]);
    }
    let s = String::from_utf8(payload[body_starts..body_starts + content_length].to_vec())
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "parse error"))?;
    Ok((
        payload[body_starts + content_length..].to_vec(),
        serde_json::from_str(&s)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "parse error"))?,
    ))
}
