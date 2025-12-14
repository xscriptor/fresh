//! Setting items for the UI
//!
//! Converts schema information into renderable setting items.

use super::schema::{SettingCategory, SettingSchema, SettingType};
use crate::view::controls::{
    DropdownState, NumberInputState, TextInputState, TextListState, ToggleState,
};

/// A renderable setting item
#[derive(Debug, Clone)]
pub struct SettingItem {
    /// JSON pointer path
    pub path: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// The control for this setting
    pub control: SettingControl,
    /// Default value (for reset)
    pub default: Option<serde_json::Value>,
    /// Whether this setting has been modified from default
    pub modified: bool,
}

/// The type of control to render for a setting
#[derive(Debug, Clone)]
pub enum SettingControl {
    Toggle(ToggleState),
    Number(NumberInputState),
    Dropdown(DropdownState),
    Text(TextInputState),
    TextList(TextListState),
    /// Complex settings that can't be edited inline
    Complex { type_name: String },
}

/// A page of settings (corresponds to a category)
#[derive(Debug, Clone)]
pub struct SettingsPage {
    /// Page name
    pub name: String,
    /// JSON path prefix
    pub path: String,
    /// Description
    pub description: Option<String>,
    /// Settings on this page
    pub items: Vec<SettingItem>,
    /// Subpages
    pub subpages: Vec<SettingsPage>,
}

/// Convert a category tree into pages with control states
pub fn build_pages(
    categories: &[SettingCategory],
    config_value: &serde_json::Value,
) -> Vec<SettingsPage> {
    categories
        .iter()
        .map(|cat| build_page(cat, config_value))
        .collect()
}

/// Build a single page from a category
fn build_page(category: &SettingCategory, config_value: &serde_json::Value) -> SettingsPage {
    let items = category
        .settings
        .iter()
        .map(|s| build_item(s, config_value))
        .collect();

    let subpages = category
        .subcategories
        .iter()
        .map(|sub| build_page(sub, config_value))
        .collect();

    SettingsPage {
        name: category.name.clone(),
        path: category.path.clone(),
        description: category.description.clone(),
        items,
        subpages,
    }
}

/// Build a setting item with its control state initialized from current config
fn build_item(schema: &SettingSchema, config_value: &serde_json::Value) -> SettingItem {
    // Get current value from config
    let current_value = config_value.pointer(&schema.path);

    // Create control based on type
    let control = match &schema.setting_type {
        SettingType::Boolean => {
            let checked = current_value
                .and_then(|v| v.as_bool())
                .or_else(|| schema.default.as_ref().and_then(|d| d.as_bool()))
                .unwrap_or(false);
            SettingControl::Toggle(ToggleState::new(checked, &schema.name))
        }

        SettingType::Integer { minimum, maximum } => {
            let value = current_value
                .and_then(|v| v.as_i64())
                .or_else(|| schema.default.as_ref().and_then(|d| d.as_i64()))
                .unwrap_or(0);

            let mut state = NumberInputState::new(value, &schema.name);
            if let Some(min) = minimum {
                state = state.with_min(*min);
            }
            if let Some(max) = maximum {
                state = state.with_max(*max);
            }
            SettingControl::Number(state)
        }

        SettingType::Number { minimum, maximum } => {
            // For floats, we'll display as integers (multiply by 100 for percentages)
            let value = current_value
                .and_then(|v| v.as_f64())
                .or_else(|| schema.default.as_ref().and_then(|d| d.as_f64()))
                .unwrap_or(0.0);

            // Convert to integer representation
            let int_value = (value * 100.0).round() as i64;
            let mut state = NumberInputState::new(int_value, &schema.name);
            if let Some(min) = minimum {
                state = state.with_min((*min * 100.0) as i64);
            }
            if let Some(max) = maximum {
                state = state.with_max((*max * 100.0) as i64);
            }
            SettingControl::Number(state)
        }

        SettingType::String => {
            let value = current_value
                .and_then(|v| v.as_str())
                .or_else(|| schema.default.as_ref().and_then(|d| d.as_str()))
                .unwrap_or("");

            let state = TextInputState::new(&schema.name).with_value(value);
            SettingControl::Text(state)
        }

        SettingType::Enum { options } => {
            let current = current_value
                .and_then(|v| v.as_str())
                .or_else(|| schema.default.as_ref().and_then(|d| d.as_str()))
                .unwrap_or("");

            let display_names: Vec<String> = options.iter().map(|o| o.name.clone()).collect();
            let values: Vec<String> = options.iter().map(|o| o.value.clone()).collect();
            let selected = values.iter().position(|v| v == current).unwrap_or(0);
            let state =
                DropdownState::with_values(display_names, values, &schema.name).with_selected(selected);
            SettingControl::Dropdown(state)
        }

        SettingType::StringArray => {
            let items: Vec<String> = current_value
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .or_else(|| {
                    schema.default.as_ref().and_then(|d| {
                        d.as_array().map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                    })
                })
                .unwrap_or_default();

            let state = TextListState::new(&schema.name).with_items(items);
            SettingControl::TextList(state)
        }

        SettingType::Object { .. } => SettingControl::Complex {
            type_name: "Object".to_string(),
        },

        SettingType::Map { .. } => SettingControl::Complex {
            type_name: "Map".to_string(),
        },

        SettingType::Complex => SettingControl::Complex {
            type_name: "Complex".to_string(),
        },
    };

    // Check if modified from default
    let modified = match (&current_value, &schema.default) {
        (Some(current), Some(default)) => *current != default,
        (Some(_), None) => true,
        _ => false,
    };

    SettingItem {
        path: schema.path.clone(),
        name: schema.name.clone(),
        description: schema.description.clone(),
        control,
        default: schema.default.clone(),
        modified,
    }
}

