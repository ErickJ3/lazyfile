use crate::common::{
    TEST_REMOTE, cleanup_test_dir, create_test_client, get_remote_path, get_test_dir,
    setup_test_remote,
};
use lazyfile::rclone::RcloneClient;
use std::fs;

#[tokio::test]
async fn test_client_wrong_port() {
    let client = RcloneClient::new("localhost", 9999);

    let result = client.list_remotes().await;
    assert!(result.is_err(), "Connection to wrong port should fail");
}

#[tokio::test]
async fn test_file_with_spaces_in_name() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("file_spaces");
    let remote_path = get_remote_path("file_spaces");

    fs::write(test_dir.join("file with spaces.txt"), "content").unwrap();

    let files = client.list_files(TEST_REMOTE, &remote_path).await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "file with spaces.txt");

    let file_path = format!("{}/file with spaces.txt", remote_path);
    let result = client.delete_file(TEST_REMOTE, &file_path).await;
    assert!(
        result.is_ok(),
        "delete file with spaces should succeed: {:?}",
        result.err()
    );

    cleanup_test_dir("file_spaces");
}

#[tokio::test]
async fn test_file_with_special_characters() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("file_special");
    let remote_path = get_remote_path("file_special");

    fs::write(test_dir.join("file-with-dashes.txt"), "content").unwrap();
    fs::write(test_dir.join("file_with_underscores.txt"), "content").unwrap();
    fs::write(test_dir.join("file.multiple.dots.txt"), "content").unwrap();

    let files = client.list_files(TEST_REMOTE, &remote_path).await.unwrap();
    assert_eq!(files.len(), 3);

    cleanup_test_dir("file_special");
}

#[tokio::test]
async fn test_unicode_filenames() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("file_unicode");
    let remote_path = get_remote_path("file_unicode");

    fs::write(test_dir.join("arquivo_português.txt"), "content").unwrap();
    fs::write(test_dir.join("файл_русский.txt"), "content").unwrap();
    fs::write(test_dir.join("日本語ファイル.txt"), "content").unwrap();

    let files = client.list_files(TEST_REMOTE, &remote_path).await.unwrap();
    assert_eq!(files.len(), 3);

    let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"arquivo_português.txt"));
    assert!(names.contains(&"файл_русский.txt"));
    assert!(names.contains(&"日本語ファイル.txt"));

    cleanup_test_dir("file_unicode");
}

#[tokio::test]
async fn test_large_file_operations() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("large_file");
    let remote_path = get_remote_path("large_file");

    let large_content = "x".repeat(1024 * 1024);
    fs::write(test_dir.join("large.txt"), &large_content).unwrap();

    let files = client.list_files(TEST_REMOTE, &remote_path).await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].size, 1024 * 1024);

    let src_path = format!("{}/large.txt", remote_path);
    let dst_path = format!("{}/large_copy.txt", remote_path);
    let result = client
        .copy_file(TEST_REMOTE, &src_path, TEST_REMOTE, &dst_path)
        .await;
    assert!(
        result.is_ok(),
        "copy large file should succeed: {:?}",
        result.err()
    );

    let files = client.list_files(TEST_REMOTE, &remote_path).await.unwrap();
    assert_eq!(files.len(), 2);

    cleanup_test_dir("large_file");
}

#[tokio::test]
async fn test_many_files() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("many_files");
    let remote_path = get_remote_path("many_files");

    for i in 0..100 {
        fs::write(
            test_dir.join(format!("file_{:03}.txt", i)),
            format!("content {}", i),
        )
        .unwrap();
    }

    let files = client.list_files(TEST_REMOTE, &remote_path).await.unwrap();
    assert_eq!(files.len(), 100);

    cleanup_test_dir("many_files");
}

#[tokio::test]
async fn test_deeply_nested_directories() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("deep_nested");
    let remote_path = get_remote_path("deep_nested");

    let deep_path = test_dir.join("a/b/c/d/e/f/g/h/i/j");
    fs::create_dir_all(&deep_path).unwrap();
    fs::write(deep_path.join("deep_file.txt"), "deep").unwrap();

    let list_path = format!("{}/a/b/c/d/e/f/g/h/i/j", remote_path);
    let files = client.list_files(TEST_REMOTE, &list_path).await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "deep_file.txt");

    cleanup_test_dir("deep_nested");
}
