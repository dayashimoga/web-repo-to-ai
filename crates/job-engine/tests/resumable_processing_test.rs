//! Integration tests for Resumable Processing & Durable Checkpointing.

use std::sync::Arc;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_job_engine::config::BatchExecutionConfig;
use web_repo_to_ai_job_engine::error::JobEngineError;
use web_repo_to_ai_job_engine::models::{BatchItemStatusOutcome, BatchItemTask};
use web_repo_to_ai_job_engine::resumable::{
    CheckpointItemResult, CheckpointStore, InMemoryCheckpointStore, JobCheckpoint, ResumableConfig,
    ResumableExecutionPlan, ResumableParallelExecutor,
};

#[test]
fn test_checkpoint_checksum_integrity_and_tampering_detection() {
    let mut cp = JobCheckpoint::new("job-123", 5);
    assert_eq!(cp.sequence_number, 0);
    assert!(cp.verify_checksum().is_ok());

    // Record an item
    cp.record_item(CheckpointItemResult {
        item_index: 0,
        job_id: JobId::new(),
        url: "https://example.com/0".to_string(),
        custom_id: None,
        status: BatchItemStatusOutcome::Succeeded,
        value_json: Some("\"result-0\"".to_string()),
        error: None,
        duration_ms: 12,
    });

    assert_eq!(cp.sequence_number, 1);
    assert!(cp.completed_indices.contains(&0));
    assert!(cp.verify_checksum().is_ok());

    // Tamper with data without calling update_checksum()
    cp.completed_indices.insert(99);
    assert!(matches!(
        cp.verify_checksum(),
        Err(JobEngineError::CheckpointCorrupted { .. })
    ));
}

#[tokio::test]
async fn test_in_memory_checkpoint_store_crud() {
    let store = InMemoryCheckpointStore::new();
    assert!(store.is_empty().await);

    let cp = JobCheckpoint::new("batch-456", 10);
    store.save_checkpoint(&cp).await.unwrap();

    assert_eq!(store.len().await, 1);
    let loaded = store.load_checkpoint("batch-456").await.unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().job_id, "batch-456");

    store.delete_checkpoint("batch-456").await.unwrap();
    assert!(store.is_empty().await);
    assert!(store.load_checkpoint("batch-456").await.unwrap().is_none());
}

#[test]
fn test_resumable_execution_plan_partitioning() {
    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..5)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/{i}"),
                None,
                (),
            )
        })
        .collect();

    // 1. Fresh plan without checkpoint
    let fresh_plan = ResumableExecutionPlan::create(tasks.clone(), None);
    assert!(!fresh_plan.is_resumed);
    assert_eq!(fresh_plan.already_completed_indices.len(), 0);
    assert_eq!(fresh_plan.pending_tasks.len(), 5);

    // 2. Plan with partial checkpoint (items 0 and 2 completed)
    let mut cp = JobCheckpoint::new(batch_id.to_string(), 5);
    cp.record_item(CheckpointItemResult {
        item_index: 0,
        job_id: tasks[0].job_id,
        url: tasks[0].url.clone(),
        custom_id: None,
        status: BatchItemStatusOutcome::Succeeded,
        value_json: Some("\"done\"".to_string()),
        error: None,
        duration_ms: 10,
    });
    cp.record_item(CheckpointItemResult {
        item_index: 2,
        job_id: tasks[2].job_id,
        url: tasks[2].url.clone(),
        custom_id: None,
        status: BatchItemStatusOutcome::Succeeded,
        value_json: Some("\"done\"".to_string()),
        error: None,
        duration_ms: 15,
    });

    let resumed_plan = ResumableExecutionPlan::create(tasks, Some(&cp));
    assert!(resumed_plan.is_resumed);
    assert_eq!(resumed_plan.already_completed_indices, vec![0, 2]);
    assert_eq!(resumed_plan.pending_tasks.len(), 3);
    assert_eq!(resumed_plan.pending_tasks[0].item_index, 1);
    assert_eq!(resumed_plan.pending_tasks[1].item_index, 3);
    assert_eq!(resumed_plan.pending_tasks[2].item_index, 4);
}

