use crate::types::Query;

/// Standard query filtering logic used across all components
/// Filters for queries with a good range of relevant documents (3-20) for manual ranking
pub fn is_target_query(query: &Query) -> bool {
    let relevant_count = query.labels.iter().filter(|l| l.score > 0).count();
    relevant_count >= 3 && relevant_count <= 20
}

/// Get all target queries from a collection
pub fn get_target_queries(queries: &[Query]) -> Vec<&Query> {
    queries.iter().filter(|q| is_target_query(q)).collect()
}