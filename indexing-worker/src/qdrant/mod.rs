use crate::config::Config;
use crate::error::IndexerError;

/// Initialize and return a Qdrant gRPC client
///
/// Connects to the Qdrant instance and verifies the target collection exists.
pub async fn create_client(config: &Config) -> Result<qdrant_client::Qdrant, IndexerError> {
    let client = qdrant_client::Qdrant::from_url(&config.qdrant_grpc_url)
        .build()
        .map_err(|e| IndexerError::QdrantConnectionError(e.to_string()))?;

    // Verify the collection exists
    client
        .collection_info(&config.collection_name)
        .await
        .map_err(|e| IndexerError::QdrantConnectionError(e.to_string()))?;

    tracing::info!(
        collection = %config.collection_name,
        url = %config.qdrant_grpc_url,
        "Connected to Qdrant successfully"
    );

    Ok(client)
}
