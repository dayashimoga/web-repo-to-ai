//! Integration tests for batch jobs and item tracking persistence.

use chrono::Utc;
use uuid::Uuid;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_storage::batch::{
    BatchItemRecord, BatchItemStatusKind, BatchJobStatus, BatchRecord, BatchRepository,
    InMemoryBatchRepository,
};

#[tokio::test]
async fn test_batch_repository_full_lifecycle() {
    let repo = InMemoryBatchRepository::new();
    let batch_id = BatchId::new();
    let owner_session = Uuid::new_v4();
    let now = Utc::now();

    let job1 = JobId::new();
    let job2 = JobId::new();
    let job3 = JobId::new();

    let batch = BatchRecord {
        id: batch_id,
        name: Some("Integration Batch".to_string()),
        status: BatchJobStatus::Queued,
        total_items: 3,
        queued_items: 3,
        processing_items: 0,
        completed_items: 0,
        failed_items: 0,
        max_parallelism: 5,
        owner_session_id: Some(owner_session),
        created_at: now,
        updated_at: now,
        completed_at: None,
    };

    let items = vec![
        BatchItemRecord {
            batch_id,
            item_index: 0,
            job_id: job1,
            url: "https://example.com/item1".to_string(),
            custom_id: Some("custom-1".to_string()),
            status: BatchItemStatusKind::Queued,
            error_code: None,
            error_message: None,
            duration_ms: None,
            artifact_count: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        },
        BatchItemRecord {
            batch_id,
            item_index: 1,
            job_id: job2,
            url: "https://example.com/item2".to_string(),
            custom_id: Some("custom-2".to_string()),
            status: BatchItemStatusKind::Queued,
            error_code: None,
            error_message: None,
            duration_ms: None,
            artifact_count: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        },
        BatchItemRecord {
            batch_id,
            item_index: 2,
            job_id: job3,
            url: "https://example.com/item3".to_string(),
            custom_id: None,
            status: BatchItemStatusKind::Queued,
            error_code: None,
            error_message: None,
            duration_ms: None,
            artifact_count: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        },
    ];

    // 1. Create batch
    let created = repo.create_batch(batch, items).await.expect("create batch");
    assert_eq!(created.id, batch_id);
    assert_eq!(created.total_items, 3);

    // 2. Lookup individual item by child job_id
    let item_by_job = repo
        .get_batch_item_by_job_id(job2)
        .await
        .expect("get item by job_id")
        .expect("item must exist");
    assert_eq!(item_by_job.item_index, 1);
    assert_eq!(item_by_job.custom_id, Some("custom-2".to_string()));

    // 3. Transition item 0 to Processing
    repo.update_item_status(
        batch_id,
        0,
        BatchItemStatusKind::Processing,
        None,
        None,
        None,
    )
    .await
    .expect("update item 0");

    let b1 = repo.get_batch(batch_id).await.unwrap().unwrap();
    assert_eq!(b1.status, BatchJobStatus::Processing);
    assert_eq!(b1.processing_items, 1);
    assert_eq!(b1.queued_items, 2);

    // 4. Complete item 0
    repo.update_item_status(
        batch_id,
        0,
        BatchItemStatusKind::Completed,
        None,
        None,
        Some(250),
    )
    .await
    .expect("complete item 0");

    // 5. Fail item 1
    repo.update_item_status(
        batch_id,
        1,
        BatchItemStatusKind::Failed,
        Some("SSRF_BLOCKED".to_string()),
        Some("Target resolved to private IP".to_string()),
        Some(15),
    )
    .await
    .expect("fail item 1");

    let b2 = repo.get_batch(batch_id).await.unwrap().unwrap();
    assert_eq!(b2.completed_items, 1);
    assert_eq!(b2.failed_items, 1);
    assert_eq!(b2.queued_items, 1);
    // Not terminal yet because item 2 is still queued
    assert_eq!(b2.status, BatchJobStatus::Processing);

    // 6. Complete item 2
    repo.update_item_status(
        batch_id,
        2,
        BatchItemStatusKind::Completed,
        None,
        None,
        Some(340),
    )
    .await
    .expect("complete item 2");

    // All 3 items are now terminal (2 completed, 1 failed) -> PartialFailure
    let b3 = repo.get_batch(batch_id).await.unwrap().unwrap();
    assert_eq!(b3.status, BatchJobStatus::PartialFailure);
    assert_eq!(b3.completed_items, 2);
    assert_eq!(b3.failed_items, 1);
    assert_eq!(b3.queued_items, 0);
    assert_eq!(b3.processing_items, 0);
    assert!(b3.completed_at.is_some());

    // 7. Verify deterministic item ordering
    let all_items = repo.get_batch_items(batch_id).await.unwrap();
    assert_eq!(all_items.len(), 3);
    assert_eq!(all_items[0].item_index, 0);
    assert_eq!(all_items[1].item_index, 1);
    assert_eq!(all_items[2].item_index, 2);
    assert_eq!(all_items[1].error_code, Some("SSRF_BLOCKED".to_string()));

    // 8. List batches with session filtering
    let listed = repo
        .list_batches(Some(owner_session), 10, 0)
        .await
        .expect("list batches");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, batch_id);

    // Other session should return empty
    let empty = repo
        .list_batches(Some(Uuid::new_v4()), 10, 0)
        .await
        .expect("list empty");
    assert_eq!(empty.len(), 0);
}

#[tokio::test]
async fn test_batch_cancellation_marks_unfinished_items() {
    let repo = InMemoryBatchRepository::new();
    let batch_id = BatchId::new();
    let now = Utc::now();

    let batch = BatchRecord {
        id: batch_id,
        name: None,
        status: BatchJobStatus::Processing,
        total_items: 2,
        queued_items: 1,
        processing_items: 1,
        completed_items: 0,
        failed_items: 0,
        max_parallelism: 5,
        owner_session_id: None,
        created_at: now,
        updated_at: now,
        completed_at: None,
    };

    let items = vec![
        BatchItemRecord {
            batch_id,
            item_index: 0,
            job_id: JobId::new(),
            url: "https://example.com/active".to_string(),
            custom_id: None,
            status: BatchItemStatusKind::Processing,
            error_code: None,
            error_message: None,
            duration_ms: None,
            artifact_count: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        },
        BatchItemRecord {
            batch_id,
            item_index: 1,
            job_id: JobId::new(),
            url: "https://example.com/waiting".to_string(),
            custom_id: None,
            status: BatchItemStatusKind::Queued,
            error_code: None,
            error_message: None,
            duration_ms: None,
            artifact_count: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        },
    ];

    repo.create_batch(batch, items).await.unwrap();

    let cancelled = repo.cancel_batch(batch_id).await.unwrap().unwrap();
    assert_eq!(cancelled.status, BatchJobStatus::Cancelled);
    assert_eq!(cancelled.queued_items, 0);
    assert_eq!(cancelled.processing_items, 0);

    let items = repo.get_batch_items(batch_id).await.unwrap();
    assert_eq!(items[0].status, BatchItemStatusKind::Cancelled);
    assert_eq!(items[1].status, BatchItemStatusKind::Cancelled);
}