/// Extract the current value from a control
pub fn control_to_value(control: &SettingControl) -> serde_json::Value {
    match control {
        SettingControl::Toggle(state) => serde_json::Value::Bool(state.checked),

        SettingControl::Number(state) => {
            // TODO: Handle float values properly (check schema for type)
            serde_json::Value::Number(state.value.into())
        }

        SettingControl::Dropdown(state) => {
            state
                .selected_value()
                .map(|s| serde_json::Value::String(s.to_string()))
                .unwrap_or(serde_json::Value::Null)
        }

        SettingControl::Text(state) => serde_json::Value::String(state.value.clone()),

        SettingControl::TextList(state) => {
            let arr: Vec<serde_json::Value> = state
                .items
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect();
            serde_json::Value::Array(arr)
        }

        SettingControl::Complex { .. } => serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> serde_json::Value {
        serde_json::json!({
            "theme": "monokai",
            "check_for_updates": false,
            "editor": {
                "tab_size": 2,
                "line_numbers": true
            }
        })
    }

    #[test]
    fn test_build_toggle_item() {
        let schema = SettingSchema {
            path: "/check_for_updates".to_string(),
            name: "Check For Updates".to_string(),
            description: Some("Check for updates".to_string()),
            setting_type: SettingType::Boolean,
            default: Some(serde_json::Value::Bool(true)),
        };

        let config = sample_config();
        let item = build_item(&schema, &config);

        assert_eq!(item.path, "/check_for_updates");
        assert!(item.modified); // false != true (default)

        if let SettingControl::Toggle(state) = &item.control {
            assert!(!state.checked); // Current value is false
        } else {
            panic!("Expected toggle control");
        }
    }

    #[test]
    fn test_build_number_item() {
        let schema = SettingSchema {
            path: "/editor/tab_size".to_string(),
            name: "Tab Size".to_string(),
            description: None,
            setting_type: SettingType::Integer {
                minimum: Some(1),
                maximum: Some(16),
            },
            default: Some(serde_json::Value::Number(4.into())),
        };

        let config = sample_config();
        let item = build_item(&schema, &config);

        assert!(item.modified); // 2 != 4 (default)

        if let SettingControl::Number(state) = &item.control {
            assert_eq!(state.value, 2);
            assert_eq!(state.min, Some(1));
            assert_eq!(state.max, Some(16));
        } else {
            panic!("Expected number control");
        }
    }

    #[test]
    fn test_build_text_item() {
        let schema = SettingSchema {
            path: "/theme".to_string(),
            name: "Theme".to_string(),
            description: None,
            setting_type: SettingType::String,
            default: Some(serde_json::Value::String("high-contrast".to_string())),
        };

        let config = sample_config();
        let item = build_item(&schema, &config);

        assert!(item.modified);

        if let SettingControl::Text(state) = &item.control {
            assert_eq!(state.value, "monokai");
        } else {
            panic!("Expected text control");
        }
    }

    #[test]
    fn test_control_to_value() {
        let toggle = SettingControl::Toggle(ToggleState::new(true, "Test"));
        assert_eq!(control_to_value(&toggle), serde_json::Value::Bool(true));

        let number = SettingControl::Number(NumberInputState::new(42, "Test"));
        assert_eq!(control_to_value(&number), serde_json::json!(42));

        let text = SettingControl::Text(TextInputState::new("Test").with_value("hello"));
        assert_eq!(control_to_value(&text), serde_json::Value::String("hello".to_string()));
    }
}