#[tokio::test]
async fn test_resumable_executor_fresh_execution() {
    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    let batch_config = BatchExecutionConfig {
        max_parallelism: 2,
        ..Default::default()
    };
    let resumable_config = ResumableConfig::new(2).unwrap();

    let executor =
        ResumableParallelExecutor::new(batch_config, resumable_config, Arc::clone(&store)).unwrap();

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..4)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/fresh/{i}"),
                None,
                (),
            )
        })
        .collect();

    let (summary, stats) = executor
        .execute_resumable(
            batch_id,
            tasks,
            |t| async move { Ok(format!("val-{}", t.item_index)) },
            None,
            None,
        )
        .await
        .unwrap();

    assert!(!stats.is_resumed);
    assert_eq!(stats.items_reused, 0);
    assert_eq!(stats.items_executed, 4);
    assert!(stats.checkpoints_saved >= 2);

    assert_eq!(summary.total_items, 4);
    assert_eq!(summary.succeeded_count, 4);
    assert_eq!(summary.failed_count, 0);

    // Verify checkpoint is stored in completed state
    let cp = store.load_checkpoint(&batch_id.to_string()).await.unwrap();
    assert!(cp.is_some());
    let cp = cp.unwrap();
    assert!(cp.is_fully_finished());
    assert_eq!(cp.completed_indices.len(), 4);
}

#[tokio::test]
async fn test_resumable_executor_recovery_skips_completed_items() {
    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    let batch_config = BatchExecutionConfig {
        max_parallelism: 2,
        ..Default::default()
    };
    let resumable_config = ResumableConfig::new(2).unwrap();

    let executor =
        ResumableParallelExecutor::new(batch_config, resumable_config, Arc::clone(&store)).unwrap();

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..5)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/resume/{i}"),
                None,
                (),
            )
        })
        .collect();

    // Pre-populate checkpoint simulating crash after items 0, 1, 2 finished
    let mut initial_cp = JobCheckpoint::new(batch_id.to_string(), 5);
    for (i, task) in tasks.iter().enumerate().take(3) {
        initial_cp.record_item(CheckpointItemResult {
            item_index: i,
            job_id: task.job_id,
            url: task.url.clone(),
            custom_id: None,
            status: BatchItemStatusOutcome::Succeeded,
            value_json: Some(format!("\"pre-computed-{i}\"")),
            error: None,
            duration_ms: 10,
        });
    }
    store.save_checkpoint(&initial_cp).await.unwrap();

    // Now execute: should execute ONLY items 3 and 4!
    let executed_indices = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let exec_track = Arc::clone(&executed_indices);

    let (summary, stats) = executor
        .execute_resumable(
            batch_id,
            tasks,
            move |t| {
                let track = Arc::clone(&exec_track);
                async move {
                    track.lock().await.push(t.item_index);
                    Ok(format!("computed-{}", t.item_index))
                }
            },
            None,
            None,
        )
        .await
        .unwrap();

    assert!(stats.is_resumed);
    assert_eq!(stats.items_reused, 3);
    assert_eq!(stats.items_executed, 2);

    // Verify only items 3 and 4 were executed
    let executed = executed_indices.lock().await;
    assert_eq!(*executed, vec![3, 4]);

    // But final summary includes all 5 items in deterministic order
    assert_eq!(summary.total_items, 5);
    assert_eq!(summary.succeeded_count, 5);
    assert_eq!(summary.item_results.len(), 5);
    for i in 0..5 {
        assert_eq!(summary.item_results[i].item_index, i);
    }
    assert_eq!(
        summary.item_results[0].value,
        Some("pre-computed-0".to_string())
    );
    assert_eq!(
        summary.item_results[4].value,
        Some("computed-4".to_string())
    );
}

#[tokio::test]
async fn test_resumable_executor_delete_checkpoint_on_complete() {
    let store: Arc<dyn CheckpointStore> = Arc::new(InMemoryCheckpointStore::new());
    let batch_config = BatchExecutionConfig {
        max_parallelism: 2,
        ..Default::default()
    };
    let resumable_config = ResumableConfig::new(2).unwrap().with_cleanup(true);

    let executor =
        ResumableParallelExecutor::new(batch_config, resumable_config, Arc::clone(&store)).unwrap();

    let batch_id = BatchId::new();
    let tasks: Vec<BatchItemTask<()>> = (0..3)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/cleanup/{i}"),
                None,
                (),
            )
        })
        .collect();

    let (summary, _) = executor
        .execute_resumable(
            batch_id,
            tasks,
            |t| async move { Ok(format!("val-{}", t.item_index)) },
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(summary.succeeded_count, 3);

    // Verify checkpoint was deleted upon completion
    assert!(store
        .load_checkpoint(&batch_id.to_string())
        .await
        .unwrap()
        .is_none());
}
