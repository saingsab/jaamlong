pub mod http;
pub mod database;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {

    //start server
    http::start().await?;

    Ok(())
}