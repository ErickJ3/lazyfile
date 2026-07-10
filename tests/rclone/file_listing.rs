use crate::common::{
    TEST_REMOTE, cleanup_test_dir, create_test_client, get_remote_path, get_test_dir,
    setup_test_remote,
};
use std::fs;

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_list_files_empty_directory() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let _test_dir = get_test_dir("list_files_empty");
    let remote_path = get_remote_path("list_files_empty");

    let result = client.list_files(TEST_REMOTE, &remote_path).await;
    assert!(
        result.is_ok(),
        "list_files should succeed: {:?}",
        result.err()
    );

    let files = result.unwrap();
    assert!(files.is_empty(), "Empty directory should return empty list");

    cleanup_test_dir("list_files_empty");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_list_files_with_content() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("list_files_content");
    let remote_path = get_remote_path("list_files_content");

    fs::write(test_dir.join("file1.txt"), "content1").unwrap();
    fs::write(test_dir.join("file2.txt"), "content2").unwrap();
    fs::create_dir(test_dir.join("subdir")).unwrap();

    let result = client.list_files(TEST_REMOTE, &remote_path).await;
    assert!(
        result.is_ok(),
        "list_files should succeed: {:?}",
        result.err()
    );

    let files = result.unwrap();
    assert_eq!(
        files.len(),
        3,
        "Should have 3 items, got {:?}",
        files.iter().map(|f| &f.name).collect::<Vec<_>>()
    );

    let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"file1.txt"));
    assert!(names.contains(&"file2.txt"));
    assert!(names.contains(&"subdir"));

    let subdir = files.iter().find(|f| f.name == "subdir").unwrap();
    assert!(subdir.is_dir, "subdir should be marked as directory");

    let file1 = files.iter().find(|f| f.name == "file1.txt").unwrap();
    assert!(!file1.is_dir, "file1.txt should not be a directory");

    cleanup_test_dir("list_files_content");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_list_files_nested_path() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir = get_test_dir("list_files_nested");
    let remote_path = get_remote_path("list_files_nested");

    let subdir = test_dir.join("level1").join("level2");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("deep_file.txt"), "deep content").unwrap();

    let nested_path = format!("{}/level1/level2", remote_path);
    let result = client.list_files(TEST_REMOTE, &nested_path).await;
    assert!(
        result.is_ok(),
        "list_files should succeed: {:?}",
        result.err()
    );

    let files = result.unwrap();
    assert_eq!(files.len(), 1, "Should have 1 item");
    assert_eq!(files[0].name, "deep_file.txt");

    cleanup_test_dir("list_files_nested");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_list_files_invalid_remote() {
    let client = create_test_client();
    setup_test_remote(&client).await;

    let result = client.list_files("nonexistent_remote_xyz", "").await;
    assert!(
        result.is_err(),
        "list_files with invalid remote should fail"
    );
}
