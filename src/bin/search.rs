use clap::{Parser, Subcommand};
use elasticsearch_in_action::*;

#[derive(Parser)]
#[command(name = "MyApp")]
#[command(about = "Full-text search on the tweets index", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Some examples using the search API
    Examples,
    /// Search a word appearing on the tweet's content
    Search {
        #[arg(long)]
        word: String,
        #[arg(long, default_value_t = 10)]
        size: u64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = get_basic_auth_client().await?;
    match args.command {
        Some(Command::Search { word, size }) => {
            let tweets = search_tweet_by_message(&client, &word[..], size).await?;
            println!(
                r#"Tweets containing the word "{}": {}"#,
                word,
                serde_json::to_string_pretty(&tweets)?
            );
            Ok(())
        }
        Some(Command::Examples) | None => {
            let tweets = search_filtered(&client).await?;
            println!("Count: {}", tweets.len());
            println!(
                "Ids: {:?}",
                tweets.into_iter().map(|tweet| tweet.id).collect::<Vec<_>>()
            );

            Ok(())
        }
    }
}
