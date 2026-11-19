//! Integration tests for API key storage and lifecycle management.

use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;
use web_repo_to_ai_core::auth::{ApiKeyScope, GeneratedApiKey};
use web_repo_to_ai_storage::api_key::{ApiKeyRepository, InMemoryApiKeyRepository};

#[tokio::test]
async fn test_in_memory_api_key_lifecycle() {
    let repo = InMemoryApiKeyRepository::new();
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    let mut scopes = HashSet::new();
    scopes.insert(ApiKeyScope::Read);
    scopes.insert(ApiKeyScope::Write);

    // 1. Generate and save key
    let gen = GeneratedApiKey::generate(tenant_a, "Agent-01", scopes.clone(), None)
        .expect("Generated key");
    let saved = repo.save_key(&gen.record).await.expect("Saved key");
    assert_eq!(saved.name, "Agent-01");
    assert_eq!(saved.tenant_id, tenant_a);

    // 2. Lookup by hash
    let found = repo
        .find_by_hash(&gen.record.key_hash)
        .await
        .expect("Query succeeded")
        .expect("Key exists");
    assert_eq!(found.id, gen.record.id);
    assert_eq!(found.name, "Agent-01");
    assert_eq!(found.scopes, scopes);
    assert!(found.is_active(Utc::now()));

    // 3. Lookup unknown hash returns None
    let unknown = repo
        .find_by_hash("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        .await
        .expect("Query succeeded");
    assert!(unknown.is_none());

    // 4. Duplicate hash rejected
    let dup_res = repo.save_key(&gen.record).await;
    assert!(dup_res.is_err());

    // 5. Tenant isolation in list_by_tenant
    let tenant_a_keys = repo.list_by_tenant(tenant_a).await.expect("List tenant A");
    assert_eq!(tenant_a_keys.len(), 1);
    assert_eq!(tenant_a_keys[0].id, gen.record.id);

    let tenant_b_keys = repo.list_by_tenant(tenant_b).await.expect("List tenant B");
    assert!(tenant_b_keys.is_empty());

    // 6. Record usage
    repo.record_usage(gen.record.id)
        .await
        .expect("Recorded usage");
    let after_usage = repo
        .find_by_id(gen.record.id)
        .await
        .expect("Query succeeded")
        .expect("Found");
    assert!(after_usage.last_used_at.is_some());

    // 7. Revoke key with wrong tenant returns false
    let wrong_tenant_revoke = repo
        .revoke_key(gen.record.id, tenant_b)
        .await
        .expect("Revoke query succeeded");
    assert!(!wrong_tenant_revoke);
    assert!(repo
        .find_by_id(gen.record.id)
        .await
        .unwrap()
        .unwrap()
        .revoked_at
        .is_none());

    // 8. Revoke key with correct tenant returns true
    let correct_revoke = repo
        .revoke_key(gen.record.id, tenant_a)
        .await
        .expect("Revoke query succeeded");
    assert!(correct_revoke);

    // Once revoked, list_by_tenant excludes it
    let active_after_revoke = repo.list_by_tenant(tenant_a).await.expect("List tenant A");
    assert!(active_after_revoke.is_empty());

    // find_by_id shows revoked_at is populated
    let revoked_record = repo.find_by_id(gen.record.id).await.unwrap().unwrap();
    assert!(revoked_record.revoked_at.is_some());
    assert!(!revoked_record.is_active(Utc::now()));
}

#[tokio::test]
async fn test_expired_api_key_inactive() {
    let repo = InMemoryApiKeyRepository::new();
    let tenant_id = Uuid::new_v4();

    let mut scopes = HashSet::new();
    scopes.insert(ApiKeyScope::Read);

    let past_expiry = Utc::now() - chrono::Duration::hours(2);
    let gen = GeneratedApiKey::generate(tenant_id, "Expired Key", scopes, Some(past_expiry))
        .expect("Key generated");

    repo.save_key(&gen.record).await.expect("Saved");

    // list_by_tenant filters out expired keys
    let keys = repo.list_by_tenant(tenant_id).await.expect("List");
    assert!(keys.is_empty());

    // Directly retrieved key indicates is_active is false
    let fetched = repo
        .find_by_id(gen.record.id)
        .await
        .expect("Query")
        .expect("Found");
    assert!(!fetched.is_active(Utc::now()));
}
