//! Settings UI renderer
//!
//! Renders the settings modal with category navigation and setting controls.

use super::items::SettingControl;
use super::layout::{SettingsHit, SettingsLayout};
use super::search::SearchResult;
use super::state::SettingsState;
use crate::view::controls::{
    render_button, render_dropdown, render_number_input, render_text_input, render_toggle,
    ButtonColors, ButtonState, DropdownColors, MapColors, NumberInputColors, TextInputColors,
    TextListColors, ToggleColors,
};
use crate::view::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Render the settings modal
pub fn render_settings(
    frame: &mut Frame,
    area: Rect,
    state: &mut SettingsState,
    theme: &Theme,
) -> SettingsLayout {
    // Calculate modal size (80% of screen, max 100 wide, 40 tall)
    let modal_width = (area.width * 80 / 100).min(100);
    let modal_height = (area.height * 80 / 100).min(40);
    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    // Clear the modal area and draw border
    frame.render_widget(Clear, modal_area);

    let title = if state.has_changes() {
        " Settings • (modified) "
    } else {
        " Settings "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.popup_border_fg))
        .style(Style::default().bg(theme.popup_bg));
    frame.render_widget(block, modal_area);

    // Inner area after border
    let inner_area = Rect::new(
        modal_area.x + 1,
        modal_area.y + 1,
        modal_area.width.saturating_sub(2),
        modal_area.height.saturating_sub(2),
    );

    // Render search header if search is active
    let (search_header_height, content_area) = if state.search_active {
        let search_area = Rect::new(inner_area.x, inner_area.y, inner_area.width, 2);
        render_search_header(frame, search_area, state, theme);
        (
            2,
            Rect::new(
                inner_area.x,
                inner_area.y + 2,
                inner_area.width,
                inner_area.height.saturating_sub(2),
            ),
        )
    } else {
        (0, inner_area)
    };
    let _ = search_header_height; // suppress unused warning

    // Layout: [left panel (categories)] | [right panel (settings)]
    let chunks = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Min(40),
    ])
    .split(content_area);

    let categories_area = chunks[0];
    let settings_area = chunks[1];

    // Create layout tracker
    let mut layout = SettingsLayout::new(modal_area);

    // Render category list (left panel)
    render_categories(frame, categories_area, state, theme, &mut layout);

    // Render separator
    let separator_area = Rect::new(
        categories_area.x + categories_area.width,
        categories_area.y,
        1,
        categories_area.height,
    );
    render_separator(frame, separator_area, theme);

    // Render settings (right panel) or search results
    let settings_inner = Rect::new(
        settings_area.x + 1,
        settings_area.y,
        settings_area.width.saturating_sub(1),
        settings_area.height,
    );

    if state.search_active && !state.search_results.is_empty() {
        render_search_results(frame, settings_inner, state, theme, &mut layout);
    } else {
        render_settings_panel(frame, settings_inner, state, theme, &mut layout);
    }

    // Render footer with buttons
    render_footer(frame, modal_area, state, theme, &mut layout);

    // Render confirmation dialog if showing
    if state.showing_confirm_dialog {
        render_confirm_dialog(frame, modal_area, state, theme);
    }

    // Render help overlay if showing
    if state.showing_help {
        render_help_overlay(frame, modal_area, theme);
    }

    layout
}

/// Render the category list
fn render_categories(
    frame: &mut Frame,
    area: Rect,
    state: &SettingsState,
    theme: &Theme,
    layout: &mut SettingsLayout,
) {
    use super::layout::SettingsHit;

    for (idx, page) in state.pages.iter().enumerate() {
        if idx as u16 >= area.height {
            break;
        }

        let is_selected = idx == state.selected_category;
        let is_hovered = matches!(state.hover_hit, Some(SettingsHit::Category(i)) if i == idx);
        let row_area = Rect::new(area.x, area.y + idx as u16, area.width, 1);

        layout.add_category(idx, row_area);

        let style = if is_selected {
            if state.category_focus {
                Style::default()
                    .fg(theme.menu_highlight_fg)
                    .bg(theme.menu_highlight_bg)
            } else {
                Style::default()
                    .fg(theme.menu_fg)
                    .bg(theme.selection_bg)
            }
        } else if is_hovered {
            // Hover highlight using menu hover colors
            Style::default()
                .fg(theme.menu_hover_fg)
                .bg(theme.menu_hover_bg)
        } else {
            Style::default().fg(theme.popup_text_fg)
        };

        // Indicator for categories with modified settings
        let has_changes = page.items.iter().any(|i| i.modified);
        let prefix = if has_changes { "● " } else { "  " };

        let text = format!("{}{}", prefix, page.name);
        let line = Line::from(Span::styled(text, style));
        frame.render_widget(Paragraph::new(line), row_area);
    }
}

