use chrono::{DateTime, Utc};
use elasticsearch::{
    http::transport::Transport, CountParts, Elasticsearch, IndexParts, SearchParts,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tweet {
    id: u64,
    user: User,
    post_date: DateTime<Utc>,
    message: Message,
}

const TWEETS_INDEX: &str = "tweets";

async fn get_client() -> anyhow::Result<Elasticsearch> {
    let transport = Transport::single_node("http://localhost:9200")?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

/// ```bash
/// curl -H "Content-Type: application/json" -XGET 'http://localhost:9200/_count?pretty' -d '{"query": {"match_all": {}}}'
/// ```
async fn count_tweets(client: &Elasticsearch) -> anyhow::Result<u64> {
    let response = client
        .count(CountParts::Index(&[TWEETS_INDEX]))
        .body(json!({
            "query": {
                "match_all": {}
            }
        }))
        .pretty(true)
        .send()
        .await?
        .error_for_status_code()?;
    let response_body: Value = response.json().await?;
    let count: u64 = response_body["count"].as_u64().unwrap_or_default();
    Ok(count)
}

#[derive(Debug, strum::Display, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum IndexResult {
    #[strum(serialize = "created")]
    Created,
    #[strum(serialize = "updated")]
    Updated,
}

/// Inserts or updates the given tweet
async fn index_tweet(client: &Elasticsearch, tweet: Tweet) -> anyhow::Result<IndexResult> {
    let response = client
        .index(IndexParts::IndexId(TWEETS_INDEX, &tweet.id.to_string()))
        .body(&tweet)
        .send()
        .await?
        .error_for_status_code()?;
    let response_body: Value = response.json().await?;
    let index_result = serde_json::from_value(response_body["result"].clone())?;
    Ok(index_result)
}

/// Use for Search API.
fn deserialize_hits<T>(json: &Value) -> Vec<T>
where
    T: DeserializeOwned,
{
    json["hits"]["hits"]
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|h| serde_json::from_value::<T>(h["_source"].clone()).ok())
        .collect()
}

/// Search all tweets containing the given word
///
/// ```bash
/// curl -H "Content-Type: application/json" -XGET 'http://localhost:9200/tweets/_search?pretty' -d '{"query": {"match": {"message": "example"}}}'
/// ```
async fn search_tweet_by_message(
    client: &Elasticsearch,
    message: &str,
) -> anyhow::Result<Vec<Tweet>> {
    let response = client
        .search(SearchParts::Index(&[TWEETS_INDEX]))
        .from(0)
        .size(100)
        .body(json!({
            "query": {
                "match": {
                    "message": {
                        "query": message,
                        "operator": "and"
                    }
                }
            }
        }))
        .send()
        .await?
        .error_for_status_code()?;
    let response_body: Value = response.json().await?;
    let tweets = deserialize_hits::<Tweet>(&response_body);
    Ok(tweets)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = get_client().await?;

    let tweet = Tweet {
        id: 1,
        user: User("Arnau".to_string()),
        post_date: Utc::now(),
        message: Message("This is an example".to_string()),
    };
    let tweet_id = tweet.id;
    let index_result = index_tweet(&client, tweet).await?;
    println!("Tweet {} was {}", tweet_id, index_result);

    let count = count_tweets(&client).await?;
    println!("Number of tweets indexed: {}", count);

    let tweets = search_tweet_by_message(&client, "example").await?;
    println!(
        r#"Tweets containing the word "example": {}"#,
        serde_json::to_string_pretty(&tweets)?
    );

    Ok(())
}
