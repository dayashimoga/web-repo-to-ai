//! Integration tests for secure token storage, envelope encryption, and tenant isolation under Rule 05.

use chrono::{Duration, Utc};
use uuid::Uuid;
use web_repo_to_ai_core::crypto::EncryptionKey;
use web_repo_to_ai_storage::token_storage::{InMemoryTokenStorage, TokenStorageRepository};

#[tokio::test]
async fn test_in_memory_token_storage_save_and_retrieve() {
    let key = EncryptionKey::generate().expect("key generation succeeds");
    let storage = InMemoryTokenStorage::new(key);

    let session_id = Uuid::new_v4();
    let install_id = 987654321i64;
    let login = "octocat-org";
    let token = "ghs_SecretInstallationAccessToken1234567890abcdef";
    let expires_at = Utc::now() + Duration::hours(1);

    let saved_row = storage
        .save_installation_token(session_id, install_id, login, token, expires_at)
        .await
        .expect("saving token succeeds");

    assert_eq!(saved_row.owner_session_id, session_id);
    assert_eq!(saved_row.github_installation_id, install_id);
    assert_eq!(saved_row.account_login, login);
    assert!(saved_row.encrypted_token.is_some());
    assert!(saved_row.token_nonce.is_some());
    assert_eq!(saved_row.token_expires_at, Some(expires_at));
    assert!(saved_row.revoked_at.is_none());
    assert!(saved_row.is_active());

    // Retrieve active token
    let active_token = storage
        .get_active_token(session_id, install_id)
        .await
        .expect("retrieval succeeds");
    assert_eq!(active_token.as_deref(), Some(token));

    // Get metadata
    let metadata = storage
        .get_installation(session_id, install_id)
        .await
        .expect("metadata query succeeds");
    assert!(metadata.is_some());
    let meta = metadata.unwrap();
    assert_eq!(meta.account_login, login);
}

#[tokio::test]
async fn test_in_memory_token_storage_tenant_isolation() {
    let key = EncryptionKey::generate().expect("key generation succeeds");
    let storage = InMemoryTokenStorage::new(key);

    let session_a = Uuid::new_v4();
    let session_b = Uuid::new_v4();
    let install_id = 11223344i64;
    let token = "ghs_SecretTokenForSessionA";
    let expires_at = Utc::now() + Duration::hours(1);

    storage
        .save_installation_token(session_a, install_id, "user-a", token, expires_at)
        .await
        .expect("save succeeds");

    // Session A accesses token -> OK
    let token_a = storage
        .get_active_token(session_a, install_id)
        .await
        .expect("retrieval succeeds");
    assert_eq!(token_a.as_deref(), Some(token));

    // Session B tries to access same installation ID -> None
    let token_b = storage
        .get_active_token(session_b, install_id)
        .await
        .expect("query succeeds");
    assert!(token_b.is_none());

    // Session A tries non-existent installation ID -> None
    let token_wrong_install = storage
        .get_active_token(session_a, 99999999i64)
        .await
        .expect("query succeeds");
    assert!(token_wrong_install.is_none());
}

#[tokio::test]
async fn test_in_memory_token_storage_expiration() {
    let key = EncryptionKey::generate().expect("key generation succeeds");
    let storage = InMemoryTokenStorage::new(key);

    let session_id = Uuid::new_v4();
    let install_id = 55667788i64;
    let token = "ghs_ExpiredToken";
    // Expired 10 minutes ago
    let expires_at = Utc::now() - Duration::minutes(10);

    storage
        .save_installation_token(session_id, install_id, "expired-user", token, expires_at)
        .await
        .expect("save succeeds");

    // Active token retrieval returns None because token is expired
    let retrieved = storage
        .get_active_token(session_id, install_id)
        .await
        .expect("query succeeds");
    assert!(retrieved.is_none());

    // Listing active installations also filters it out
    let active_list = storage
        .list_active_installations(session_id)
        .await
        .expect("listing succeeds");
    assert!(active_list.is_empty());

    // Metadata still exists
    let meta = storage
        .get_installation(session_id, install_id)
        .await
        .expect("query succeeds");
    assert!(meta.is_some());
    assert!(!meta.unwrap().is_active());
}