/// Render vertical separator
fn render_separator(frame: &mut Frame, area: Rect, theme: &Theme) {
    for y in 0..area.height {
        let cell = Rect::new(area.x, area.y + y, 1, 1);
        let sep = Paragraph::new("│").style(Style::default().fg(theme.split_separator_fg));
        frame.render_widget(sep, cell);
    }
}

/// Context for rendering a setting item (extracted to avoid borrow issues)
struct RenderContext {
    selected_item: usize,
    category_focus: bool,
    hover_hit: Option<SettingsHit>,
}

/// Render the settings panel for the current category
fn render_settings_panel(
    frame: &mut Frame,
    area: Rect,
    state: &mut SettingsState,
    theme: &Theme,
    layout: &mut SettingsLayout,
) {
    let page = match state.current_page() {
        Some(p) => p,
        None => return,
    };

    // Render page title and description
    let mut y = area.y;
    let header_start_y = y;

    // Page title
    let title_style = Style::default()
        .fg(theme.menu_active_fg)
        .add_modifier(Modifier::BOLD);
    let title = Line::from(Span::styled(&page.name, title_style));
    frame.render_widget(Paragraph::new(title), Rect::new(area.x, y, area.width, 1));
    y += 1;

    // Page description
    if let Some(ref desc) = page.description {
        let desc_style = Style::default().fg(theme.line_number_fg);
        let desc_line = Line::from(Span::styled(desc, desc_style));
        frame.render_widget(
            Paragraph::new(desc_line),
            Rect::new(area.x, y, area.width, 1),
        );
        y += 1;
    }

    y += 1; // Blank line

    let header_height = (y - header_start_y) as usize;
    let items_start_y = y;

    // Calculate available height for items
    let available_height = area.height.saturating_sub(header_height as u16 + 1);

    // Update scroll panel with current viewport and content
    let page = state.pages.get(state.selected_category).unwrap();
    state.scroll_panel.set_viewport(available_height);
    state.scroll_panel.update_content_height(&page.items);

    // Extract state needed for rendering (to avoid borrow issues with scroll_panel)
    let render_ctx = RenderContext {
        selected_item: state.selected_item,
        category_focus: state.category_focus,
        hover_hit: state.hover_hit.clone(),
    };

    // Area for items (below header)
    let items_area = Rect::new(area.x, items_start_y, area.width, available_height.max(1));

    // Get items reference for rendering
    let page = state.pages.get(state.selected_category).unwrap();

    // Use ScrollablePanel to render items with automatic scroll handling
    let panel_layout = state.scroll_panel.render(
        frame,
        items_area,
        &page.items,
        |frame, info, item| {
            render_setting_item_pure(frame, info.area, item, info.index, info.skip_top, &render_ctx, theme)
        },
        theme,
    );

    // Transfer item layouts to SettingsLayout
    let page = state.pages.get(state.selected_category).unwrap();
    for item_info in panel_layout.item_layouts {
        layout.add_item(
            item_info.index,
            page.items[item_info.index].path.clone(),
            item_info.area,
            item_info.layout,
        );
    }

    // Track the settings panel area for scroll hit testing
    layout.settings_panel_area = Some(panel_layout.content_area);

    // Track scrollbar area for drag detection
    if let Some(sb_area) = panel_layout.scrollbar_area {
        layout.scrollbar_area = Some(sb_area);
    }
}

