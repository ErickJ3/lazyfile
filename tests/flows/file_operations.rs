use crate::common::{
    cleanup_test_dir, create_key_event, create_test_client, get_test_dir, unique_remote_name,
};
use crossterm::event::KeyCode;
use lazyfile::app::{App, Handler};
use std::collections::HashMap;
use std::fs;

#[tokio::test]
async fn test_flow_mkdir_via_modal() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_mkdir");
    let remote_name = unique_remote_name("flow_mkdir");

    let mut params = HashMap::new();
    params.insert("path".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "local", params)
        .await
        .unwrap();

    let mut app = App::new(create_test_client());
    app.load_remotes().await.unwrap();

    let idx = app.remotes.iter().position(|r| r == &remote_name).unwrap();
    for _ in 0..idx {
        app.navigate_down();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('n')))
        .await
        .unwrap();
    assert!(app.file_operations_modal.is_some());

    for c in "new_folder".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert!(app.file_operations_modal.is_none());
    assert!(test_dir.join("new_folder").exists());
    assert!(test_dir.join("new_folder").is_dir());

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_mkdir");
}

#[tokio::test]
async fn test_flow_delete_file_via_modal() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_delete");
    let remote_name = unique_remote_name("flow_delete");

    fs::write(test_dir.join("to_delete.txt"), "delete me").unwrap();

    let mut params = HashMap::new();
    params.insert("path".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "local", params)
        .await
        .unwrap();

    let mut app = App::new(create_test_client());
    app.load_remotes().await.unwrap();

    let idx = app.remotes.iter().position(|r| r == &remote_name).unwrap();
    for _ in 0..idx {
        app.navigate_down();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(app.files.len(), 1);
    assert_eq!(app.files[0].name(), "to_delete.txt");

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('x')))
        .await
        .unwrap();
    assert!(app.file_operations_modal.is_some());

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert!(!test_dir.join("to_delete.txt").exists());
    assert!(app.files.is_empty());

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_delete");
}

#[tokio::test]
async fn test_flow_copy_file_via_modal() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_copy");
    let remote_name = unique_remote_name("flow_copy");

    fs::write(test_dir.join("source.txt"), "copy me").unwrap();

    let mut params = HashMap::new();
    params.insert("path".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "local", params)
        .await
        .unwrap();

    let mut app = App::new(create_test_client());
    app.load_remotes().await.unwrap();

    let idx = app.remotes.iter().position(|r| r == &remote_name).unwrap();
    for _ in 0..idx {
        app.navigate_down();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('c')))
        .await
        .unwrap();
    assert!(app.file_operations_modal.is_some());

    for c in "destination.txt".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert!(test_dir.join("source.txt").exists());
    assert!(test_dir.join("destination.txt").exists());

    let src = fs::read_to_string(test_dir.join("source.txt")).unwrap();
    let dst = fs::read_to_string(test_dir.join("destination.txt")).unwrap();
    assert_eq!(src, dst);

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_copy");
}

#[tokio::test]
async fn test_flow_move_file_via_modal() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_move");
    let remote_name = unique_remote_name("flow_move");

    fs::write(test_dir.join("source.txt"), "move me").unwrap();

    let mut params = HashMap::new();
    params.insert("path".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "local", params)
        .await
        .unwrap();

    let mut app = App::new(create_test_client());
    app.load_remotes().await.unwrap();

    let idx = app.remotes.iter().position(|r| r == &remote_name).unwrap();
    for _ in 0..idx {
        app.navigate_down();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('m')))
        .await
        .unwrap();
    assert!(app.file_operations_modal.is_some());

    for c in "destination.txt".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert!(!test_dir.join("source.txt").exists());
    assert!(test_dir.join("destination.txt").exists());

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_move");
}

#[tokio::test]
async fn test_flow_purge_directory_via_modal() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_purge");
    let remote_name = unique_remote_name("flow_purge");

    fs::create_dir(test_dir.join("to_purge")).unwrap();
    fs::write(test_dir.join("to_purge/file.txt"), "content").unwrap();

    let mut params = HashMap::new();
    params.insert("path".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "local", params)
        .await
        .unwrap();

    let mut app = App::new(create_test_client());
    app.load_remotes().await.unwrap();

    let idx = app.remotes.iter().position(|r| r == &remote_name).unwrap();
    for _ in 0..idx {
        app.navigate_down();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(app.files.len(), 1);
    assert!(app.files[0].is_dir());

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('x')))
        .await
        .unwrap();
    assert!(app.file_operations_modal.is_some());

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert!(!test_dir.join("to_purge").exists());
    assert!(app.files.is_empty());

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_purge");
}

#[tokio::test]
async fn test_flow_multiple_operations() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_multi");
    let remote_name = unique_remote_name("flow_multi");

    let mut params = HashMap::new();
    params.insert("path".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "local", params)
        .await
        .unwrap();

    let mut app = App::new(create_test_client());
    app.load_remotes().await.unwrap();

    let idx = app.remotes.iter().position(|r| r == &remote_name).unwrap();
    for _ in 0..idx {
        app.navigate_down();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('n')))
        .await
        .unwrap();
    for c in "mydir".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();
    assert!(test_dir.join("mydir").exists());

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();
    assert_eq!(app.current_path, "mydir");

    fs::write(test_dir.join("mydir/test.txt"), "test content").unwrap();
    app.load_files().await.unwrap();
    assert_eq!(app.files.len(), 1);

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('c')))
        .await
        .unwrap();
    for c in "test_copy.txt".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();
    assert!(test_dir.join("mydir/test_copy.txt").exists());

    app.files_selected = 0;
    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('x')))
        .await
        .unwrap();
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    app.load_files().await.unwrap();
    assert_eq!(app.files.len(), 1);
    assert_eq!(app.files[0].name(), "test_copy.txt");

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_multi");
}
