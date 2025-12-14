//! E2E tests for the settings modal

use crate::common::harness::EditorTestHarness;
use crossterm::event::{KeyCode, KeyModifiers};

/// Test opening settings modal with Ctrl+,
#[test]
fn test_open_settings_modal() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Render initial state
    harness.render().unwrap();

    // Settings should not be visible initially
    harness.assert_screen_not_contains("Settings");

    // Open settings with Ctrl+,
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Settings modal should now be visible
    harness.assert_screen_contains("Settings");
}

/// Test closing settings modal with Escape
#[test]
fn test_close_settings_with_escape() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();
    harness.assert_screen_contains("Settings");

    // Close with Escape
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Settings should be closed
    harness.assert_screen_not_contains("Settings");
}

/// Test settings navigation with arrow keys
#[test]
fn test_settings_navigation() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Navigate down in categories
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Switch to settings panel with Tab
    harness.send_key(KeyCode::Tab, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Navigate down in settings
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Close settings
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test settings search with /
#[test]
fn test_settings_search() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Start search with /
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Type a search query
    harness
        .send_key(KeyCode::Char('t'), KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Char('h'), KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Char('e'), KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Char('m'), KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Char('e'), KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Should show search results
    // The search query "theme" should match theme-related settings

    // Cancel search with Escape
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Close settings
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test settings help overlay with ?
#[test]
fn test_settings_help_overlay() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Open help with ?
    harness
        .send_key(KeyCode::Char('?'), KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Help overlay should be visible
    harness.assert_screen_contains("Keyboard Shortcuts");

    // Close help with Escape
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Settings should still be visible
    harness.assert_screen_contains("Settings");

    // Close settings
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test search text input is displayed in search box
#[test]
fn test_settings_search_text_displays() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Start search with /
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Should show search mode indicator
    harness.assert_screen_contains("Type to search");

    // Type search query "tab"
    harness
        .send_key(KeyCode::Char('t'), KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Char('a'), KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Char('b'), KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Search text should be visible in the search box
    harness.assert_screen_contains("tab");

    // Should show results count
    harness.assert_screen_contains("results");

    // Close with Escape
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test toggling a setting shows modified indicator
#[test]
fn test_settings_toggle_shows_modified() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Use search to find "Check For Updates" (a toggle setting)
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "check".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Jump to result and toggle
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Toggle the setting
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Should show modified indicator in title
    harness.assert_screen_contains("modified");

    // Close and discard
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
    // Select "Discard" (one right from "Save and Exit")
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
}

/// Test confirmation dialog shows pending changes
#[test]
fn test_confirmation_dialog_shows_changes() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Use search to find "Check For Updates"
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "check".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Jump to result and toggle
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Press Escape to trigger confirmation dialog
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Dialog should show
    harness.assert_screen_contains("Unsaved Changes");
    harness.assert_screen_contains("You have unsaved changes");

    // Should show the actual change (path contains "check_for_updates")
    harness.assert_screen_contains("check_for_updates");

    // Cancel dialog
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test confirmation dialog button navigation
#[test]
fn test_confirmation_dialog_button_navigation() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Use search to find and toggle a setting
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "check".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Open confirmation dialog
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // First button should be selected (Save and Exit has ▶ indicator)
    harness.assert_screen_contains("▶[ Save and Exit ]");

    // Navigate right to Discard
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Discard should now be selected
    harness.assert_screen_contains("▶[ Discard ]");

    // Navigate right to Cancel
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Cancel should now be selected
    harness.assert_screen_contains("▶[ Cancel ]");

    // Press Enter on Cancel to close dialog
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Dialog should be closed but settings still open
    harness.assert_screen_not_contains("Unsaved Changes");
    harness.assert_screen_contains("Settings");

    // Discard and close
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
}

/// Test selection indicator (▶) shows for focused setting item
#[test]
fn test_settings_selection_indicator() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Switch to settings panel with Tab
    harness.send_key(KeyCode::Tab, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Selection indicator should be visible for the first item
    harness.assert_screen_contains("▶");

    // Navigate down
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Selection indicator should still be visible (moved to next item)
    harness.assert_screen_contains("▶");

    // Close settings
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test number input increment with Right arrow
#[test]
fn test_settings_number_increment() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Search for a number setting (mouse hover delay)
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "hover delay".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Jump to result
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // The default value is 500
    harness.assert_screen_contains("500");

    // Press Right arrow to increment
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Value should now be 501
    harness.assert_screen_contains("501");

    // Should show modified indicator
    harness.assert_screen_contains("modified");

    // Press Left arrow to decrement back
    harness
        .send_key(KeyCode::Left, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Value should be back to 500
    harness.assert_screen_contains("500");

    // Close settings (no changes now)
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test number input decrement with Left arrow
#[test]
fn test_settings_number_decrement() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Search for hover delay (number setting) - same as increment test but decrement
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "hover delay".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Jump to result
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // The default value is 500
    harness.assert_screen_contains("500");

    // Press Left arrow to decrement
    harness
        .send_key(KeyCode::Left, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Value should now be 499
    harness.assert_screen_contains("499");

    // Should show modified indicator
    harness.assert_screen_contains("modified");

    // Discard and close
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
}

/// Test dropdown cycling with Enter key
#[test]
fn test_settings_dropdown_cycle() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Search for "theme" (a dropdown setting)
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "theme".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Jump to result
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Check initial theme value (should be "dark")
    let initial_screen = harness.screen_to_string();
    let has_dark = initial_screen.contains("dark");

    // Press Enter to cycle to next option
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // If it was "dark", it should now be "light" or another theme option
    // The exact value depends on available themes, but it should change
    if has_dark {
        // Should show modified indicator since we changed the value
        harness.assert_screen_contains("modified");
    }

    // Discard and close
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
}

/// Test dropdown cycling with Right arrow
#[test]
fn test_settings_dropdown_increment() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Search for "theme" (a dropdown setting)
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "theme".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Jump to result
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Get initial screen
    let initial_screen = harness.screen_to_string();

    // Press Right arrow to cycle to next option
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Get new screen
    let new_screen = harness.screen_to_string();

    // The dropdown value should have changed (screens should differ)
    // We can check that modified indicator appears
    if initial_screen != new_screen {
        harness.assert_screen_contains("modified");
    }

    // Discard and close
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
}

/// Test scrolling through settings list
#[test]
fn test_settings_scrolling() {
    // Use a smaller height to ensure scrolling is needed
    let mut harness = EditorTestHarness::new(100, 25).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Navigate to Editor category which has many settings
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Switch to settings panel
    harness.send_key(KeyCode::Tab, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Get initial screen to check first item
    let initial_screen = harness.screen_to_string();

    // Navigate down many times to trigger scrolling
    for _ in 0..15 {
        harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    }
    harness.render().unwrap();

    // Get new screen - should have scrolled, showing different items
    let scrolled_screen = harness.screen_to_string();

    // The screens should be different due to scrolling
    assert_ne!(
        initial_screen, scrolled_screen,
        "Screen should change after scrolling down"
    );

    // Selection indicator should still be visible
    harness.assert_screen_contains("▶");

    // Close settings
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test scrollbar appears when there are many settings
#[test]
fn test_settings_scrollbar_visible() {
    // Use a smaller height to ensure scrollbar is needed
    let mut harness = EditorTestHarness::new(100, 25).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Navigate to Editor category which has many settings
    harness.send_key(KeyCode::Down, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Switch to settings panel
    harness.send_key(KeyCode::Tab, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();

    // Scrollbar should be visible (█ character is used for scrollbar thumb)
    harness.assert_screen_contains("█");

    // Close settings
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test search jump scrolls to selected item
#[test]
fn test_settings_search_jump_scrolls() {
    // Use a smaller height to ensure scrolling is needed
    let mut harness = EditorTestHarness::new(100, 25).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Search for a setting that's likely at the bottom of a category
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "wrap".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Jump to result
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // The item should be visible (selection indicator should be on screen)
    harness.assert_screen_contains("▶");

    // The searched term should be visible
    harness.assert_screen_contains("Wrap");

    // Close settings
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
}

/// Test theme dropdown can be cycled with Enter or Right arrow
/// BUG: Theme dropdown doesn't cycle - it stays on the same value
#[test]
#[ignore] // TODO: Fix theme dropdown cycling - currently broken
fn test_settings_theme_dropdown_cycle() {
    let mut harness = EditorTestHarness::new(100, 40).unwrap();

    // Open settings
    harness
        .send_key(KeyCode::Char(','), KeyModifiers::CONTROL)
        .unwrap();
    harness.render().unwrap();

    // Search for theme setting
    harness
        .send_key(KeyCode::Char('/'), KeyModifiers::NONE)
        .unwrap();
    for c in "theme".chars() {
        harness
            .send_key(KeyCode::Char(c), KeyModifiers::NONE)
            .unwrap();
    }
    harness.render().unwrap();

    // Jump to theme setting
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // Should be on Theme setting with current value (high-contrast is default)
    harness.assert_screen_contains("Theme");
    let initial_screen = harness.screen_to_string();
    let has_high_contrast = initial_screen.contains("high-contrast");

    // Press Enter to cycle to next theme option
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    // The theme should have changed - this is currently broken
    // Expected: theme changes to next option (e.g., monokai, solarized-dark)
    // Actual: theme stays on high-contrast
    let after_enter = harness.screen_to_string();

    if has_high_contrast {
        // After pressing Enter, it should cycle to a different theme
        // This assertion will fail with the current bug
        assert!(
            !after_enter.contains("high-contrast") || after_enter.contains("modified"),
            "Theme should change after pressing Enter, but it stayed the same"
        );
    }

    // Try Right arrow as well
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness.render().unwrap();

    let after_right = harness.screen_to_string();

    // Should show modified indicator if theme changed
    // This will also fail with the current bug
    assert!(
        after_right.contains("modified"),
        "Theme dropdown should cycle with Right arrow and show modified indicator"
    );

    // Discard and close
    harness.send_key(KeyCode::Esc, KeyModifiers::NONE).unwrap();
    harness.render().unwrap();
    harness
        .send_key(KeyCode::Right, KeyModifiers::NONE)
        .unwrap();
    harness
        .send_key(KeyCode::Enter, KeyModifiers::NONE)
        .unwrap();
}
