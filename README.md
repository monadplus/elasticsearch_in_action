# Elasticsearch in Action

Start your single node elasticsearch server:

```bash
docker-compose -f docker/simple.yaml up
  ```

Index a [collection of tweets](./tweets.json):

```bash
cargo run --bin index
  ```

Search by word:

```bash
cargo run --bin search -- --word=ipsum
  ```