/// Pure render function for a setting item (returns layout, doesn't modify external state)
///
/// # Arguments
/// * `skip_top` - Number of rows to skip at top of item (for partial visibility when scrolling)
fn render_setting_item_pure(
    frame: &mut Frame,
    area: Rect,
    item: &super::items::SettingItem,
    idx: usize,
    skip_top: u16,
    ctx: &RenderContext,
    theme: &Theme,
) -> ControlLayoutInfo {
    let is_selected = !ctx.category_focus && idx == ctx.selected_item;

    // Check if this item or any of its controls is being hovered
    let is_item_hovered = match ctx.hover_hit {
        Some(SettingsHit::Item(i)) => i == idx,
        Some(SettingsHit::ControlToggle(i)) => i == idx,
        Some(SettingsHit::ControlDecrement(i)) => i == idx,
        Some(SettingsHit::ControlIncrement(i)) => i == idx,
        Some(SettingsHit::ControlDropdown(i)) => i == idx,
        Some(SettingsHit::ControlText(i)) => i == idx,
        Some(SettingsHit::ControlTextListRow(i, _)) => i == idx,
        Some(SettingsHit::ControlMapRow(i, _)) => i == idx,
        _ => false,
    };

    // Draw selection or hover highlight background (for visible portion)
    if is_selected || is_item_hovered {
        let bg_style = if is_selected {
            Style::default().bg(theme.current_line_bg)
        } else {
            Style::default().bg(theme.menu_hover_bg)
        };
        for row in 0..area.height {
            let row_area = Rect::new(area.x, area.y + row, area.width, 1);
            frame.render_widget(Paragraph::new("").style(bg_style), row_area);
        }
    }

    // All controls render their own label, so just render the control
    render_control(frame, area, &item.control, &item.name, item.modified, skip_top, theme)
}

/// Render the appropriate control for a setting
///
/// # Arguments
/// * `name` - Setting name (for controls that render their own label)
/// * `modified` - Whether the setting has been modified from default
/// * `skip_rows` - Number of rows to skip at top of control (for partial visibility)
fn render_control(
    frame: &mut Frame,
    area: Rect,
    control: &SettingControl,
    name: &str,
    modified: bool,
    skip_rows: u16,
    theme: &Theme,
) -> ControlLayoutInfo {
    match control {
        // Single-row controls: only render if not skipped
        SettingControl::Toggle(state) => {
            if skip_rows > 0 {
                return ControlLayoutInfo::Toggle(Rect::default());
            }
            let colors = ToggleColors::from_theme(theme);
            let toggle_layout = render_toggle(frame, area, state, &colors);
            ControlLayoutInfo::Toggle(toggle_layout.full_area)
        }

        SettingControl::Number(state) => {
            if skip_rows > 0 {
                return ControlLayoutInfo::Number {
                    decrement: Rect::default(),
                    increment: Rect::default(),
                    value: Rect::default(),
                };
            }
            let colors = NumberInputColors::from_theme(theme);
            let num_layout = render_number_input(frame, area, state, &colors);
            ControlLayoutInfo::Number {
                decrement: num_layout.decrement_area,
                increment: num_layout.increment_area,
                value: num_layout.value_area,
            }
        }

        SettingControl::Dropdown(state) => {
            if skip_rows > 0 {
                return ControlLayoutInfo::Dropdown(Rect::default());
            }
            let colors = DropdownColors::from_theme(theme);
            let drop_layout = render_dropdown(frame, area, state, &colors);
            ControlLayoutInfo::Dropdown(drop_layout.button_area)
        }

        SettingControl::Text(state) => {
            if skip_rows > 0 {
                return ControlLayoutInfo::Text(Rect::default());
            }
            let colors = TextInputColors::from_theme(theme);
            let text_layout = render_text_input(frame, area, state, &colors, 30);
            ControlLayoutInfo::Text(text_layout.input_area)
        }

        // Multi-row controls: pass skip_rows to render partial view
        SettingControl::TextList(state) => {
            let colors = TextListColors::from_theme(theme);
            let list_layout = render_text_list_partial(frame, area, state, &colors, 30, skip_rows);
            ControlLayoutInfo::TextList {
                rows: list_layout.rows.iter().map(|r| r.text_area).collect(),
            }
        }

        SettingControl::Map(state) => {
            let colors = MapColors::from_theme(theme);
            let map_layout = render_map_partial(frame, area, state, &colors, 20, skip_rows);
            ControlLayoutInfo::Map {
                entry_rows: map_layout.entry_areas.iter().map(|e| e.row_area).collect(),
            }
        }

        SettingControl::Complex { type_name } => {
            if skip_rows > 0 {
                return ControlLayoutInfo::Complex;
            }
            // Render label with modified indicator
            let label_style = Style::default().fg(theme.editor_fg);
            let value_style = Style::default().fg(theme.line_number_fg);
            let modified_indicator = if modified { "• " } else { "" };

            let label = Span::styled(format!("{}{}: ", modified_indicator, name), label_style);
            let value = Span::styled(format!("<{} - edit in config.toml>", type_name), value_style);

            frame.render_widget(Paragraph::new(Line::from(vec![label, value])), area);
            ControlLayoutInfo::Complex
        }
    }
}

