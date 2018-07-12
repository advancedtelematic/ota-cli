use reqwest::{
    header::{Authorization, Bearer},
    Client,
    Method,
    Request,
    Response,
    Url,
};
use serde_json::{self, Value};
use std::io::{self, Read, Write};

use api::auth_plus::AccessToken;
use error::{Error, Result};


/// Convenience methods for making simple HTTP requests.
pub trait HttpMethods {
    fn get(url: impl AsRef<str>, token: Option<AccessToken>) -> Result<Response> { Http::send(Request::new(Method::Get, Url::parse(url.as_ref())?), token) }
    fn post(url: impl AsRef<str>, token: Option<AccessToken>) -> Result<Response> { Http::send(Request::new(Method::Post, Url::parse(url.as_ref())?), token) }
    fn put(url: impl AsRef<str>, token: Option<AccessToken>) -> Result<Response> { Http::send(Request::new(Method::Put, Url::parse(url.as_ref())?), token) }
    fn delete(url: impl AsRef<str>, token: Option<AccessToken>) -> Result<Response> { Http::send(Request::new(Method::Delete, Url::parse(url.as_ref())?), token) }
}

/// Make HTTP requests to server endpoints.
pub struct Http;

impl HttpMethods for Http {}

impl Http {
    /// Send an HTTP request with an optional bearer token.
    pub fn send(mut req: Request, token: Option<AccessToken>) -> Result<Response> {
        Self::add_headers(&mut req, token);
        if let Some(body) = req.body() {
            debug!("request body:\n{:?}\n", body);
        }
        Client::new().execute(req).map_err(Error::Http)
    }

    /// Add a bearer token and `x-ats-namespace` header when available.
    fn add_headers(req: &mut Request, token: Option<AccessToken>) -> () {
        let headers = req.headers_mut();
        if headers.len() > 0 {
            debug!("request headers:\n{}", headers);
        }

        if let Some(t) = token {
            debug!("request token scopes: {}", t.scope);
            headers.set(Authorization(Bearer { token: t.access_token.clone() }));
            t.namespace()
                .map(|ns| headers.set_raw("x-ats-namespace", ns))
                .unwrap_or_else(|err| error!("reading namespace: {}", err));
        }
    }

    /// Print the HTTP response to stdout.
    pub fn print_response(mut resp: Response) -> Result<()> {
        let mut body = Vec::new();
        let _ = resp.read_to_end(&mut body)?;
        debug!("response headers:\n{}", resp.headers());

        #[cfg_attr(rustfmt, rustfmt_skip)] 
        match serde_json::from_slice::<Value>(&body) {
            Ok(json) => print_bytes(serde_json::to_string_pretty(&json)?),
            Err(_)   => print_bytes(body),
        }
    }
}

/// Print bytes to stdout.
fn print_bytes(data: impl AsRef<[u8]>) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write(data.as_ref());
    Ok(())
}
