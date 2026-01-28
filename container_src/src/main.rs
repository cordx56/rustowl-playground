use axum::{
    Json, Router,
    response::Html,
    routing::{get, post},
};
use std::env;
use std::io;
use std::process;
use tokio::{fs, io::AsyncWriteExt, time};
use uuid::Uuid;

mod lsp;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(async || Html("OK")))
        .route("/api/analyze", post(analyze));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("http://localhost:3000/analyze");
    axum::serve(listener, app).await.unwrap();
}

#[derive(serde::Deserialize)]
struct RequestBody {
    source: String,
    line: u32,
    character: u32,
}
#[derive(serde::Serialize)]
struct ResponseErrorBody {
    message: String,
}

async fn analyze(
    body: Json<RequestBody>,
) -> Result<Json<serde_json::Value>, Json<ResponseErrorBody>> {
    let mut cmd = process::Command::new("rustowl")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            Json(ResponseErrorBody {
                message: e.to_string(),
            })
        })?;
    let decoration = do_analyze(
        cmd.stdin.as_mut().unwrap(),
        cmd.stdout.as_mut().unwrap(),
        body.source.clone(),
        body.line,
        body.character,
    )
    .await
    .map_err(|e| {
        Json(ResponseErrorBody {
            message: e.to_string(),
        })
    })?;
    let _ = cmd.kill();
    Ok(Json(decoration))
}

async fn do_analyze(
    mut send: impl io::Write,
    mut recv: impl io::Read,
    source: String,
    line: u32,
    character: u32,
) -> io::Result<serde_json::Value> {
    let cwd = env::current_dir().unwrap();
    let id = Uuid::new_v4();
    let path = cwd.join(format!("{}.rs", id.to_string()));

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path)
        .await?;
    file.write_all(source.as_bytes()).await?;
    drop(file);

    let mut buffer = Vec::with_capacity(4096);

    send.write_all(lsp::init(10, &cwd).as_bytes())?;
    time::sleep(time::Duration::from_millis(300)).await;

    send.write_all(lsp::did_open(&path, source).as_bytes())?;
    time::sleep(time::Duration::from_millis(300)).await;

    send.write_all(lsp::analyze(30).as_bytes())?;
    time::sleep(time::Duration::from_millis(300)).await;

    send.write_all(lsp::cursor(40, &path, line, character).as_bytes())?;

    let mut result;
    loop {
        (buffer, result) = lsp::read(buffer, &mut recv)?;
        if result
            .as_object()
            .map(|v| v.get("id"))
            .flatten()
            .map(|v| v.as_number())
            .flatten()
            .map(|v| v.as_u64())
            .flatten()
            == Some(40)
        {
            break;
        }
    }
    let _ = fs::remove_file(&path).await;
    result
        .as_object()
        .map(|v| v.get("result").cloned())
        .flatten()
        .map_or(
            Err(io::Error::new(io::ErrorKind::Other, "result error")),
            |v| Ok(v),
        )
}
