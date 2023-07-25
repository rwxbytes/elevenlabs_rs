pub mod models;

use crate::error::Error;
use crate::prelude::*;
use http_body_util::{BodyExt, Empty};
use hyper::{
    body::Bytes,
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, Request, Uri,
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::{
    fs::{read_to_string, File},
    io::{AsyncWriteExt as _, BufWriter},
    net::TcpStream,
};

const ACCEPT_HEADER: &str = "ACCEPT";
//const AUTHORITY: HeaderValue = "api.elevenlabs.io";
const BASE_URL: &str = "https://api.elevenlabs.io";
//const HOST_HEADER: &str = "HOST";
const V1_PATH: &str = "/v1";
const XI_API_KEY_HEADER: &str = "xi-api-key";

static HOST: &'static str = "host";
static AUTHORITY: &'static str = "api.elevenlabs.io";

#[derive(Debug)]
pub struct Client {
    //apikey: String,
    pub url: Uri,
    pub method: Method,
    pub headers: HeaderMap,
}

impl Client {
    pub fn connect() {}
}

pub struct ClientBuilder {
    //apikey: Option<String>,
    url: Option<Uri>,
    method: Option<Method>,
    headers: Option<HeaderMap>,
}

impl ClientBuilder {
    pub fn new() -> Result<Self> {
        let mut cb = ClientBuilder::default();
        let apikey = env::var("ELEVEN_API_KEY")?;
        let _ = cb.header("xi-api-key", &apikey);
        Ok(cb)
    }

    pub fn path(&mut self, path: impl Into<String>) -> Result<&mut Self> {
        let url = format!("{}{}{}", BASE_URL, V1_PATH, path.into()).parse::<Uri>()?;
        self.url = Some(url);
        Ok(self)
    }

    pub fn method(&mut self, method: impl Into<String>) -> Result<&mut Self> {
        // test caps and lowercase
        let method = method.into().parse::<Method>()?;
        self.method = Some(method);
        Ok(self)
    }

    pub fn header(&mut self, name: &str, value: &str) -> Result<&mut Self> {
        let header_name = name.parse::<HeaderName>()?;
        let header_value = value.parse::<HeaderValue>()?;
        // unwrap() is warranted because self.headers has default headers set with one initial entry
        self.headers
            .as_mut()
            .unwrap()
            .append(header_name, header_value);
        Ok(self)
    }

    pub fn build(&self) -> Result<Client> {
        let Some(url) = self.url.as_ref() else {
            return Err(Box::new(Error::ClientBuildError(
                "url is not set".to_string(),
            )));
        };

        let method = self.method.as_ref().unwrap_or(&Method::GET);

        // unwrap() is warranted because self.headers has default headers set with one intial entry
        let headers = self.headers.as_ref().unwrap();

        let c = Client {
            url: url.clone(),
            method: method.clone(),
            headers: headers.clone(),
        };
        Ok(c)
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        let host_header = HeaderName::from_static(HOST);
        let authority_header = HeaderValue::from_static(AUTHORITY);
        let mut headers = HeaderMap::new();
        headers.append(host_header, authority_header);
        Self {
            url: None,
            method: None,
            headers: Some(headers),
        }
    }
}