/// Render TextList with partial visibility (skipping top rows)
fn render_text_list_partial(
    frame: &mut Frame,
    area: Rect,
    state: &crate::view::controls::TextListState,
    colors: &TextListColors,
    field_width: u16,
    skip_rows: u16,
) -> crate::view::controls::TextListLayout {
    use crate::view::controls::text_list::{TextListLayout, TextListRowLayout};
    use crate::view::controls::FocusState;

    let empty_layout = TextListLayout {
        rows: Vec::new(),
        full_area: area,
    };

    if area.height == 0 || area.width < 10 {
        return empty_layout;
    }

    let label_color = match state.focus {
        FocusState::Focused => colors.focused,
        FocusState::Hovered => colors.focused,
        FocusState::Disabled => colors.disabled,
        FocusState::Normal => colors.label,
    };

    let mut rows = Vec::new();
    let mut y = area.y;
    let mut content_row = 0u16; // Which row of content we're at

    // Row 0 is label
    if skip_rows == 0 {
        let label_line = Line::from(vec![
            Span::styled(&state.label, Style::default().fg(label_color)),
            Span::raw(":"),
        ]);
        frame.render_widget(Paragraph::new(label_line), Rect::new(area.x, y, area.width, 1));
        y += 1;
    }
    content_row += 1;

    let indent = 2u16;
    let actual_field_width = field_width.min(area.width.saturating_sub(indent + 5));

    // Render existing items (rows 1 to N)
    for (idx, item) in state.items.iter().enumerate() {
        if y >= area.y + area.height {
            break;
        }

        // Skip rows before skip_rows
        if content_row < skip_rows {
            content_row += 1;
            continue;
        }

        let is_focused = state.focused_item == Some(idx) && state.focus == FocusState::Focused;
        let (border_color, text_color) = if is_focused {
            (colors.focused, colors.text)
        } else if state.focus == FocusState::Disabled {
            (colors.disabled, colors.disabled)
        } else {
            (colors.border, colors.text)
        };

        let inner_width = actual_field_width.saturating_sub(2) as usize;
        let visible: String = item.chars().take(inner_width).collect();
        let padded = format!("{:width$}", visible, width = inner_width);

        let line = Line::from(vec![
            Span::raw(" ".repeat(indent as usize)),
            Span::styled("[", Style::default().fg(border_color)),
            Span::styled(padded, Style::default().fg(text_color)),
            Span::styled("]", Style::default().fg(border_color)),
            Span::raw(" "),
            Span::styled("[x]", Style::default().fg(colors.remove_button)),
        ]);

        let row_area = Rect::new(area.x, y, area.width, 1);
        frame.render_widget(Paragraph::new(line), row_area);

        let text_area = Rect::new(area.x + indent, y, actual_field_width, 1);
        let button_area = Rect::new(area.x + indent + actual_field_width + 1, y, 3, 1);
        rows.push(TextListRowLayout {
            text_area,
            button_area,
            index: Some(idx),
        });

        y += 1;
        content_row += 1;
    }

    // Add-new row
    if y < area.y + area.height && content_row >= skip_rows {
        let add_line = Line::from(vec![
            Span::raw(" ".repeat(indent as usize)),
            Span::styled("[+] Add new", Style::default().fg(colors.add_button)),
        ]);
        let row_area = Rect::new(area.x, y, area.width, 1);
        frame.render_widget(Paragraph::new(add_line), row_area);

        rows.push(TextListRowLayout {
            text_area: Rect::new(area.x + indent, y, 11, 1), // "[+] Add new"
            button_area: Rect::new(area.x + indent, y, 11, 1),
            index: None,
        });
    }

    TextListLayout {
        rows,
        full_area: area,
    }
}

