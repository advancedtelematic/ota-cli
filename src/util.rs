use env_logger::Builder;
use log::LevelFilter;
use reqwest::Response;
use serde_json::{self, Value};
use std::io::{self, Read, Write};

use error::Result;


/// Start logging at the specified level.
pub fn start_logging(level: &str) {
    Builder::from_default_env()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .parse(level)
        .filter(Some("tokio"), LevelFilter::Info)
        .init();
}

/// Print HTTP response to stdout.
pub fn print_resp(mut resp: Response) -> Result<()> {
    let mut body = Vec::new();
    let _ = resp.read_to_end(&mut body)?;
    trace!(
        "response status: {}\nresponse length: {}\nresponse headers:\n{}",
        resp.status(),
        body.len(),
        resp.headers()
    );
    if let Ok(json) = serde_json::from_slice::<Value>(&body) {
        print_bytes(serde_json::to_string_pretty(&json)?.as_bytes())
    } else {
        print_bytes(&body)
    }
}

pub fn print_bytes(data: &[u8]) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write(data);
    Ok(())
}
