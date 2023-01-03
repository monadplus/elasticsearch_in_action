use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tweet {
    pub id: u64,
    pub user: User,
    pub date: DateTime<Utc>,
    pub message: Message,
}

pub async fn read_tweets<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Tweet>> {
    let contents = fs::read(path).await?;
    let tweets = serde_json::from_slice(&contents)?;
    Ok(tweets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn read_tweets_test() {
        let path = Path::new("./tweets.json");
        let tweets = read_tweets(&path).await.unwrap();
        assert_eq!(tweets.len(), 100);
        assert_eq!(
            tweets[0],
            Tweet {
                id: 0,
                user: User("Juanita".to_string()),
                date: "2019-03-31T08:16:22 -02:00"
                    .parse::<DateTime<Utc>>()
                    .unwrap(),
                message: Message(
                    "excepteur ea aute aute in sunt consequat amet duis non".to_string()
                ),
            }
        )
    }
}
