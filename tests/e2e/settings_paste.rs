use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};

fn send_text(harness: &mut EditorTestHarness, text: &str) {
    for c in text.chars() {
        harness.send_key(KeyCode::Char(c), KeyModifiers::NONE).unwrap();
    }
}

#[test]
fn test_settings_paste() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Set clipboard content to "rust"
    send_text(&mut harness, "rust");
    harness.send_key(KeyCode::Char('a'), KeyModifiers::CONTROL).unwrap();
    harness.send_key(KeyCode::Char('c'), KeyModifiers::CONTROL).unwrap();
    
    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Search for "languages"
    harness.send_key(KeyCode::Char('/'), KeyModifiers::NONE).unwrap();
    send_text(&mut harness, "languages");
    harness.send_key(KeyCode::Enter, KeyModifiers::NONE).unwrap(); // Confirm search
    harness.render().unwrap();
    
    // Enter to open "Add Language" dialog (since it's a Map and we are on "Add new")
    harness.send_key(KeyCode::Enter, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
    
    // Verify Edit Value dialog
    harness.assert_screen_contains("Key");
    
    // Enter to start editing the "Key" field
    harness.send_key(KeyCode::Enter, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Clear existing value "bash"
    for _ in 0..10 {
        harness.send_key(KeyCode::Backspace, KeyModifiers::NONE).unwrap();
    }

    // Paste "rust"
    harness.send_key(KeyCode::Char('v'), KeyModifiers::CONTROL).unwrap();
    harness.render().unwrap();
    
    // Verify content is pasted
    harness.assert_screen_contains("rust");
}
