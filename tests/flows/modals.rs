use crate::common::{
    cleanup_test_dir, create_key_event, create_test_client, get_test_dir, unique_remote_name,
};
use crossterm::event::KeyCode;
use lazyfile::app::{App, Handler};
use std::collections::HashMap;
use std::fs;

#[tokio::test]
async fn test_flow_cancel_modal_with_escape() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_cancel");
    let remote_name = unique_remote_name("flow_cancel");

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
    assert!(app.file_operations_modal().is_some());

    for c in "test".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }

    Handler::handle_key(&mut app, create_key_event(KeyCode::Esc))
        .await
        .unwrap();

    assert!(app.file_operations_modal().is_none());
    assert!(!test_dir.join("test").exists());

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_cancel");
}

#[tokio::test]
async fn test_flow_modal_validation_error() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_error");
    let remote_name = unique_remote_name("flow_error");

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

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert!(app.file_operations_modal().is_some());
    let modal = app.file_operations_modal().unwrap();
    assert!(modal.error.is_some());

    for c in "valid_dir".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert!(app.file_operations_modal().is_none());
    assert!(test_dir.join("valid_dir").exists());

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_error");
}

#[tokio::test]
async fn test_flow_modal_backspace_clears_input() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_backspace");
    let remote_name = unique_remote_name("flow_backspace");

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

    for c in "wrong".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }

    for _ in 0..5 {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Backspace))
            .await
            .unwrap();
    }

    for c in "correct".chars() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char(c)))
            .await
            .unwrap();
    }

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert!(!test_dir.join("wrong").exists());
    assert!(test_dir.join("correct").exists());

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_backspace");
}

#[tokio::test]
async fn test_flow_copy_modal_with_existing_file() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_copy_existing");
    let remote_name = unique_remote_name("flow_copy_existing");

    fs::write(test_dir.join("source.txt"), "original").unwrap();

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
    assert!(app.file_operations_modal().is_some());

    let modal = app.file_operations_modal().unwrap();
    assert_eq!(modal.file_name, "source.txt");

    Handler::handle_key(&mut app, create_key_event(KeyCode::Esc))
        .await
        .unwrap();

    assert_eq!(app.files.len(), 1);

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_copy_existing");
}
