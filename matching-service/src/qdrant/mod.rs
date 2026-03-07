use crate::config::Config;
use crate::error::MatcherError;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{SearchPoints, WithPayloadSelector, with_payload_selector};

/// Create a Qdrant client from configuration
pub async fn create_client(config: &Config) -> Result<Qdrant, MatcherError> {
    Qdrant::from_url(&config.qdrant_grpc_url)
        .build()
        .map_err(|e| MatcherError::QdrantSearchError(e.to_string()))
}

/// Search for the nearest vectors in the collection
pub async fn search_nearest(
    client: &Qdrant,
    collection_name: &str,
    query_vector: Vec<f32>,
    top_k: u64,
) -> Result<Vec<qdrant_client::qdrant::ScoredPoint>, MatcherError> {
    let request = SearchPoints {
        collection_name: collection_name.to_string(),
        vector: query_vector,
        limit: top_k,
        with_payload: Some(WithPayloadSelector {
            selector_options: Some(with_payload_selector::SelectorOptions::Enable(true)),
        }),
        ..Default::default()
    };

    let response = client
        .search_points(request)
        .await
        .map_err(|e| MatcherError::QdrantSearchError(e.to_string()))?;

    Ok(response.result)
}
