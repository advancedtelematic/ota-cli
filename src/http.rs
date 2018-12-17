use reqwest::{Client, RequestBuilder, Response, Url};
use serde_json::{self, Value};
use std::io::{self, Read};

use api::auth_plus::AccessToken;
use error::{Error, Result};


/// Convenience methods for making simple HTTP requests.
pub trait HttpMethods {
    fn get(url: impl AsRef<str>, token: Option<AccessToken>) -> Result<Response> {
        Http::send(Client::new().get(Url::parse(url.as_ref())?), token)
    }
    fn post(url: impl AsRef<str>, token: Option<AccessToken>) -> Result<Response> {
        Http::send(Client::new().post(Url::parse(url.as_ref())?), token)
    }
    fn put(url: impl AsRef<str>, token: Option<AccessToken>) -> Result<Response> {
        Http::send(Client::new().put(Url::parse(url.as_ref())?), token)
    }
    fn delete(url: impl AsRef<str>, token: Option<AccessToken>) -> Result<Response> {
        Http::send(Client::new().delete(Url::parse(url.as_ref())?), token)
    }
}


/// Make HTTP requests to server endpoints.
pub struct Http;

impl HttpMethods for Http {}

impl Http {
    /// Send an HTTP request with an optional bearer token.
    pub fn send(mut builder: RequestBuilder, token: Option<AccessToken>) -> Result<Response> {
        if let Some(token) = token {
            debug!("request with token scopes: {}", token.scope);
            builder = builder.bearer_auth(token.access_token.clone());

            match token.namespace() {
                Ok(name) => builder = builder.header("x-ats-namespace", name),
                Err(err) => error!("reading token namespace: {}", err),
            }
        }

        let req = builder.build()?;
        if req.headers().len() > 0 {
            debug!("request headers:\n{:#?}", req.headers());
        }
        if let Some(body) = req.body() {
            debug!("request body:\n{:?}\n", body);
        }

        Client::new().execute(req).map_err(Error::Http)
    }

    /// Print the HTTP response to stdout.
    pub fn print_response(mut resp: Response) -> Result<()> {
        let mut body = Vec::new();
        debug!("response headers:\n{:#?}", resp.headers());
        debug!("response length: {}\n", resp.read_to_end(&mut body)?);

        let out = if let Ok(json) = serde_json::from_slice::<Value>(&body) {
            serde_json::to_vec_pretty(&json)?
        } else {
            body
        };

        let _ = io::copy(&mut out.as_slice(), &mut io::stdout())?;
        Ok(())
    }
}
