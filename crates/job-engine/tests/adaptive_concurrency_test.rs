//! Adaptive concurrency integration test suite.

use std::time::Duration;
use tokio::time::sleep;
use web_repo_to_ai_core::id::{BatchId, JobId};
use web_repo_to_ai_job_engine::adaptive::{
    AdaptiveConcurrencyConfig, AdaptiveConcurrencyController, AdaptiveDecision, AdaptiveOutcome,
    AdaptiveParallelExecutor, DomainAdaptiveConcurrencyManager,
};
use web_repo_to_ai_job_engine::config::BatchExecutionConfig;
use web_repo_to_ai_job_engine::models::BatchItemTask;

#[test]
fn test_adaptive_config_validation() {
    let valid = AdaptiveConcurrencyConfig::new(2, 10, 4);
    assert!(valid.is_ok());

    let invalid_min = AdaptiveConcurrencyConfig::new(0, 10, 4);
    assert!(invalid_min.is_err());

    let invalid_max = AdaptiveConcurrencyConfig::new(5, 3, 4);
    assert!(invalid_max.is_err());

    let invalid_initial = AdaptiveConcurrencyConfig::new(2, 10, 12);
    assert!(invalid_initial.is_err());

    let invalid_decrease = AdaptiveConcurrencyConfig::new(2, 10, 4)
        .unwrap()
        .with_decrease_factor(1.5);
    assert!(invalid_decrease.validate().is_err());
}

#[test]
fn test_adaptive_concurrency_additive_increase() {
    let config = AdaptiveConcurrencyConfig::new(1, 8, 2)
        .unwrap()
        .with_success_threshold(2)
        .with_increase_step(1)
        .with_cooldown(Duration::from_millis(0));

    let controller = AdaptiveConcurrencyController::new(config).unwrap();
    assert_eq!(controller.current_limit(), 2);

    // 1st success: no increase yet
    let d1 = controller.record_outcome(AdaptiveOutcome::Success {
        duration: Duration::from_millis(50),
    });
    assert_eq!(d1, AdaptiveDecision::Unchanged { current_limit: 2 });
    assert_eq!(controller.current_limit(), 2);

    // 2nd success: hits threshold 2, scales up to 3
    let d2 = controller.record_outcome(AdaptiveOutcome::Success {
        duration: Duration::from_millis(60),
    });
    assert_eq!(
        d2,
        AdaptiveDecision::Increased {
            old_limit: 2,
            new_limit: 3
        }
    );
    assert_eq!(controller.current_limit(), 3);

    // 2 more successes scale up to 4
    controller.record_outcome(AdaptiveOutcome::Success {
        duration: Duration::from_millis(40),
    });
    let d4 = controller.record_outcome(AdaptiveOutcome::Success {
        duration: Duration::from_millis(45),
    });
    assert_eq!(
        d4,
        AdaptiveDecision::Increased {
            old_limit: 3,
            new_limit: 4
        }
    );
    assert_eq!(controller.current_limit(), 4);

    let stats = controller.stats();
    assert_eq!(stats.total_successes, 4);
    assert_eq!(stats.total_scale_ups, 2);
    assert_eq!(stats.current_limit, 4);
}

#[test]
fn test_adaptive_concurrency_multiplicative_decrease_rate_limit() {
    let config = AdaptiveConcurrencyConfig::new(2, 10, 8)
        .unwrap()
        .with_decrease_factor(0.5);

    let controller = AdaptiveConcurrencyController::new(config).unwrap();
    assert_eq!(controller.current_limit(), 8);

    // Rate limited signal cuts from 8 to 4
    let decision = controller.record_outcome(AdaptiveOutcome::RateLimited { retry_after: None });
    match decision {
        AdaptiveDecision::Decreased {
            old_limit,
            new_limit,
            ..
        } => {
            assert_eq!(old_limit, 8);
            assert_eq!(new_limit, 4);
        }
        _ => panic!("Expected Decreased decision, got {:?}", decision),
    }
    assert_eq!(controller.current_limit(), 4);

    // Server overload signal cuts from 4 to 2 (hitting min_concurrency floor)
    let decision2 = controller.record_outcome(AdaptiveOutcome::ServerOverloaded);
    match decision2 {
        AdaptiveDecision::Decreased {
            old_limit,
            new_limit,
            ..
        } => {
            assert_eq!(old_limit, 4);
            assert_eq!(new_limit, 2);
        }
        _ => panic!("Expected Decreased decision, got {:?}", decision2),
    }
    assert_eq!(controller.current_limit(), 2);

    // Another cut stays clamped at min_concurrency 2
    let decision3 = controller.record_outcome(AdaptiveOutcome::Timeout);
    match decision3 {
        AdaptiveDecision::Decreased {
            old_limit,
            new_limit,
            ..
        } => {
            assert_eq!(old_limit, 2);
            assert_eq!(new_limit, 2);
        }
        _ => panic!("Expected Decreased decision, got {:?}", decision3),
    }
    assert_eq!(controller.current_limit(), 2);

    let stats = controller.stats();
    assert_eq!(stats.total_rate_limits, 1);
    assert_eq!(stats.total_server_overloads, 1);
    assert_eq!(stats.total_timeouts, 1);
    assert_eq!(stats.total_backoffs, 3);
}

