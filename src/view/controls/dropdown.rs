//! Dropdown selection control
//!
//! Renders as: `Label: [Selected Option ▼]`

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::FocusState;

/// State for a dropdown control
#[derive(Debug, Clone)]
pub struct DropdownState {
    /// Currently selected index
    pub selected: usize,
    /// Display names for options (shown in UI)
    pub options: Vec<String>,
    /// Actual values for options (stored in config)
    /// If empty, options are used as values
    pub values: Vec<String>,
    /// Label displayed before the dropdown
    pub label: String,
    /// Whether the dropdown is currently open
    pub open: bool,
    /// Focus state
    pub focus: FocusState,
}

impl DropdownState {
    /// Create a new dropdown state where display names equal values
    pub fn new(options: Vec<String>, label: impl Into<String>) -> Self {
        Self {
            selected: 0,
            options,
            values: Vec::new(), // Empty means use options as values
            label: label.into(),
            open: false,
            focus: FocusState::Normal,
        }
    }

    /// Create a dropdown with separate display names and values
    pub fn with_values(
        options: Vec<String>,
        values: Vec<String>,
        label: impl Into<String>,
    ) -> Self {
        debug_assert_eq!(options.len(), values.len());
        Self {
            selected: 0,
            options,
            values,
            label: label.into(),
            open: false,
            focus: FocusState::Normal,
        }
    }

