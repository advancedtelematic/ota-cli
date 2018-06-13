use serde_json::{self, Value};
use std::io::{self, Write};

use error::Error;

/// Pretty-print JSON to stdout.
pub fn print_json(resp: Value) -> Result<(), Error> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write(serde_json::to_string_pretty(&resp)?.as_bytes());
    Ok(())
}
