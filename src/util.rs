use reqwest::Response;
use serde_json::{self, Value};
use std::io::{self, Write};

use error::Result;

/// Print HTTP response to stdout.
pub fn print_resp(mut resp: Response) -> Result<()> {
    match resp.json() {
        Ok(json) => print_json(json),
        Err(err) => Ok(error!("Parsing HTTP response: {}", err)),
    }
}

/// Pretty-print JSON to stdout.
pub fn print_json(json: Value) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write(serde_json::to_string_pretty(&json)?.as_bytes());
    Ok(())
}
