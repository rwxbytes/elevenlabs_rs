use elevenlabs_rs::api::voice::*;
use elevenlabs_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let v = get_voice("udhjsdh", false).await?;
    println!("Voice: {:#?}", v);
    Ok(())
}