/// Render Map with partial visibility (skipping top rows)
fn render_map_partial(
    frame: &mut Frame,
    area: Rect,
    state: &crate::view::controls::MapState,
    colors: &MapColors,
    key_width: u16,
    skip_rows: u16,
) -> crate::view::controls::MapLayout {
    use crate::view::controls::map_input::{MapEntryLayout, MapLayout};
    use crate::view::controls::FocusState;

    let empty_layout = MapLayout {
        entry_areas: Vec::new(),
        add_row_area: None,
        full_area: area,
    };

    if area.height == 0 || area.width < 15 {
        return empty_layout;
    }

    let label_color = match state.focus {
        FocusState::Focused => colors.focused,
        FocusState::Hovered => colors.focused,
        FocusState::Disabled => colors.disabled,
        FocusState::Normal => colors.label,
    };

    let mut entry_areas = Vec::new();
    let mut y = area.y;
    let mut content_row = 0u16;

    // Row 0 is label
    if skip_rows == 0 {
        let label_line = Line::from(vec![
            Span::styled(&state.label, Style::default().fg(label_color)),
            Span::raw(":"),
        ]);
        frame.render_widget(Paragraph::new(label_line), Rect::new(area.x, y, area.width, 1));
        y += 1;
    }
    content_row += 1;

    let indent = 2u16;

    // Render entries
    for (idx, (key, _value)) in state.entries.iter().enumerate() {
        if y >= area.y + area.height {
            break;
        }

        if content_row < skip_rows {
            content_row += 1;
            continue;
        }

        let is_focused = state.focused_entry == Some(idx) && state.focus == FocusState::Focused;
        let key_color = if is_focused {
            colors.focused
        } else if state.focus == FocusState::Disabled {
            colors.disabled
        } else {
            colors.key
        };

        let display_key: String = key.chars().take(key_width as usize).collect();
        let line = Line::from(vec![
            Span::raw(" ".repeat(indent as usize)),
            Span::styled(format!("{:width$}", display_key, width = key_width as usize), Style::default().fg(key_color)),
            Span::raw(" "),
            Span::styled("[x]", Style::default().fg(colors.remove_button)),
        ]);

        let row_area = Rect::new(area.x, y, area.width, 1);
        frame.render_widget(Paragraph::new(line), row_area);

        entry_areas.push(MapEntryLayout {
            index: idx,
            row_area,
            expand_area: Rect::default(), // Not rendering expand button in partial view
            key_area: Rect::new(area.x + indent, y, key_width, 1),
            remove_area: Rect::new(area.x + indent + key_width + 1, y, 3, 1),
        });

        y += 1;
        content_row += 1;
    }

    // Add-new row
    let add_row_area = if y < area.y + area.height && content_row >= skip_rows {
        let add_line = Line::from(vec![
            Span::raw(" ".repeat(indent as usize)),
            Span::styled("[+] Add new", Style::default().fg(colors.add_button)),
        ]);
        let row_area = Rect::new(area.x, y, area.width, 1);
        frame.render_widget(Paragraph::new(add_line), row_area);
        Some(row_area)
    } else {
        None
    };

    MapLayout {
        entry_areas,
        add_row_area,
        full_area: area,
    }
}

/// Layout info for a control (for hit testing)
#[derive(Debug, Clone)]
pub enum ControlLayoutInfo {
    Toggle(Rect),
    Number {
        decrement: Rect,
        increment: Rect,
        value: Rect,
    },
    Dropdown(Rect),
    Text(Rect),
    TextList { rows: Vec<Rect> },
    Map { entry_rows: Vec<Rect> },
    Complex,
}

/// Render footer with action buttons
fn render_footer(
    frame: &mut Frame,
    modal_area: Rect,
    state: &SettingsState,
    theme: &Theme,
    layout: &mut SettingsLayout,
) {
    use super::layout::SettingsHit;
    use crate::view::controls::FocusState;

    let footer_y = modal_area.y + modal_area.height - 2;
    let footer_area = Rect::new(
        modal_area.x + 1,
        footer_y,
        modal_area.width.saturating_sub(2),
        1,
    );

    // Draw separator line
    let sep_area = Rect::new(modal_area.x + 1, footer_y - 1, modal_area.width.saturating_sub(2), 1);
    let sep_line: String = "─".repeat(sep_area.width as usize);
    frame.render_widget(
        Paragraph::new(sep_line).style(Style::default().fg(theme.split_separator_fg)),
        sep_area,
    );

    // Buttons on the right side
    let button_colors = ButtonColors::from_theme(theme);

    // Determine hover states for buttons
    let save_hovered = matches!(state.hover_hit, Some(SettingsHit::SaveButton));
    let cancel_hovered = matches!(state.hover_hit, Some(SettingsHit::CancelButton));
    let reset_hovered = matches!(state.hover_hit, Some(SettingsHit::ResetButton));

    let save_state = ButtonState::new("Save")
        .with_focus(if save_hovered { FocusState::Hovered } else { FocusState::Normal });
    let cancel_state = ButtonState::new("Cancel")
        .with_focus(if cancel_hovered { FocusState::Hovered } else { FocusState::Normal });
    let reset_state = ButtonState::new("Reset")
        .with_focus(if reset_hovered { FocusState::Hovered } else { FocusState::Normal });

    // Calculate button positions from right
    let cancel_width = 10; // "[ Cancel ]"
    let save_width = 8;    // "[ Save ]"
    let reset_width = 9;   // "[ Reset ]"
    let gap = 2;

    let cancel_x = footer_area.x + footer_area.width - cancel_width;
    let save_x = cancel_x - save_width - gap;
    let reset_x = save_x - reset_width - gap;

    // Render buttons
    let reset_area = Rect::new(reset_x, footer_y, reset_width, 1);
    let reset_layout = render_button(frame, reset_area, &reset_state, &button_colors);
    layout.reset_button = Some(reset_layout.button_area);

    let save_area = Rect::new(save_x, footer_y, save_width, 1);
    let save_layout = render_button(frame, save_area, &save_state, &button_colors);
    layout.save_button = Some(save_layout.button_area);

    let cancel_area = Rect::new(cancel_x, footer_y, cancel_width, 1);
    let cancel_layout = render_button(frame, cancel_area, &cancel_state, &button_colors);
    layout.cancel_button = Some(cancel_layout.button_area);

    // Help text on the left
    let help = if state.search_active {
        "Type to search, ↑↓:Navigate  Enter:Jump  Esc:Cancel"
    } else {
        "↑↓:Navigate  Tab:Switch panel  Enter:Edit  /:Search  Esc:Close"
    };
    let help_style = Style::default().fg(theme.line_number_fg);
    frame.render_widget(
        Paragraph::new(help).style(help_style),
        Rect::new(footer_area.x, footer_y, reset_x - footer_area.x - 1, 1),
    );
}

