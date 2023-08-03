mod database; 

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {

    // let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    database::schema::query::transact().await?;
    
   
    Ok(())
}
