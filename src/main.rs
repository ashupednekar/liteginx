use cmd::run;

mod cmd;
mod pkg;
pub mod prelude;

#[tokio::main]
async fn main() -> prelude::Result<()> {
    tracing_subscriber::fmt::init();
    run().await?;
    Ok(())
}
