pub mod history;
pub mod models;
pub mod samples;
pub mod tts;
pub mod user;
pub mod voice;

use crate::error::Error;
use crate::prelude::*;
use crate::support::*;
use http_body_util::BodyExt;
use hyper::{
    body::{Body, Bytes},
    client::conn::http1::handshake,
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, Request, Uri,
};
use std::env;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::TcpStream,
};

const BASE_URL: &str = "https://api.elevenlabs.io";
const V1_PATH: &str = "/v1";
const XI_API_KEY_HEADER: &str = "xi-api-key";

static HOST: &'static str = "host";
static AUTHORITY: &'static str = "api.elevenlabs.io";

#[derive(Debug)]
pub struct Client {
    pub url: Uri,
    pub method: Method,
    pub headers: HeaderMap,
}

impl Client {
    fn build_request<T: Body>(&self, body: T) -> Result<Request<T>> {
        let mut req_builder = Request::builder()
            .uri(self.url.clone())
            .method(self.method.clone());

        for (name, value) in self.headers.clone().iter() {
            req_builder = req_builder.header(name, value);
        }

        let req = req_builder.body(body)?;

        Ok(req)
    }
    pub fn format_address(&self) -> String {
        // unwrap() is warranted because the client is always built with the BASE_URL
        let host = self.url.host().unwrap();
        let addr = format!("{}:{}", host, "443");
        addr
    }

    pub async fn send_request<T: Body + Send>(&self, body: T) -> Result<Bytes>
    where
        T::Data: Send,
        T::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let req = self.build_request(body)?;
        let stream = TcpStream::connect(self.format_address()).await?;
        let tls_stream = async_native_tls::connect(self.url.host().unwrap(), stream).await?;
        let io = TokioIo::new(tls_stream);
        let (mut sender, conn) = handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("connection error: {}", e);
            }
        });
        let mut res = sender.send_request(req).await?;
        if res.status() != 200 {
            let w = Vec::new();
            let mut writer = BufWriter::new(w);
            while let Some(next) = res.frame().await {
                let frame = next?;
                if let Some(chunk) = frame.data_ref() {
                    writer.write_all(&chunk).await?;
                }
                writer.flush().await?;
            }
            let err_value = serde_json::from_slice::<serde_json::Value>(&writer.into_inner())?;

            return Err(Box::new(Error::ClientSendRequestError(err_value)));
        }
        let w = Vec::new();
        let mut writer = BufWriter::new(w);
        while let Some(next) = res.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                writer.write_all(&chunk).await?;
            }
            writer.flush().await?;
        }
        Ok(Bytes::from(writer.into_inner()))
    }
}

#[derive(Debug)]
pub struct ClientBuilder {
    url: Option<Uri>,
    method: Option<Method>,
    headers: Option<HeaderMap>,
}

impl ClientBuilder {
    pub fn new() -> Result<Self> {
        let mut cb = ClientBuilder::default();
        let apikey = env::var("ELEVEN_API_KEY")?;
        cb = cb.header(XI_API_KEY_HEADER, &apikey)?;
        Ok(cb)
    }

    pub fn path(mut self, path: impl Into<String>) -> Result<Self> {
        let url = format!("{}{}{}", BASE_URL, V1_PATH, path.into()).parse::<Uri>()?;
        self.url = Some(url);
        Ok(self)
    }

    pub fn method(mut self, method: impl Into<String>) -> Result<Self> {
        let method = method.into().parse::<Method>()?;
        self.method = Some(method);
        Ok(self)
    }

    pub fn header(mut self, name: &str, value: &str) -> Result<Self> {
        let header_name = name.parse::<HeaderName>()?;
        let header_value = value.parse::<HeaderValue>()?;
        // unwrap() is warranted because self.headers has default headers set with one initial entry
        self.headers
            .as_mut()
            .unwrap()
            .append(header_name, header_value);
        Ok(self)
    }

    pub fn build(self) -> Result<Client> {
        let Some(url) = self.url else {
            return Err(Box::new(Error::ClientBuildError(
                "url is not set".to_string(),
            )));
        };

        let method = self.method.unwrap_or(Method::GET);

        Ok(Client {
            url,
            method,
            // unwrap() is warranted because self.headers has default headers set with one intial entry
            headers: self.headers.unwrap(),
        })
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
