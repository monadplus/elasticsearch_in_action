use elasticsearch::{
    http::{transport::Transport, StatusCode},
    indices::{IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts},
    params::VersionType,
    BulkOperation, BulkParts, CountParts, Elasticsearch, IndexParts, SearchParts,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tokio::time::Instant;

mod tweet;
use tweet::*;

pub const TWEETS_INDEX: &str = "tweets";

pub async fn get_client() -> anyhow::Result<Elasticsearch> {
    let transport = Transport::single_node("http://localhost:9200")?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

/// ```bash
/// curl -H "Content-Type: application/json" -XGET 'http://localhost:9200/_count?pretty' -d '{"query": {"match_all": {}}}'
/// ```
pub async fn count_tweets(client: &Elasticsearch) -> anyhow::Result<u64> {
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
pub enum IndexResult {
    #[strum(serialize = "created")]
    Created,
    #[strum(serialize = "updated")]
    Updated,
}

/// Creates index "tweets". If the index exists, it deletes the previous data.
pub async fn create_index_if_not_exists(
    client: &Elasticsearch,
    delete: bool,
) -> anyhow::Result<()> {
    let exists = client
        .indices()
        .exists(IndicesExistsParts::Index(&[TWEETS_INDEX]))
        .send()
        .await?;

    if exists.status_code().is_success() && delete {
        let delete = client
            .indices()
            .delete(IndicesDeleteParts::Index(&[TWEETS_INDEX]))
            .send()
            .await?;

        if !delete.status_code().is_success() {
            println!("Problem deleting index: {}", delete.text().await?);
        }
    }

    if exists.status_code() == StatusCode::NOT_FOUND || delete {
        let response = client
            .indices()
            .create(IndicesCreateParts::Index(TWEETS_INDEX))
            .body(json!({
                "mappings": {
                    "properties": {
                        "user": { "type": "keyword" },
                        "date": { "type": "date" },
                        "message": { "type": "text" },
                    }
                }
            }))
            .send()
            .await?;

        if !response.status_code().is_success() {
            println!("Error while creating index");
        }
    }

    Ok(())
}

/// Inserts or updates the given tweet
///
/// ```rust, ignore
/// let tweet = Tweet {
///     id: 1,
///     user: User("Arnau".to_string()),
///     date: Utc::now(),
///     message: Message("This is an example".to_string()),
/// };
/// let tweet_id = tweet.id;
/// let index_result = index_tweet(&client, tweet).await?;
/// println!("Tweet {} {}", tweet_id, index_result);
/// ```
pub async fn index_tweet(client: &Elasticsearch, tweet: Tweet) -> anyhow::Result<IndexResult> {
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
pub fn deserialize_hits<T>(json: &Value) -> Vec<T>
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
pub async fn search_tweet_by_message(
    client: &Elasticsearch,
    message: &str,
    size: u64,
) -> anyhow::Result<Vec<Tweet>> {
    let response = client
        .search(SearchParts::Index(&[TWEETS_INDEX]))
        .from(0)
        .size(size as i64)
        .body(json!({
            "query": {
                "match": {
                    "message": {
                        "query": message,
                        "operator": "or"
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

pub async fn load_tweets<P: AsRef<Path>>(client: &Elasticsearch, path: P) -> anyhow::Result<()> {
    let tweets: Vec<Tweet> = read_tweets(path).await?;

    let start_time = Instant::now();

    for tweets in tweets.chunks(1000) {
        let body: Vec<BulkOperation<_>> = tweets
            .iter()
            .map(|tweet| {
                let id = tweet.id.to_string();
                BulkOperation::index(tweet)
                    .id(&id)
                    // .version(1)
                    // .version_type(VersionType::External)
                    .into()
            })
            .collect();

        let response = client
            .bulk(BulkParts::Index(TWEETS_INDEX))
            .body(body)
            .send()
            .await?;

        let json: Value = response.json().await?;
        if json["errors"].as_bool().unwrap() {
            let failed: Vec<&Value> = json["items"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| !v["error"].is_null())
                .collect();

            // TODO: retry failures
            println!("Errors whilst indexing. Failures: {}", failed.len());
        }
    }

    println!(
        "Indexed {} tweets in {} ms",
        tweets.len(),
        start_time.elapsed().as_millis()
    );

    Ok(())
}
