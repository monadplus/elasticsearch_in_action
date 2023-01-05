# Elasticsearch in Action

## Example

Start a local elasticsearch server (single node or cluster):

```bash
# Option 1
docker-compose -f docker/single_node_with_kibana.yaml up
# Option 2 (recommended)
docker-compose -f docker/cluster_with_kibana.yaml up
```

Before continuing, check your cluster status:

```bash
curl -X GET "localhost:9200/_cluster/stats?human&pretty&pretty"`

curl -X GET "localhost:9200/_cluster/health?pretty"
```

Optionally, open [Kibana](https://www.elastic.co/kibana/) on your browser: `firefox http://0.0.0.0:5601`.

Index a [collection of tweets](./tweets.json):

```bash
cargo run --bin index
```

Search by word:

```bash
cargo run --bin search -- --word=ipsum
```
