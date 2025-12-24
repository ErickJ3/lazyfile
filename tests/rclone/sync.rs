use crate::common::{
    TEST_REMOTE, cleanup_test_dir, create_test_client, get_remote_path, get_test_dir,
    setup_test_remote, unique_remote_name,
};
use std::collections::HashMap;
use std::fs;

#[tokio::test]
async fn test_sync_copy() {
    let client = create_test_client();
    setup_test_remote(&client).await;
    let test_dir1 = get_test_dir("sync_src");
    let test_dir2 = get_test_dir("sync_dst");
    let remote_path1 = get_remote_path("sync_src");
    let remote_path2 = get_remote_path("sync_dst");
    let remote_name1 = unique_remote_name("test_sync_src");
    let remote_name2 = unique_remote_name("test_sync_dst");

    fs::write(test_dir1.join("file1.txt"), "content1").unwrap();
    fs::write(test_dir1.join("file2.txt"), "content2").unwrap();
    fs::create_dir(test_dir1.join("subdir")).unwrap();
    fs::write(test_dir1.join("subdir/nested.txt"), "nested").unwrap();

    let mut params1 = HashMap::new();
    params1.insert(
        "remote".to_string(),
        format!("{}:{}", TEST_REMOTE, remote_path1),
    );
    client
        .create_remote(&remote_name1, "alias", params1)
        .await
        .unwrap();

    let mut params2 = HashMap::new();
    params2.insert(
        "remote".to_string(),
        format!("{}:{}", TEST_REMOTE, remote_path2),
    );
    client
        .create_remote(&remote_name2, "alias", params2)
        .await
        .unwrap();

    let result = client.sync_copy(&remote_name1, &remote_name2).await;
    assert!(
        result.is_ok(),
        "sync_copy should succeed: {:?}",
        result.err()
    );

    assert!(test_dir2.join("file1.txt").exists());
    assert!(test_dir2.join("file2.txt").exists());
    assert!(test_dir2.join("subdir/nested.txt").exists());

    let content = fs::read_to_string(test_dir2.join("file1.txt")).unwrap();
    assert_eq!(content, "content1");

    let _ = client.delete_remote(&remote_name1).await;
    let _ = client.delete_remote(&remote_name2).await;
    cleanup_test_dir("sync_src");
    cleanup_test_dir("sync_dst");
}