/// Render the search header with query input
fn render_search_header(
    frame: &mut Frame,
    area: Rect,
    state: &SettingsState,
    theme: &Theme,
) {
    // First line: Search input
    let search_style = Style::default().fg(theme.popup_text_fg);
    let cursor_style = Style::default()
        .fg(theme.menu_highlight_fg)
        .add_modifier(Modifier::UNDERLINED);

    let spans = vec![
        Span::styled("🔍 ", search_style),
        Span::styled(&state.search_query, search_style),
        Span::styled("█", cursor_style), // Cursor
    ];
    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), Rect::new(area.x, area.y, area.width, 1));

    // Second line: Result count
    let result_count = state.search_results.len();
    let count_text = if result_count == 0 {
        if state.search_query.is_empty() {
            String::new()
        } else {
            "No results found".to_string()
        }
    } else if result_count == 1 {
        "1 result".to_string()
    } else {
        format!("{} results", result_count)
    };

    let count_style = Style::default().fg(theme.line_number_fg);
    frame.render_widget(
        Paragraph::new(count_text).style(count_style),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );
}

/// Render search results with breadcrumbs
fn render_search_results(
    frame: &mut Frame,
    area: Rect,
    state: &SettingsState,
    theme: &Theme,
    layout: &mut SettingsLayout,
) {
    let mut y = area.y;

    for (idx, result) in state.search_results.iter().enumerate() {
        if y >= area.y + area.height.saturating_sub(3) {
            break;
        }

        let is_selected = idx == state.selected_search_result;
        let item_area = Rect::new(area.x, y, area.width, 3);

        render_search_result_item(frame, item_area, result, is_selected, theme, layout);
        y += 3;
    }
}

