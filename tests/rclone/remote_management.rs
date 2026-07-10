use crate::common::{TEST_REMOTE, create_test_client, setup_test_remote, unique_remote_name};
use std::collections::HashMap;

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_list_remotes() {
    let client = create_test_client();
    setup_test_remote(&client).await;

    let result = client.list_remotes().await;
    assert!(
        result.is_ok(),
        "list_remotes should succeed: {:?}",
        result.err()
    );

    let remotes = result.unwrap();
    assert!(
        remotes.contains(&TEST_REMOTE.to_string()),
        "Test remote should exist"
    );
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_create_remote_local_type() {
    let client = create_test_client();
    let remote_name = unique_remote_name("test_create");

    let params = HashMap::new();

    let result = client.create_remote(&remote_name, "local", params).await;
    assert!(
        result.is_ok(),
        "create_remote should succeed: {:?}",
        result.err()
    );

    let remotes = client.list_remotes().await.unwrap();
    assert!(remotes.contains(&remote_name), "Remote should be in list");

    let _ = client.delete_remote(&remote_name).await;
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_create_remote_with_params() {
    let client = create_test_client();
    let remote_name = unique_remote_name("test_create_params");

    let mut params = HashMap::new();
    params.insert("nounc".to_string(), "true".to_string());

    let result = client.create_remote(&remote_name, "local", params).await;
    assert!(
        result.is_ok(),
        "create_remote with params should succeed: {:?}",
        result.err()
    );

    let _ = client.delete_remote(&remote_name).await;
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_update_remote() {
    let client = create_test_client();
    let remote_name = unique_remote_name("test_update");

    let params = HashMap::new();
    client
        .create_remote(&remote_name, "local", params)
        .await
        .unwrap();

    let mut new_params = HashMap::new();
    new_params.insert("nounc".to_string(), "true".to_string());

    let result = client.update_remote(&remote_name, new_params).await;
    assert!(
        result.is_ok(),
        "update_remote should succeed: {:?}",
        result.err()
    );

    let _ = client.delete_remote(&remote_name).await;
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_delete_remote() {
    let client = create_test_client();
    let remote_name = unique_remote_name("test_delete");

    let params = HashMap::new();
    client
        .create_remote(&remote_name, "local", params)
        .await
        .unwrap();

    let remotes = client.list_remotes().await.unwrap();
    assert!(remotes.contains(&remote_name));

    let result = client.delete_remote(&remote_name).await;
    assert!(
        result.is_ok(),
        "delete_remote should succeed: {:?}",
        result.err()
    );

    let remotes = client.list_remotes().await.unwrap();
    assert!(!remotes.contains(&remote_name), "Remote should be deleted");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_delete_nonexistent_remote() {
    let client = create_test_client();
    let remote_name = "nonexistent_remote_xyz123";
    let result = client.delete_remote(remote_name).await;
    let _ = result;
}
