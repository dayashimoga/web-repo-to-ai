//! Integration tests for persistent rate limit counters.

use web_repo_to_ai_storage::rate_limit::{InMemoryRateLimitStore, RateLimitStore};

#[tokio::test]
async fn test_in_memory_rate_limit_counter_lifecycle() {
    let store = InMemoryRateLimitStore::new();

    // 1. Initial count is 0
    let count0 = store.get_count("client_test", 100).await.unwrap();
    assert_eq!(count0, 0);

    // 2. Increment by 1
    let c1 = store
        .increment_and_count("client_test", 100, 1, 60)
        .await
        .unwrap();
    assert_eq!(c1, 1);

    // 3. Increment by 4
    let c2 = store
        .increment_and_count("client_test", 100, 4, 60)
        .await
        .unwrap();
    assert_eq!(c2, 5);

    // 4. Read back
    let read_count = store.get_count("client_test", 100).await.unwrap();
    assert_eq!(read_count, 5);

    // 5. Separate window bucket is isolated
    let other_bucket = store.get_count("client_test", 101).await.unwrap();
    assert_eq!(other_bucket, 0);

    // 6. Separate client key is isolated
    let other_client = store.get_count("other_client", 100).await.unwrap();
    assert_eq!(other_client, 0);

    // 7. Prune does not remove active keys
    let pruned = store.prune_expired().await.unwrap();
    assert_eq!(pruned, 0);
}
