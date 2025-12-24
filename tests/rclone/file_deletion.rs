use crate::common::{
    TEST_REMOTE, cleanup_test_dir, create_test_client, get_remote_path, get_test_dir,
    setup_test_remote,
};
use std::fs;

#[tokio::test]
async fn test_delete_file() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("delete_file");
    let remote_path = get_remote_path("delete_file");

    fs::write(test_dir.join("to_delete.txt"), "delete me").unwrap();
    assert!(test_dir.join("to_delete.txt").exists());

    let file_path = format!("{}/to_delete.txt", remote_path);
    let result = client.delete_file(TEST_REMOTE, &file_path).await;
    assert!(
        result.is_ok(),
        "delete_file should succeed: {:?}",
        result.err()
    );

    assert!(
        !test_dir.join("to_delete.txt").exists(),
        "File should be deleted"
    );

    cleanup_test_dir("delete_file");
}

#[tokio::test]
async fn test_delete_file_in_subdirectory() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("delete_file_subdir");
    let remote_path = get_remote_path("delete_file_subdir");

    fs::create_dir_all(test_dir.join("subdir")).unwrap();
    fs::write(test_dir.join("subdir/nested_file.txt"), "nested").unwrap();

    let file_path = format!("{}/subdir/nested_file.txt", remote_path);
    let result = client.delete_file(TEST_REMOTE, &file_path).await;
    assert!(
        result.is_ok(),
        "delete_file in subdir should succeed: {:?}",
        result.err()
    );

    assert!(!test_dir.join("subdir/nested_file.txt").exists());
    assert!(
        test_dir.join("subdir").exists(),
        "Directory should still exist"
    );

    cleanup_test_dir("delete_file_subdir");
}

#[tokio::test]
async fn test_delete_nonexistent_file() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let _test_dir = get_test_dir("delete_nonexistent");
    let remote_path = get_remote_path("delete_nonexistent");

    let file_path = format!("{}/nonexistent.txt", remote_path);
    let result = client.delete_file(TEST_REMOTE, &file_path).await;
    assert!(
        result.is_err(),
        "delete_file on nonexistent file should fail"
    );

    cleanup_test_dir("delete_nonexistent");
}

#[tokio::test]
async fn test_purge_directory() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("purge_dir");
    let remote_path = get_remote_path("purge_dir");

    fs::create_dir_all(test_dir.join("to_purge")).unwrap();
    fs::write(test_dir.join("to_purge/file1.txt"), "content1").unwrap();
    fs::write(test_dir.join("to_purge/file2.txt"), "content2").unwrap();
    fs::create_dir(test_dir.join("to_purge/subdir")).unwrap();
    fs::write(test_dir.join("to_purge/subdir/nested.txt"), "nested").unwrap();

    assert!(test_dir.join("to_purge/subdir/nested.txt").exists());

    let purge_path = format!("{}/to_purge", remote_path);
    let result = client.purge(TEST_REMOTE, &purge_path).await;
    assert!(result.is_ok(), "purge should succeed: {:?}", result.err());

    assert!(
        !test_dir.join("to_purge").exists(),
        "Directory should be purged"
    );

    cleanup_test_dir("purge_dir");
}

#[tokio::test]
async fn test_purge_empty_directory() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("purge_empty");
    let remote_path = get_remote_path("purge_empty");

    fs::create_dir_all(test_dir.join("empty_dir")).unwrap();

    let purge_path = format!("{}/empty_dir", remote_path);
    let result = client.purge(TEST_REMOTE, &purge_path).await;
    assert!(
        result.is_ok(),
        "purge empty dir should succeed: {:?}",
        result.err()
    );

    assert!(!test_dir.join("empty_dir").exists());

    cleanup_test_dir("purge_empty");
}
