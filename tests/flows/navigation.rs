use crate::common::{
    cleanup_test_dir, create_key_event, create_test_client, get_test_dir, unique_remote_name,
};
use crossterm::event::KeyCode;
use lazyfile::app::{App, Handler, Panel};
use std::collections::HashMap;
use std::fs;

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_flow_create_remote_and_browse() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_browse");
    let remote_name = unique_remote_name("flow_browse");

    fs::write(test_dir.join("file1.txt"), "content1").unwrap();
    fs::write(test_dir.join("file2.txt"), "content2").unwrap();
    fs::create_dir(test_dir.join("subdir")).unwrap();
    fs::write(test_dir.join("subdir/nested.txt"), "nested content").unwrap();

    let mut params = HashMap::new();
    params.insert("remote".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "alias", params)
        .await
        .unwrap();

    let mut app = App::new(create_test_client());
    app.load_remotes().await.unwrap();

    let remote_idx = app.remotes.iter().position(|r| r == &remote_name);
    assert!(remote_idx.is_some(), "Remote should be in list");

    let idx = remote_idx.unwrap();
    for _ in 0..idx {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char('j')))
            .await
            .unwrap();
    }

    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(app.focused_panel, Panel::Files);
    assert_eq!(app.current_remote, Some(remote_name.clone()));
    assert_eq!(app.files.len(), 3);

    let subdir_idx = app.files.iter().position(|f| f.name() == "subdir");
    assert!(subdir_idx.is_some());
    for _ in 0..subdir_idx.unwrap() {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Char('j')))
            .await
            .unwrap();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(app.current_path, "subdir");
    assert_eq!(app.files.len(), 1);

    Handler::handle_key(&mut app, create_key_event(KeyCode::Backspace))
        .await
        .unwrap();
    assert_eq!(app.current_path, "");
    assert_eq!(app.files.len(), 3);

    Handler::handle_key(&mut app, create_key_event(KeyCode::Backspace))
        .await
        .unwrap();
    assert_eq!(app.focused_panel, Panel::Remotes);
    assert!(app.current_remote.is_none());

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_browse");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_flow_deep_navigation() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_deep");
    let remote_name = unique_remote_name("flow_deep");

    fs::create_dir_all(test_dir.join("a/b/c/d")).unwrap();
    fs::write(test_dir.join("a/b/c/d/deep.txt"), "very deep").unwrap();

    let mut params = HashMap::new();
    params.insert("remote".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "alias", params)
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

    for dir in &["a", "b", "c", "d"] {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
            .await
            .unwrap();
        assert!(app.current_path.ends_with(dir));
    }

    assert_eq!(app.files.len(), 1);
    assert_eq!(app.files[0].name(), "deep.txt");

    for _ in 0..4 {
        Handler::handle_key(&mut app, create_key_event(KeyCode::Backspace))
            .await
            .unwrap();
    }
    assert_eq!(app.current_path, "");

    Handler::handle_key(&mut app, create_key_event(KeyCode::Backspace))
        .await
        .unwrap();
    assert_eq!(app.focused_panel, Panel::Remotes);

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_deep");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_flow_panel_switching() {
    let client = create_test_client();
    let test_dir = get_test_dir("flow_switch");
    let remote_name = unique_remote_name("flow_switch");

    fs::write(test_dir.join("file.txt"), "content").unwrap();

    let mut params = HashMap::new();
    params.insert("remote".to_string(), test_dir.to_string_lossy().to_string());
    client
        .create_remote(&remote_name, "alias", params)
        .await
        .unwrap();

    let mut app = App::new(create_test_client());
    app.load_remotes().await.unwrap();

    assert_eq!(app.focused_panel, Panel::Remotes);

    Handler::handle_key(&mut app, create_key_event(KeyCode::Tab))
        .await
        .unwrap();
    assert_eq!(app.focused_panel, Panel::Files);
    assert!(app.files.is_empty());

    Handler::handle_key(&mut app, create_key_event(KeyCode::Tab))
        .await
        .unwrap();
    assert_eq!(app.focused_panel, Panel::Remotes);

    let idx = app.remotes.iter().position(|r| r == &remote_name).unwrap();
    for _ in 0..idx {
        app.navigate_down();
    }
    Handler::handle_key(&mut app, create_key_event(KeyCode::Enter))
        .await
        .unwrap();

    assert_eq!(app.focused_panel, Panel::Files);
    assert_eq!(app.files.len(), 1);

    Handler::handle_key(&mut app, create_key_event(KeyCode::Tab))
        .await
        .unwrap();
    assert_eq!(app.focused_panel, Panel::Remotes);
    assert_eq!(app.current_remote, Some(remote_name.clone()));

    client.delete_remote(&remote_name).await.unwrap();
    cleanup_test_dir("flow_switch");
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_flow_quit() {
    let client = create_test_client();
    let mut app = App::new(client);
    assert!(app.running);

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('q')))
        .await
        .unwrap();
    assert!(!app.running);
}

#[tokio::test]
#[ignore = "requires rclone daemon on localhost:5572"]
async fn test_flow_quit_from_any_panel() {
    let client = create_test_client();
    let mut app = App::new(client);

    app.switch_panel();
    assert_eq!(app.focused_panel, Panel::Files);

    Handler::handle_key(&mut app, create_key_event(KeyCode::Char('q')))
        .await
        .unwrap();
    assert!(!app.running);
}
