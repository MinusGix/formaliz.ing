use crate::build::{self, BuildOptions};
use anyhow::{Result, anyhow};
use notify::{EventKind, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Duration;
use tiny_http::{Header, Request, Response, Server};

/// Builds `site` into `target/serve/<site>`, serves it, and rebuilds on changes.
/// Pages poll `/__livereload` and refresh when the build generation changes.
pub fn serve(root: &Path, site: &str, port: u16, drafts: bool) -> Result<()> {
    let root = root.canonicalize()?;
    let out = root.join("target").join("serve").join(site);
    let opts = BuildOptions {
        drafts,
        base_url: Some(format!("http://localhost:{port}")),
        live_reload: true,
    };
    let generation = Arc::new(AtomicU64::new(0));
    rebuild(&root, site, &out, &opts);

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res
            && !matches!(event.kind, EventKind::Access(_))
        {
            let _ = tx.send(());
        }
    })?;
    watcher.watch(&root.join("sites").join(site), RecursiveMode::Recursive)?;
    watcher.watch(&root.join("theme"), RecursiveMode::Recursive)?;
    {
        let (root, site, out, generation) =
            (root.clone(), site.to_string(), out.clone(), generation.clone());
        std::thread::spawn(move || {
            while rx.recv().is_ok() {
                // Editors write in bursts; wait for a quiet moment.
                while rx.recv_timeout(Duration::from_millis(120)).is_ok() {}
                rebuild(&root, &site, &out, &opts);
                generation.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    let server = Server::http(("127.0.0.1", port)).map_err(|e| anyhow!(e))?;
    println!("Serving {site} at http://localhost:{port}/  (ctrl-c to stop)");
    for request in server.incoming_requests() {
        handle(request, &out, &generation);
    }
    drop(watcher);
    Ok(())
}

fn rebuild(root: &Path, site: &str, out: &Path, opts: &BuildOptions) {
    let start = std::time::Instant::now();
    match build::build(root, site, out, opts) {
        Ok(n) => println!("built {site}: {n} entries in {:.0?}", start.elapsed()),
        Err(e) => eprintln!("build failed: {e:#}"),
    }
}

fn handle(request: Request, out: &Path, generation: &AtomicU64) {
    let raw = request.url().split(['?', '#']).next().unwrap_or("/").to_string();
    if raw == "/__livereload" {
        let body = generation.load(Ordering::SeqCst).to_string();
        let _ = request.respond(Response::from_string(body).with_header(no_store()));
        return;
    }
    let path = percent_decode(&raw);
    if path.split('/').any(|seg| seg == "..") {
        let _ = request.respond(Response::from_string("bad path").with_status_code(400));
        return;
    }
    let mut file: PathBuf = out.join(path.trim_start_matches('/'));
    if file.is_dir() {
        if !path.ends_with('/') {
            let location = Header::from_bytes("Location", format!("{path}/")).unwrap();
            let _ = request.respond(Response::empty(301).with_header(location));
            return;
        }
        file = file.join("index.html");
    }
    let response = match fs::read(&file) {
        Ok(bytes) => Response::from_data(bytes).with_header(content_type(&file)),
        Err(_) => Response::from_data(fs::read(out.join("404.html")).unwrap_or_default())
            .with_status_code(404)
            .with_header(content_type(Path::new("404.html"))),
    };
    let _ = request.respond(response.with_header(no_store()));
}

fn no_store() -> Header {
    Header::from_bytes("Cache-Control", "no-store").unwrap()
}

fn content_type(path: &Path) -> Header {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let mime = match ext {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "xml" => "application/atom+xml; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "pdf" => "application/pdf",
        "txt" | "" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    Header::from_bytes("Content-Type", mime).unwrap()
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(byte) = s
                .get(i + 1..i + 3)
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