    /// Set the initially selected index
    pub fn with_selected(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected = index;
        }
        self
    }

    /// Set the focus state
    pub fn with_focus(mut self, focus: FocusState) -> Self {
        self.focus = focus;
        self
    }

    /// Get the currently selected value (for storing in config)
    pub fn selected_value(&self) -> Option<&str> {
        if self.values.is_empty() {
            self.options.get(self.selected).map(|s| s.as_str())
        } else {
            self.values.get(self.selected).map(|s| s.as_str())
        }
    }

    /// Get the currently selected display name (for UI)
    pub fn selected_option(&self) -> Option<&str> {
        self.options.get(self.selected).map(|s| s.as_str())
    }

    /// Find the index of a value
    pub fn index_of_value(&self, value: &str) -> Option<usize> {
        if self.values.is_empty() {
            self.options.iter().position(|o| o == value)
        } else {
            self.values.iter().position(|v| v == value)
        }
    }

    /// Toggle the dropdown open/closed
    pub fn toggle_open(&mut self) {
        if self.focus != FocusState::Disabled {
            self.open = !self.open;
        }
    }

    /// Select the next option
    pub fn select_next(&mut self) {
        if self.focus != FocusState::Disabled && !self.options.is_empty() {
            self.selected = (self.selected + 1) % self.options.len();
        }
    }

    /// Select the previous option
    pub fn select_prev(&mut self) {
        if self.focus != FocusState::Disabled && !self.options.is_empty() {
            self.selected = if self.selected == 0 {
                self.options.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Select an option by index
    pub fn select(&mut self, index: usize) {
        if self.focus != FocusState::Disabled && index < self.options.len() {
            self.selected = index;
            self.open = false;
        }
    }
}

/// Colors for the dropdown control
#[derive(Debug, Clone, Copy)]
pub struct DropdownColors {
    /// Label color
    pub label: Color,
    /// Selected option text color
    pub selected: Color,
    /// Border/bracket color
    pub border: Color,
    /// Arrow indicator color
    pub arrow: Color,
    /// Option text in dropdown menu
    pub option: Color,
    /// Highlighted option background
    pub highlight_bg: Color,
    /// Focused highlight color
    pub focused: Color,
    /// Disabled color
    pub disabled: Color,
}

impl Default for DropdownColors {
    fn default() -> Self {
        Self {
            label: Color::White,
            selected: Color::Cyan,
            border: Color::Gray,
            arrow: Color::DarkGray,
            option: Color::White,
            highlight_bg: Color::DarkGray,
            focused: Color::Cyan,
            disabled: Color::DarkGray,
        }
    }
}

impl DropdownColors {
    /// Create colors from theme
    pub fn from_theme(theme: &crate::view::theme::Theme) -> Self {
        Self {
            label: theme.editor_fg,
            selected: theme.menu_active_fg,
            border: theme.line_number_fg,
            arrow: theme.line_number_fg,
            option: theme.editor_fg,
            highlight_bg: theme.selection_bg,
            focused: theme.selection_bg,
            disabled: theme.line_number_fg,
        }
    }
}

/// Layout information returned after rendering for hit testing
#[derive(Debug, Clone)]
pub struct DropdownLayout {
    /// The main dropdown button area
    pub button_area: Rect,
    /// Areas for each option when open (empty if closed)
    pub option_areas: Vec<Rect>,
    /// The full control area
    pub full_area: Rect,
}

impl DropdownLayout {
    /// Check if a point is on the dropdown button
    pub fn is_button(&self, x: u16, y: u16) -> bool {
        x >= self.button_area.x
            && x < self.button_area.x + self.button_area.width
            && y >= self.button_area.y
            && y < self.button_area.y + self.button_area.height
    }

    /// Get the option index at a point, if any
    pub fn option_at(&self, x: u16, y: u16) -> Option<usize> {
        for (i, area) in self.option_areas.iter().enumerate() {
            if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height {
                return Some(i);
            }
        }
        None
    }
}

/// Render a dropdown control (closed state)
///
/// # Arguments
/// * `frame` - The ratatui frame to render to
/// * `area` - Rectangle where the control should be rendered
/// * `state` - The dropdown state
/// * `colors` - Colors for rendering
///
/// # Returns
/// Layout information for hit testing
pub fn render_dropdown(
    frame: &mut Frame,
    area: Rect,
    state: &DropdownState,
    colors: &DropdownColors,
) -> DropdownLayout {
    let empty_layout = DropdownLayout {
        button_area: Rect::default(),
        option_areas: Vec::new(),
        full_area: area,
    };

    if area.height == 0 || area.width < 10 {
        return empty_layout;
    }

    let (label_color, selected_color, border_color, arrow_color) = match state.focus {
        FocusState::Normal => (colors.label, colors.selected, colors.border, colors.arrow),
        FocusState::Focused => (colors.focused, colors.selected, colors.focused, colors.focused),
        FocusState::Disabled => {
            (colors.disabled, colors.disabled, colors.disabled, colors.disabled)
        }
    };

    let selected_text = state.selected_option().unwrap_or("");
    let max_option_len = state.options.iter().map(|s| s.len()).max().unwrap_or(10);
    let display_width = max_option_len.max(selected_text.len()).min(20);
    let padded = format!("{:width$}", selected_text, width = display_width);

    let arrow = if state.open { "▲" } else { "▼" };

    let line = Line::from(vec![
        Span::styled(&state.label, Style::default().fg(label_color)),
        Span::styled(": ", Style::default().fg(label_color)),
        Span::styled("[", Style::default().fg(border_color)),
        Span::styled(padded, Style::default().fg(selected_color)),
        Span::styled(" ", Style::default()),
        Span::styled(arrow, Style::default().fg(arrow_color)),
        Span::styled("]", Style::default().fg(border_color)),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);

    let label_width = state.label.len() as u16 + 2; // ": "
    let button_start = area.x + label_width;
    let button_width = display_width as u16 + 4; // "[" + text + " " + arrow + "]"

    let mut option_areas = Vec::new();

    // Render dropdown menu if open
    if state.open && area.height > 1 {
        let menu_y = area.y + 1;
        let available_height = area.height.saturating_sub(1) as usize;
        let options_to_show = state.options.len().min(available_height);

        for (i, option) in state.options.iter().take(options_to_show).enumerate() {
            let option_area = Rect::new(button_start, menu_y + i as u16, button_width, 1);
            option_areas.push(option_area);

            let is_selected = i == state.selected;
            let (bg, fg) = if is_selected {
                (colors.highlight_bg, colors.selected)
            } else {
                (Color::Reset, colors.option)
            };

            let padded_option = format!(" {:width$} ", option, width = display_width);
            let option_line = Line::from(vec![Span::styled(
                padded_option,
                Style::default().fg(fg).bg(bg),
            )]);

            let option_para = Paragraph::new(option_line);
            frame.render_widget(option_para, option_area);
        }
    }

    DropdownLayout {
        button_area: Rect::new(button_start, area.y, button_width, 1),
        option_areas,
        full_area: Rect::new(area.x, area.y, button_start - area.x + button_width, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn test_frame<F>(width: u16, height: u16, f: F)
    where
        F: FnOnce(&mut Frame, Rect),
    {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, width, height);
                f(frame, area);
            })
            .unwrap();
    }

    #[test]
    fn test_dropdown_renders() {
        test_frame(40, 1, |frame, area| {
            let state = DropdownState::new(
                vec!["Option A".to_string(), "Option B".to_string()],
                "Choice",
            );
            let colors = DropdownColors::default();
            let layout = render_dropdown(frame, area, &state, &colors);

            assert!(layout.button_area.width > 0);
            assert!(layout.option_areas.is_empty()); // Closed
        });
    }

    #[test]
    fn test_dropdown_open() {
        test_frame(40, 5, |frame, area| {
            let mut state = DropdownState::new(
                vec!["Option A".to_string(), "Option B".to_string()],
                "Choice",
            );
            state.open = true;
            let colors = DropdownColors::default();
            let layout = render_dropdown(frame, area, &state, &colors);

            assert_eq!(layout.option_areas.len(), 2);
        });
    }

    #[test]
    fn test_dropdown_selection() {
        let mut state = DropdownState::new(
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
            ],
            "Test",
        );

        assert_eq!(state.selected, 0);
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_next();
        assert_eq!(state.selected, 2);
        state.select_next();
        assert_eq!(state.selected, 0); // Wraps around

        state.select_prev();
        assert_eq!(state.selected, 2); // Wraps around backwards
    }

    #[test]
    fn test_dropdown_select_by_index() {
        let mut state = DropdownState::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            "Test",
        );
        state.open = true;
        state.select(2);
        assert_eq!(state.selected, 2);
        assert!(!state.open); // Should close after selection
    }

    #[test]
    fn test_dropdown_disabled() {
        let mut state = DropdownState::new(vec!["A".to_string(), "B".to_string()], "Test")
            .with_focus(FocusState::Disabled);

        state.toggle_open();
        assert!(!state.open);

        state.select_next();
        assert_eq!(state.selected, 0);
    }
}
