use crate::common::{
    TEST_REMOTE, cleanup_test_dir, create_test_client, get_remote_path, get_test_dir,
    setup_test_remote,
};
use std::fs;

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_copy_file_same_remote() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("copy_file");
    let remote_path = get_remote_path("copy_file");

    fs::write(test_dir.join("source.txt"), "source content").unwrap();

    let src_path = format!("{}/source.txt", remote_path);
    let dst_path = format!("{}/destination.txt", remote_path);
    let result = client
        .copy_file(TEST_REMOTE, &src_path, TEST_REMOTE, &dst_path)
        .await;
    assert!(
        result.is_ok(),
        "copy_file should succeed: {:?}",
        result.err()
    );

    assert!(
        test_dir.join("source.txt").exists(),
        "Source should still exist"
    );
    assert!(
        test_dir.join("destination.txt").exists(),
        "Destination should exist"
    );

    let src_content = fs::read_to_string(test_dir.join("source.txt")).unwrap();
    let dst_content = fs::read_to_string(test_dir.join("destination.txt")).unwrap();
    assert_eq!(src_content, dst_content);

    cleanup_test_dir("copy_file");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_copy_file_to_subdirectory() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("copy_file_subdir");
    let remote_path = get_remote_path("copy_file_subdir");

    fs::write(test_dir.join("source.txt"), "content").unwrap();
    fs::create_dir(test_dir.join("dest_dir")).unwrap();

    let src_path = format!("{}/source.txt", remote_path);
    let dst_path = format!("{}/dest_dir/copied.txt", remote_path);
    let result = client
        .copy_file(TEST_REMOTE, &src_path, TEST_REMOTE, &dst_path)
        .await;
    assert!(
        result.is_ok(),
        "copy_file to subdir should succeed: {:?}",
        result.err()
    );

    assert!(test_dir.join("dest_dir/copied.txt").exists());

    cleanup_test_dir("copy_file_subdir");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_copy_nonexistent_file() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let _test_dir = get_test_dir("copy_nonexistent");
    let remote_path = get_remote_path("copy_nonexistent");

    let src_path = format!("{}/nonexistent.txt", remote_path);
    let dst_path = format!("{}/dest.txt", remote_path);
    let result = client
        .copy_file(TEST_REMOTE, &src_path, TEST_REMOTE, &dst_path)
        .await;
    assert!(result.is_err(), "copy_file on nonexistent should fail");

    cleanup_test_dir("copy_nonexistent");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_move_file_same_remote() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("move_file");
    let remote_path = get_remote_path("move_file");

    fs::write(test_dir.join("source.txt"), "move me").unwrap();

    let src_path = format!("{}/source.txt", remote_path);
    let dst_path = format!("{}/destination.txt", remote_path);
    let result = client
        .move_file(TEST_REMOTE, &src_path, TEST_REMOTE, &dst_path)
        .await;
    assert!(
        result.is_ok(),
        "move_file should succeed: {:?}",
        result.err()
    );

    assert!(
        !test_dir.join("source.txt").exists(),
        "Source should be gone"
    );
    assert!(
        test_dir.join("destination.txt").exists(),
        "Destination should exist"
    );

    let content = fs::read_to_string(test_dir.join("destination.txt")).unwrap();
    assert_eq!(content, "move me");

    cleanup_test_dir("move_file");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_move_file_rename() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("move_rename");
    let remote_path = get_remote_path("move_rename");

    fs::write(test_dir.join("old_name.txt"), "content").unwrap();

    let src_path = format!("{}/old_name.txt", remote_path);
    let dst_path = format!("{}/new_name.txt", remote_path);
    let result = client
        .move_file(TEST_REMOTE, &src_path, TEST_REMOTE, &dst_path)
        .await;
    assert!(
        result.is_ok(),
        "move (rename) should succeed: {:?}",
        result.err()
    );

    assert!(!test_dir.join("old_name.txt").exists());
    assert!(test_dir.join("new_name.txt").exists());

    cleanup_test_dir("move_rename");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_move_file_to_subdirectory() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("move_subdir");
    let remote_path = get_remote_path("move_subdir");

    fs::write(test_dir.join("file.txt"), "content").unwrap();
    fs::create_dir(test_dir.join("subdir")).unwrap();

    let src_path = format!("{}/file.txt", remote_path);
    let dst_path = format!("{}/subdir/file.txt", remote_path);
    let result = client
        .move_file(TEST_REMOTE, &src_path, TEST_REMOTE, &dst_path)
        .await;
    assert!(
        result.is_ok(),
        "move to subdir should succeed: {:?}",
        result.err()
    );

    assert!(!test_dir.join("file.txt").exists());
    assert!(test_dir.join("subdir/file.txt").exists());

    cleanup_test_dir("move_subdir");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_move_nonexistent_file() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let _test_dir = get_test_dir("move_nonexistent");
    let remote_path = get_remote_path("move_nonexistent");

    let src_path = format!("{}/nonexistent.txt", remote_path);
    let dst_path = format!("{}/dest.txt", remote_path);
    let result = client
        .move_file(TEST_REMOTE, &src_path, TEST_REMOTE, &dst_path)
        .await;
    assert!(result.is_err(), "move_file on nonexistent should fail");

    cleanup_test_dir("move_nonexistent");
}