/// Render a single search result with breadcrumb
fn render_search_result_item(
    frame: &mut Frame,
    area: Rect,
    result: &SearchResult,
    is_selected: bool,
    theme: &Theme,
    layout: &mut SettingsLayout,
) {
    // Draw selection highlight background
    if is_selected {
        let bg_style = Style::default().bg(theme.current_line_bg);
        for row in 0..area.height.min(3) {
            let row_area = Rect::new(area.x, area.y + row, area.width, 1);
            frame.render_widget(Paragraph::new("").style(bg_style), row_area);
        }
    }

    // First line: Setting name with highlighting
    let name_style = if is_selected {
        Style::default().fg(theme.menu_highlight_fg)
    } else {
        Style::default().fg(theme.popup_text_fg)
    };

    // Build name with match highlighting
    let name_line = build_highlighted_text(
        &result.item.name,
        &result.name_matches,
        name_style,
        Style::default()
            .fg(theme.diagnostic_warning_fg)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(
        Paragraph::new(name_line),
        Rect::new(area.x, area.y, area.width, 1),
    );

    // Second line: Breadcrumb
    let breadcrumb_style = Style::default()
        .fg(theme.line_number_fg)
        .add_modifier(Modifier::ITALIC);
    let breadcrumb = format!("  {} > {}", result.breadcrumb, result.item.path);
    let breadcrumb_line = Line::from(Span::styled(breadcrumb, breadcrumb_style));
    frame.render_widget(
        Paragraph::new(breadcrumb_line),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );

    // Third line: Description (if any)
    if let Some(ref desc) = result.item.description {
        let desc_style = Style::default().fg(theme.line_number_fg);
        let truncated_desc = if desc.len() > area.width as usize - 2 {
            format!("  {}...", &desc[..area.width as usize - 5])
        } else {
            format!("  {}", desc)
        };
        frame.render_widget(
            Paragraph::new(truncated_desc).style(desc_style),
            Rect::new(area.x, area.y + 2, area.width, 1),
        );
    }

    // Track this item in layout
    layout.add_search_result(result.page_index, result.item_index, area);
}

/// Build a line with highlighted match positions
fn build_highlighted_text(
    text: &str,
    matches: &[usize],
    normal_style: Style,
    highlight_style: Style,
) -> Line<'static> {
    if matches.is_empty() {
        return Line::from(Span::styled(text.to_string(), normal_style));
    }

    let chars: Vec<char> = text.chars().collect();
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut in_highlight = false;

    for (idx, ch) in chars.iter().enumerate() {
        let should_highlight = matches.contains(&idx);

        if should_highlight != in_highlight {
            if !current.is_empty() {
                let style = if in_highlight {
                    highlight_style
                } else {
                    normal_style
                };
                spans.push(Span::styled(current, style));
                current = String::new();
            }
            in_highlight = should_highlight;
        }

        current.push(*ch);
    }

    // Push remaining
    if !current.is_empty() {
        let style = if in_highlight {
            highlight_style
        } else {
            normal_style
        };
        spans.push(Span::styled(current, style));
    }

    Line::from(spans)
}

/// Render the unsaved changes confirmation dialog
fn render_confirm_dialog(
    frame: &mut Frame,
    parent_area: Rect,
    state: &SettingsState,
    theme: &Theme,
) {
    // Calculate dialog size
    let changes = state.get_change_descriptions();
    let dialog_width = 50.min(parent_area.width.saturating_sub(4));
    // Base height: 2 borders + 2 prompt lines + 1 separator + 1 buttons + 1 help = 7
    // Plus one line per change
    let dialog_height = (7 + changes.len() as u16).min(20).min(parent_area.height.saturating_sub(4));

    // Center the dialog
    let dialog_x = parent_area.x + (parent_area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = parent_area.y + (parent_area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

    // Clear and draw border
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Unsaved Changes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.diagnostic_warning_fg))
        .style(Style::default().bg(theme.popup_bg));
    frame.render_widget(block, dialog_area);

    // Inner area
    let inner = Rect::new(
        dialog_area.x + 2,
        dialog_area.y + 1,
        dialog_area.width.saturating_sub(4),
        dialog_area.height.saturating_sub(2),
    );

    let mut y = inner.y;

    // Prompt text
    let prompt = "You have unsaved changes:";
    let prompt_style = Style::default().fg(theme.popup_text_fg);
    frame.render_widget(
        Paragraph::new(prompt).style(prompt_style),
        Rect::new(inner.x, y, inner.width, 1),
    );
    y += 2;

    // List changes
    let change_style = Style::default().fg(theme.popup_text_fg);
    for change in changes.iter().take((dialog_height as usize).saturating_sub(7)) {
        let truncated = if change.len() > inner.width as usize - 2 {
            format!("• {}...", &change[..inner.width as usize - 5])
        } else {
            format!("• {}", change)
        };
        frame.render_widget(
            Paragraph::new(truncated).style(change_style),
            Rect::new(inner.x, y, inner.width, 1),
        );
        y += 1;
    }

    // Skip to button row
    let button_y = dialog_area.y + dialog_area.height - 3;

    // Draw separator
    let sep_line: String = "─".repeat(inner.width as usize);
    frame.render_widget(
        Paragraph::new(sep_line).style(Style::default().fg(theme.split_separator_fg)),
        Rect::new(inner.x, button_y - 1, inner.width, 1),
    );

    // Render the three options
    let options = ["Save and Exit", "Discard", "Cancel"];
    let total_width: u16 = options.iter().map(|o| o.len() as u16 + 4).sum::<u16>() + 4; // +4 for gaps
    let mut x = inner.x + (inner.width.saturating_sub(total_width)) / 2;

    for (idx, label) in options.iter().enumerate() {
        let is_selected = idx == state.confirm_dialog_selection;
        let button_width = label.len() as u16 + 4;

        let style = if is_selected {
            Style::default()
                .fg(theme.menu_highlight_fg)
                .bg(theme.menu_highlight_bg)
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(theme.popup_text_fg)
        };

        let text = if is_selected {
            format!("▶[ {} ]", label)
        } else {
            format!(" [ {} ]", label)
        };
        frame.render_widget(
            Paragraph::new(text).style(style),
            Rect::new(x, button_y, button_width + 1, 1),
        );

        x += button_width + 3;
    }

    // Help text
    let help = "←/→: Select   Enter: Confirm   Esc: Cancel";
    let help_style = Style::default().fg(theme.line_number_fg);
    frame.render_widget(
        Paragraph::new(help).style(help_style),
        Rect::new(inner.x, button_y + 1, inner.width, 1),
    );
}

