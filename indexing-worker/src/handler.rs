use crate::config::Config;
use crate::error::IndexerError;
use qdrant_client::Payload;
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
use shared_types::DocumentVectorizedEvent;

/// Process a single vectorized document event
///
/// 1. Converts the document UUID to a Qdrant point ID
/// 2. Constructs a Qdrant point with the embedding vector
/// 3. Upserts the point into the configured collection
pub async fn process_vector_event(
    event: DocumentVectorizedEvent,
    qdrant_client: &qdrant_client::Qdrant,
    config: &Config,
) -> Result<(), IndexerError> {
    let doc_id = event.id;

    tracing::info!(%doc_id, "Processing vector for indexing");

    let point_id = doc_id.as_u128() as u64;
    let point = PointStruct::new(
        point_id,
        event.vector,
        Payload::new(), // We could add metadata here later
    );

    qdrant_client
        .upsert_points(
            UpsertPointsBuilder::new(&config.collection_name, vec![point]).wait(true),
        )
        .await
        .map_err(|e| IndexerError::QdrantUpsertError(e.to_string()))?;

    tracing::info!(%doc_id, "Successfully indexed in Qdrant");

    Ok(())
}
