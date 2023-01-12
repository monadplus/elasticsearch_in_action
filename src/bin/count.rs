use elasticsearch_in_action::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = get_basic_auth_client().await?;
    let count = count_tweets(&client).await?;
    println!("Number of tweets indexed: {}", count);
    Ok(())
}
