pub mod handlers;
pub mod routers;
pub mod models;
pub mod utils;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {

    //start server
    handlers::start().await?;

    Ok(())
}