#[tokio::test]
async fn test_in_memory_token_storage_revocation() {
    let key = EncryptionKey::generate().expect("key generation succeeds");
    let storage = InMemoryTokenStorage::new(key);

    let session_id = Uuid::new_v4();
    let install_id = 99887766i64;
    let token = "ghs_RevokeMeToken";
    let expires_at = Utc::now() + Duration::hours(2);

    storage
        .save_installation_token(session_id, install_id, "revocable-user", token, expires_at)
        .await
        .expect("save succeeds");

    // First revocation returns true
    let revoked = storage
        .revoke_installation(session_id, install_id)
        .await
        .expect("revocation succeeds");
    assert!(revoked);

    // Second revocation returns false (already revoked)
    let second_revoked = storage
        .revoke_installation(session_id, install_id)
        .await
        .expect("revocation succeeds");
    assert!(!second_revoked);

    // Active token retrieval returns None
    let active = storage
        .get_active_token(session_id, install_id)
        .await
        .expect("query succeeds");
    assert!(active.is_none());

    // Listing active installations is empty
    let list = storage
        .list_active_installations(session_id)
        .await
        .expect("list succeeds");
    assert!(list.is_empty());

    // Metadata reflects revocation and wiped credentials
    let meta = storage
        .get_installation(session_id, install_id)
        .await
        .expect("query succeeds")
        .expect("row exists");
    assert!(meta.revoked_at.is_some());
    assert!(meta.encrypted_token.is_none());
    assert!(meta.token_nonce.is_none());
}

#[tokio::test]
async fn test_in_memory_token_storage_deletion() {
    let key = EncryptionKey::generate().expect("key generation succeeds");
    let storage = InMemoryTokenStorage::new(key);

    let session_id = Uuid::new_v4();
    let install_id = 33445566i64;
    let token = "ghs_DeleteMeToken";
    let expires_at = Utc::now() + Duration::hours(1);

    storage
        .save_installation_token(session_id, install_id, "deleted-user", token, expires_at)
        .await
        .expect("save succeeds");

    // Deletion succeeds
    let deleted = storage
        .delete_installation(session_id, install_id)
        .await
        .expect("delete succeeds");
    assert!(deleted);

    // Second deletion returns false
    let second_deleted = storage
        .delete_installation(session_id, install_id)
        .await
        .expect("delete succeeds");
    assert!(!second_deleted);

    // Query returns None
    let meta = storage
        .get_installation(session_id, install_id)
        .await
        .expect("query succeeds");
    assert!(meta.is_none());
}

#[tokio::test]
async fn test_in_memory_token_storage_upsert_and_listing() {
    let key = EncryptionKey::generate().expect("key generation succeeds");
    let storage = InMemoryTokenStorage::new(key);

    let session_a = Uuid::new_v4();
    let session_b = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(1);

    // Save 3 for session A
    storage
        .save_installation_token(session_a, 101, "charlie-org", "tok_c", expires_at)
        .await
        .unwrap();
    storage
        .save_installation_token(session_a, 102, "alice-org", "tok_a", expires_at)
        .await
        .unwrap();
    storage
        .save_installation_token(session_a, 103, "bob-org", "tok_b", expires_at)
        .await
        .unwrap();

    // Save 1 for session B
    storage
        .save_installation_token(session_b, 201, "other-org", "tok_other", expires_at)
        .await
        .unwrap();

    // List for session A - should be sorted by account_login: alice, bob, charlie
    let list_a = storage.list_active_installations(session_a).await.unwrap();
    assert_eq!(list_a.len(), 3);
    assert_eq!(list_a[0].account_login, "alice-org");
    assert_eq!(list_a[1].account_login, "bob-org");
    assert_eq!(list_a[2].account_login, "charlie-org");

    // List for session B - should have 1 item
    let list_b = storage.list_active_installations(session_b).await.unwrap();
    assert_eq!(list_b.len(), 1);
    assert_eq!(list_b[0].account_login, "other-org");

    // Upsert installation 102 with new token and updated login
    storage
        .save_installation_token(session_a, 102, "alice-renamed", "tok_a_updated", expires_at)
        .await
        .unwrap();

    let updated_token = storage.get_active_token(session_a, 102).await.unwrap();
    assert_eq!(updated_token.as_deref(), Some("tok_a_updated"));

    let list_a_after = storage.list_active_installations(session_a).await.unwrap();
    assert_eq!(list_a_after.len(), 3);
    assert_eq!(list_a_after[0].account_login, "alice-renamed");
}

