//! Backpressure integration test suite.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_job_engine::backpressure::{
    BackpressureConfig, BackpressureController, BackpressureParallelExecutor, BackpressureState,
    FlowControlledQueue, LoadSheddingStrategy,
};
use web_repo_to_ai_job_engine::config::BatchExecutionConfig;
use web_repo_to_ai_job_engine::error::JobEngineError;
use web_repo_to_ai_job_engine::models::BatchItemTask;

#[test]
fn test_backpressure_config_validation() {
    let valid = BackpressureConfig::new(100, 80, 40);
    assert!(valid.is_ok());

    let invalid_zero = BackpressureConfig::new(0, 0, 0);
    assert!(invalid_zero.is_err());

    let invalid_high = BackpressureConfig::new(100, 120, 40);
    assert!(invalid_high.is_err());

    let invalid_low = BackpressureConfig::new(100, 80, 85);
    assert!(invalid_low.is_err());
}

#[tokio::test]
async fn test_backpressure_controller_state_transitions() {
    let config = BackpressureConfig::new(10, 8, 4).unwrap();
    let controller = BackpressureController::new(config).unwrap();

    assert_eq!(controller.state(), BackpressureState::Normal);
    assert_eq!(controller.current_depth(), 0);

    // Acquire 4 slots: still Normal (<= low watermark 4)
    let mut slots = Vec::new();
    for _ in 0..4 {
        slots.push(controller.acquire_slot(None).await.unwrap());
    }
    assert_eq!(controller.state(), BackpressureState::Normal);
    assert_eq!(controller.current_depth(), 4);

    // Acquire 3 more (total 7): between 4 and 8 -> Warning
    for _ in 0..3 {
        slots.push(controller.acquire_slot(None).await.unwrap());
    }
    assert_eq!(controller.state(), BackpressureState::Warning);
    assert_eq!(controller.current_depth(), 7);

    // Acquire 1 more (total 8): >= high watermark 8 -> Saturated
    slots.push(controller.acquire_slot(None).await.unwrap());
    assert_eq!(controller.state(), BackpressureState::Saturated);
    assert_eq!(controller.current_depth(), 8);

    // Drop 5 slots: depth drops from 8 to 3 (<= low watermark 4) -> returns to Normal
    slots.truncate(3);
    assert_eq!(controller.current_depth(), 3);
    assert_eq!(controller.state(), BackpressureState::Normal);

    let stats = controller.stats();
    assert_eq!(stats.total_admitted, 8);
    assert_eq!(stats.total_drained, 5);
}

#[tokio::test]
async fn test_backpressure_load_shedding_drop_newest() {
    let config = BackpressureConfig::new(5, 3, 1)
        .unwrap()
        .with_shed_strategy(LoadSheddingStrategy::DropNewest);

    let controller = BackpressureController::new(config).unwrap();

    let mut slots = Vec::new();
    for _ in 0..3 {
        slots.push(controller.acquire_slot(None).await.unwrap());
    }
    assert_eq!(controller.state(), BackpressureState::Saturated);

    // Next acquisition must be rejected with BackpressureQueueSaturated
    let result = controller.acquire_slot(None).await;
    match result {
        Err(JobEngineError::BackpressureQueueSaturated {
            current_depth,
            capacity,
        }) => {
            assert_eq!(current_depth, 3);
            assert_eq!(capacity, 5);
        }
        _ => panic!("Expected BackpressureQueueSaturated, got {:?}", result),
    }

    let stats = controller.stats();
    assert_eq!(stats.total_items_shed, 1);
}

#[tokio::test]
async fn test_backpressure_producer_suspension_and_resume() {
    let config = BackpressureConfig::new(4, 3, 1)
        .unwrap()
        .with_shed_strategy(LoadSheddingStrategy::BlockProducer)
        .with_wait_timeout(Duration::from_millis(500));

    let controller = Arc::new(BackpressureController::new(config).unwrap());

    // Saturate controller with 3 slots
    let mut slots = Vec::new();
    for _ in 0..3 {
        slots.push(controller.acquire_slot(None).await.unwrap());
    }
    assert_eq!(controller.state(), BackpressureState::Saturated);

    let ctrl_clone = Arc::clone(&controller);
    let producer_task = tokio::spawn(async move {
        // This acquire_slot will block until drained
        let slot = ctrl_clone.acquire_slot(None).await;
        slot.is_ok()
    });

    // Verify producer is blocked
    sleep(Duration::from_millis(50)).await;
    assert!(!producer_task.is_finished());

    // Drain 2 slots: drops depth to 1 (<= low watermark 1)
    slots.pop();
    slots.pop();

    // Producer must now unblock and succeed
    let outcome = tokio::time::timeout(Duration::from_millis(200), producer_task)
        .await
        .expect("Producer must unblock")
        .expect("Task join must succeed");

    assert!(outcome);
}

#[tokio::test]
async fn test_flow_controlled_queue_push_pop() {
    let config = BackpressureConfig::new(10, 8, 3).unwrap();
    let queue = FlowControlledQueue::new(config).unwrap();

    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);

    for i in 0..5 {
        queue.push(i * 10, None).await.unwrap();
    }

    assert_eq!(queue.len(), 5);
    assert!(!queue.is_empty());

    let val1 = queue.pop(None).await.unwrap();
    assert_eq!(val1, Some(0));

    let val2 = queue.try_pop();
    assert_eq!(val2, Some(10));

    assert_eq!(queue.len(), 3);
}

#[tokio::test]
async fn test_backpressure_parallel_executor_flow_controlled() {
    let backpressure_config = BackpressureConfig::new(6, 4, 2)
        .unwrap()
        .with_wait_timeout(Duration::from_secs(5));

    let batch_config = BatchExecutionConfig::default().with_item_timeout(Duration::from_secs(5));

    let tasks = (0..6)
        .map(|i| {
            BatchItemTask::new(
                i,
                JobId::new(),
                format!("https://example.com/item/{i}"),
                None,
                i,
            )
        })
        .collect::<Vec<_>>();

    let batch_id = BatchId::new();
    let summary = BackpressureParallelExecutor::execute_flow_controlled(
        batch_id,
        tasks,
        backpressure_config,
        batch_config,
        |task| async move {
            sleep(Duration::from_millis(15)).await;
            Ok(task.payload * 5)
        },
        None,
        None,
    )
    .await
    .unwrap();

    assert_eq!(summary.batch_id, batch_id);
    assert_eq!(summary.total_items, 6);
    assert_eq!(summary.succeeded_count, 6);
    assert_eq!(summary.failed_count, 0);
    assert_eq!(summary.item_results.len(), 6);
    assert_eq!(summary.item_results[0].value, Some(0));
    assert_eq!(summary.item_results[5].value, Some(25));
}
