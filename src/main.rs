pub mod cmd;
pub mod conf;
pub mod pkg;
pub mod prelude;

use prelude::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    cmd::run().await?;
    Ok(())
}
