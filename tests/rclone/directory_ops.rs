use crate::common::{
    TEST_REMOTE, cleanup_test_dir, create_test_client, get_remote_path, get_test_dir,
    setup_test_remote,
};
use std::fs;

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_mkdir_simple() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("mkdir_simple");
    let remote_path = get_remote_path("mkdir_simple");

    let new_dir_path = format!("{}/new_directory", remote_path);
    let result = client.mkdir(TEST_REMOTE, &new_dir_path).await;
    assert!(result.is_ok(), "mkdir should succeed: {:?}", result.err());

    assert!(
        test_dir.join("new_directory").exists(),
        "Directory should exist on filesystem"
    );
    assert!(
        test_dir.join("new_directory").is_dir(),
        "Should be a directory"
    );

    cleanup_test_dir("mkdir_simple");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_mkdir_nested() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("mkdir_nested");
    let remote_path = get_remote_path("mkdir_nested");

    let nested_path = format!("{}/parent/child/grandchild", remote_path);
    let result = client.mkdir(TEST_REMOTE, &nested_path).await;
    assert!(
        result.is_ok(),
        "mkdir should succeed for nested path: {:?}",
        result.err()
    );

    assert!(test_dir.join("parent/child/grandchild").exists());

    cleanup_test_dir("mkdir_nested");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_mkdir_already_exists() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("mkdir_exists");
    let remote_path = get_remote_path("mkdir_exists");

    fs::create_dir(test_dir.join("existing")).unwrap();

    let existing_path = format!("{}/existing", remote_path);
    let result = client.mkdir(TEST_REMOTE, &existing_path).await;
    assert!(
        result.is_ok(),
        "mkdir on existing dir should succeed: {:?}",
        result.err()
    );

    cleanup_test_dir("mkdir_exists");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_mkdir_invalid_remote() {
    let client = create_test_client();
    setup_test_remote(&client).await;

    let result = client.mkdir("nonexistent_remote_xyz", "newdir").await;
    assert!(result.is_err(), "mkdir with invalid remote should fail");
}