/// Render the help overlay showing keyboard shortcuts
fn render_help_overlay(
    frame: &mut Frame,
    parent_area: Rect,
    theme: &Theme,
) {
    // Define the help content
    let help_items = [
        ("Navigation", vec![
            ("↑ / ↓", "Move up/down"),
            ("Tab", "Switch between categories and settings"),
            ("Enter", "Activate/toggle setting"),
        ]),
        ("Search", vec![
            ("/", "Start search"),
            ("Esc", "Cancel search"),
            ("↑ / ↓", "Navigate results"),
            ("Enter", "Jump to result"),
        ]),
        ("Actions", vec![
            ("Ctrl+S", "Save settings"),
            ("Ctrl+R", "Reset to default"),
            ("Esc", "Close settings"),
            ("?", "Toggle this help"),
        ]),
    ];

    // Calculate dialog size
    let dialog_width = 50.min(parent_area.width.saturating_sub(4));
    let dialog_height = 20.min(parent_area.height.saturating_sub(4));

    // Center the dialog
    let dialog_x = parent_area.x + (parent_area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = parent_area.y + (parent_area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

    // Clear and draw border
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.menu_highlight_fg))
        .style(Style::default().bg(theme.popup_bg));
    frame.render_widget(block, dialog_area);

    // Inner area
    let inner = Rect::new(
        dialog_area.x + 2,
        dialog_area.y + 1,
        dialog_area.width.saturating_sub(4),
        dialog_area.height.saturating_sub(2),
    );

    let mut y = inner.y;

    for (section_name, bindings) in &help_items {
        if y >= inner.y + inner.height.saturating_sub(1) {
            break;
        }

        // Section header
        let header_style = Style::default()
            .fg(theme.menu_active_fg)
            .add_modifier(Modifier::BOLD);
        frame.render_widget(
            Paragraph::new(*section_name).style(header_style),
            Rect::new(inner.x, y, inner.width, 1),
        );
        y += 1;

        for (key, description) in bindings {
            if y >= inner.y + inner.height.saturating_sub(1) {
                break;
            }

            let key_style = Style::default()
                .fg(theme.diagnostic_info_fg)
                .add_modifier(Modifier::BOLD);
            let desc_style = Style::default().fg(theme.popup_text_fg);

            let line = Line::from(vec![
                Span::styled(format!("  {:12}", key), key_style),
                Span::styled(*description, desc_style),
            ]);
            frame.render_widget(
                Paragraph::new(line),
                Rect::new(inner.x, y, inner.width, 1),
            );
            y += 1;
        }

        y += 1; // Blank line between sections
    }

    // Footer hint
    let footer_y = dialog_area.y + dialog_area.height - 2;
    let footer = "Press ? or Esc or Enter to close";
    let footer_style = Style::default().fg(theme.line_number_fg);
    let centered_x = inner.x + (inner.width.saturating_sub(footer.len() as u16)) / 2;
    frame.render_widget(
        Paragraph::new(footer).style(footer_style),
        Rect::new(centered_x, footer_y, footer.len() as u16, 1),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic compile test - actual rendering tests would need a test backend
    #[test]
    fn test_control_layout_info() {
        let toggle = ControlLayoutInfo::Toggle(Rect::new(0, 0, 10, 1));
        assert!(matches!(toggle, ControlLayoutInfo::Toggle(_)));

        let number = ControlLayoutInfo::Number {
            decrement: Rect::new(0, 0, 3, 1),
            increment: Rect::new(4, 0, 3, 1),
            value: Rect::new(8, 0, 5, 1),
        };
        assert!(matches!(number, ControlLayoutInfo::Number { .. }));
    }
}
