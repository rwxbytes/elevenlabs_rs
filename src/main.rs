use elevenlabs::prelude::*;
use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, Request};
use std::env;
use tokio::{
    fs::{read_to_string, File},
    io::{AsyncWriteExt as _, BufWriter},
    net::TcpStream,
};

use elevenlabs::api::models;
use elevenlabs::support::*;

#[tokio::main]
async fn main() -> Result<()> {
    const BASE_URL_V1: &str = "https://api.elevenlabs.io/v1/models";
    let apikey = env::var("ELEVEN_API_KEY").expect("ELEVEN_API_KEY is set");
    // let models = models::get_models().await?;

    let url = BASE_URL_V1.parse::<hyper::Uri>()?;

    let host = url.host().expect("host() is getting host from url");
    let port = url.port_u16().unwrap_or(443);

    let addr = format!("{}:{}", host, port);

    let stream = TcpStream::connect(&addr).await?;
    let tls_stream = async_native_tls::connect(host, stream).await?;

    let io = TokioIo::new(tls_stream);

    let (mut sender, mut conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    let authourity = url
        .authority()
        .expect("authority() is getting authority from url")
        .clone();
    dbg!(authourity.as_str());

    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authourity.as_str())
        .header(hyper::header::ACCEPT, "application/json")
        .header("xi-api-key", apikey)
        .body(Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;

    println!("request: {:?}", res.status());

    let f = File::create("foo.txt").await?;
    let mut writer = BufWriter::new(f);

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            writer.write_all(&chunk).await?;
        }
        writer.flush().await?;
    }
    let data = read_to_string("foo.txt").await?;
    let models: Vec<models::Model> = models::parse_models_resp(&data)?;
    println!("{:#?}", models);

    Ok(())
}