#[tokio::test]
async fn test_adaptive_concurrency_cooldown_enforcement() {
    let config = AdaptiveConcurrencyConfig::new(1, 10, 6)
        .unwrap()
        .with_decrease_factor(0.5)
        .with_success_threshold(1)
        .with_cooldown(Duration::from_millis(150));

    let controller = AdaptiveConcurrencyController::new(config).unwrap();

    // Trigger backoff to limit 3
    controller.record_outcome(AdaptiveOutcome::RateLimited { retry_after: None });
    assert_eq!(controller.current_limit(), 3);

    // Immediate success should be suppressed by cooldown
    let d = controller.record_outcome(AdaptiveOutcome::Success {
        duration: Duration::from_millis(10),
    });
    match d {
        AdaptiveDecision::InCooldown {
            current_limit,
            remaining,
        } => {
            assert_eq!(current_limit, 3);
            assert!(remaining > Duration::ZERO);
        }
        _ => panic!("Expected InCooldown, got {:?}", d),
    }
    assert_eq!(controller.current_limit(), 3);

    // Wait past cooldown
    sleep(Duration::from_millis(160)).await;

    // Now success scales up to 4
    let d2 = controller.record_outcome(AdaptiveOutcome::Success {
        duration: Duration::from_millis(10),
    });
    assert_eq!(
        d2,
        AdaptiveDecision::Increased {
            old_limit: 3,
            new_limit: 4
        }
    );
    assert_eq!(controller.current_limit(), 4);
}

#[test]
fn test_domain_adaptive_manager_isolation() {
    let config = AdaptiveConcurrencyConfig::new(1, 10, 5)
        .unwrap()
        .with_decrease_factor(0.5);

    let manager = DomainAdaptiveConcurrencyManager::new(config);

    let ctrl_a = manager.get_or_create("api.github.com");
    let ctrl_b = manager.get_or_create("crates.io");

    assert_eq!(ctrl_a.current_limit(), 5);
    assert_eq!(ctrl_b.current_limit(), 5);

    // github throttles down
    manager.record_outcome(
        "api.github.com",
        AdaptiveOutcome::RateLimited { retry_after: None },
    );

    assert_eq!(ctrl_a.current_limit(), 2);
    // crates.io must remain unaffected
    assert_eq!(ctrl_b.current_limit(), 5);

    let domain_stats = manager.domains_stats();
    assert_eq!(domain_stats.len(), 2);
    assert_eq!(domain_stats.get("api.github.com").unwrap().current_limit, 2);
    assert_eq!(domain_stats.get("crates.io").unwrap().current_limit, 5);
}

#[tokio::test]
async fn test_adaptive_parallel_executor_execution() {
    let adaptive_config = AdaptiveConcurrencyConfig::new(1, 6, 2)
        .unwrap()
        .with_success_threshold(2)
        .with_increase_step(1)
        .with_cooldown(Duration::from_millis(0));

    let controller = AdaptiveConcurrencyController::new(adaptive_config).unwrap();
    let batch_config = BatchExecutionConfig::default().with_item_timeout(Duration::from_secs(5));

    let executor = AdaptiveParallelExecutor::new(controller.clone(), batch_config);

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
    let summary = executor
        .execute_batch(
            batch_id,
            tasks,
            |task| async move {
                if task.payload == 3 {
                    // Simulate rate limit
                    Err("HTTP 429 Too Many Requests: Rate limit exceeded".to_string())
                } else {
                    sleep(Duration::from_millis(10)).await;
                    Ok(task.payload * 10)
                }
            },
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(summary.total_items, 6);
    assert_eq!(summary.succeeded_count, 5);
    assert_eq!(summary.failed_count, 1);
    assert_eq!(summary.timed_out_count, 0);

    let stats = controller.stats();
    assert_eq!(stats.total_successes, 5);
    assert_eq!(stats.total_rate_limits, 1);
}
