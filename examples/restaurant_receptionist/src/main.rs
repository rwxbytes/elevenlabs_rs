use crate::prelude::AppError;

mod agent;
mod prelude;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let agent = agent::create_agent().await?;
    println!("Agent created: {:?}", agent);
    Ok(())
}