#[tokio::test]
async fn test_in_memory_token_storage_cross_key_tampering_fails() {
    let key1 = EncryptionKey::generate().unwrap();
    let key2 = EncryptionKey::generate().unwrap();

    let storage1 = InMemoryTokenStorage::new(key1);
    let session_id = Uuid::new_v4();
    let install_id = 77777777i64;
    let token = "ghs_SecretKeyIsolationToken";
    let expires_at = Utc::now() + Duration::hours(1);

    let saved = storage1
        .save_installation_token(session_id, install_id, "key-test-org", token, expires_at)
        .await
        .unwrap();

    // Verify decrypting directly with wrong key (key2) fails integrity check
    let err = web_repo_to_ai_core::crypto::decrypt_token(
        &key2,
        saved.encrypted_token.as_ref().unwrap(),
        saved.token_nonce.as_ref().unwrap(),
        &session_id,
        install_id,
    );
    assert!(err.is_err());
}

#[tokio::test]
async fn test_in_memory_token_storage_validation() {
    let key = EncryptionKey::generate().unwrap();
    let storage = InMemoryTokenStorage::new(key);
    let session = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(1);

    // Negative/zero installation ID fails
    let err_zero = storage
        .save_installation_token(session, 0, "valid-login", "token", expires_at)
        .await;
    assert!(err_zero.is_err());

    let err_neg = storage
        .save_installation_token(session, -5, "valid-login", "token", expires_at)
        .await;
    assert!(err_neg.is_err());

    // Empty account login fails
    let err_empty = storage
        .save_installation_token(session, 1234, "   ", "token", expires_at)
        .await;
    assert!(err_empty.is_err());
}

#[tokio::test]
async fn test_in_memory_token_storage_safety_margin_filtering() {
    let key = EncryptionKey::generate().unwrap();
    let storage = InMemoryTokenStorage::new(key);
    let session = Uuid::new_v4();
    let install_id = 99112233i64;
    let token = "ghs_ProactiveRefreshTestToken";
    // Expires in 10 minutes (600 seconds)
    let expires_at = Utc::now() + Duration::minutes(10);

    storage
        .save_installation_token(session, install_id, "margin-user", token, expires_at)
        .await
        .unwrap();

    // 1. With 300s (5 min) margin, token has 600s left -> still fresh -> returns Some
    let token_300 = storage
        .get_active_token_with_margin(session, install_id, 300)
        .await
        .unwrap();
    assert_eq!(token_300.as_deref(), Some(token));

    // 2. With 700s (11 min) margin, token has only 600s left -> expiring soon -> returns None
    let token_700 = storage
        .get_active_token_with_margin(session, install_id, 700)
        .await
        .unwrap();
    assert!(token_700.is_none());

    // 3. Normal get_active_token still returns Some because it is not expired
    let normal = storage.get_active_token(session, install_id).await.unwrap();
    assert_eq!(normal.as_deref(), Some(token));
}

#[tokio::test]
async fn test_in_memory_token_storage_expiration_metadata() {
    let key = EncryptionKey::generate().unwrap();
    let storage = InMemoryTokenStorage::new(key);
    let session = Uuid::new_v4();
    let install_id = 44556677i64;
    let token = "ghs_ExpirationMetadataTest";
    let expires_at = Utc::now() + Duration::hours(1);

    storage
        .save_installation_token(session, install_id, "meta-user", token, expires_at)
        .await
        .unwrap();

    let fetched_exp = storage
        .get_token_expiration(session, install_id)
        .await
        .unwrap();
    assert_eq!(fetched_exp, Some(expires_at));

    // After revocation, get_token_expiration returns None
    storage
        .revoke_installation(session, install_id)
        .await
        .unwrap();
    let revoked_exp = storage
        .get_token_expiration(session, install_id)
        .await
        .unwrap();
    assert!(revoked_exp.is_none());
}
