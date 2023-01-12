use std::path::Path;

use elasticsearch_in_action::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = get_basic_auth_client().await?;
    create_index_if_not_exists(&client, true).await?;
    let tweets_filepath: &Path = Path::new("./tweets.json");
    load_tweets(&client, tweets_filepath).await?;
    Ok(())
}
