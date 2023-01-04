use clap::Parser;
use elasticsearch_in_action::*;

#[derive(Parser)]
#[command(name = "MyApp")]
#[command(about = "Full-text search on the tweets index", long_about = None)]
struct Args {
    /// Search a word appearing on the tweet's content
    #[arg(long)]
    word: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = get_client().await?;
    let tweets = search_tweet_by_message(&client, &args.word, 10).await?;
    println!(
        r#"Tweets containing the word "{}": {}"#,
        args.word,
        serde_json::to_string_pretty(&tweets)?
    );

    Ok(())
}
