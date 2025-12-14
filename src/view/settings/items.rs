//! Setting items for the UI
//!
//! Converts schema information into renderable setting items.

use super::schema::{SettingCategory, SettingSchema, SettingType};
use crate::view::controls::{
    DropdownState, MapState, NumberInputState, TextInputState, TextListState, ToggleState,
};
use crate::view::ui::{FocusRegion, ScrollItem};

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
    /// Map/dictionary control for key-value pairs
    Map(MapState),
    /// Complex settings that can't be edited inline
    Complex { type_name: String },
}

impl SettingControl {
    /// Calculate the height needed for this control (in lines)
    pub fn control_height(&self) -> u16 {
        match self {
            // TextList needs: 1 label line + items + 1 "add new" row
            SettingControl::TextList(state) => {
                // 1 for label + items count + 1 for add-new row
                (state.items.len() + 2) as u16
            }
            // Map needs: 1 label + entries + expanded content + 1 add-new row
            SettingControl::Map(state) => {
                let base = 1 + state.entries.len() + 1; // label + entries + add-new
                // Add extra height for expanded entries (up to 6 lines each)
                let expanded_height: usize = state
                    .expanded
                    .iter()
                    .filter_map(|&idx| state.entries.get(idx))
                    .map(|(_, v)| {
                        if let Some(obj) = v.as_object() {
                            obj.len().min(5) + if obj.len() > 5 { 1 } else { 0 }
                        } else {
                            0
                        }
                    })
                    .sum();
                (base + expanded_height) as u16
            }
            // All other controls fit in 1 line
            _ => 1,
        }
    }
}

impl SettingItem {
    /// Calculate the total height needed for this item (control + spacing)
    pub fn item_height(&self) -> u16 {
        // All controls render their own label, so height is just control + spacing
        self.control.control_height() + 1
    }
}

impl ScrollItem for SettingItem {
    fn height(&self) -> u16 {
        self.item_height()
    }

    fn focus_regions(&self) -> Vec<FocusRegion> {
        match &self.control {
            // TextList: each row is a focus region
            SettingControl::TextList(state) => {
                let mut regions = Vec::new();
                // Label row
                regions.push(FocusRegion {
                    id: 0,
                    y_offset: 0,
                    height: 1,
                });
                // Each item row (id = 1 + row_index)
                for i in 0..state.items.len() {
                    regions.push(FocusRegion {
                        id: 1 + i,
                        y_offset: 1 + i as u16,
                        height: 1,
                    });
                }
                // Add-new row
                regions.push(FocusRegion {
                    id: 1 + state.items.len(),
                    y_offset: 1 + state.items.len() as u16,
                    height: 1,
                });
                regions
            }
            // Map: each entry row is a focus region
            SettingControl::Map(state) => {
                let mut regions = Vec::new();
                let mut y = 0u16;

                // Label row
                regions.push(FocusRegion {
                    id: 0,
                    y_offset: y,
                    height: 1,
                });
                y += 1;

                // Each entry (id = 1 + entry_index)
                for (i, (_, v)) in state.entries.iter().enumerate() {
                    let mut entry_height = 1u16;
                    // Add expanded content height if expanded
                    if state.expanded.contains(&i) {
                        if let Some(obj) = v.as_object() {
                            entry_height += obj.len().min(5) as u16;
                            if obj.len() > 5 {
                                entry_height += 1;
                            }
                        }
                    }
                    regions.push(FocusRegion {
                        id: 1 + i,
                        y_offset: y,
                        height: entry_height,
                    });
                    y += entry_height;
                }

                // Add-new row
                regions.push(FocusRegion {
                    id: 1 + state.entries.len(),
                    y_offset: y,
                    height: 1,
                });
                regions
            }
            // Other controls: single region covering the whole item
            _ => {
                vec![FocusRegion {
                    id: 0,
                    y_offset: 0,
                    height: self.item_height().saturating_sub(1), // Exclude spacing
                }]
            }
        }
    }
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

        SettingType::Map { value_schema } => {
            // Get current map value or default
            let map_value = current_value
                .cloned()
                .or_else(|| schema.default.clone())
                .unwrap_or_else(|| serde_json::json!({}));

            let mut state = MapState::new(&schema.name).with_entries(&map_value);
            state = state.with_value_schema((**value_schema).clone());
            SettingControl::Map(state)
        }

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

        SettingControl::Map(state) => state.to_value(),

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
