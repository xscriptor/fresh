import json, re, os

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
COLORS_FILE = os.path.join(BASE_DIR, 'colors.md')
DIST_DIR = os.path.join(BASE_DIR, 'dist')

FIELD_MAP = {
    'editor.bg': 'color0',
    'editor.fg': 'color7',
    'editor.cursor': 'color7',
    'editor.inactive_cursor': 'color8',
    'editor.selection_bg': 'color8',
    'editor.current_line_bg': 'color0',
    'editor.line_number_fg': 'color8',
    'editor.line_number_bg': 'color0',
    'editor.diff_add_bg': 'color2',
    'editor.diff_remove_bg': 'color1',
    'editor.diff_modify_bg': 'color3',
    'ui.tab_active_fg': 'color3',
    'ui.tab_active_bg': 'color0',
    'ui.tab_inactive_fg': 'color8',
    'ui.tab_inactive_bg': 'color0',
    'ui.tab_separator_bg': 'color8',
    'ui.tab_close_hover_fg': 'color1',
    'ui.tab_hover_bg': 'color0',
    'ui.menu_bg': 'color0',
    'ui.menu_fg': 'color7',
    'ui.menu_active_bg': 'color0',
    'ui.menu_active_fg': 'color3',
    'ui.menu_dropdown_bg': 'color0',
    'ui.menu_dropdown_fg': 'color7',
    'ui.menu_highlight_bg': 'color3',
    'ui.menu_highlight_fg': 'color0',
    'ui.menu_border_fg': 'color8',
    'ui.menu_separator_fg': 'color8',
    'ui.menu_hover_bg': 'color0',
    'ui.menu_hover_fg': 'color3',
    'ui.menu_disabled_fg': 'color8',
    'ui.menu_disabled_bg': 'color0',
    'ui.status_bar_fg': 'color7',
    'ui.status_bar_bg': 'color0',
    'ui.prompt_fg': 'color7',
    'ui.prompt_bg': 'color0',
    'ui.prompt_selection_fg': 'color0',
    'ui.prompt_selection_bg': 'color3',
    'ui.popup_border_fg': 'color8',
    'ui.popup_bg': 'color0',
    'ui.popup_selection_fg': 'color0',
    'ui.popup_selection_bg': 'color3',
    'ui.popup_text_fg': 'color7',
    'ui.suggestion_bg': 'color0',
    'ui.suggestion_selected_bg': 'color3',
    'ui.help_bg': 'color0',
    'ui.help_fg': 'color7',
    'ui.help_key_fg': 'color3',
    'ui.help_separator_fg': 'color8',
    'ui.help_indicator_fg': 'color1',
    'ui.help_indicator_bg': 'color0',
    'ui.inline_code_bg': 'color8',
    'ui.split_separator_fg': 'color8',
    'ui.split_separator_hover_fg': 'color3',
    'ui.scrollbar_track_fg': 'color0',
    'ui.scrollbar_thumb_fg': 'color8',
    'ui.scrollbar_track_hover_fg': 'color0',
    'ui.scrollbar_thumb_hover_fg': 'color3',
    'ui.compose_margin_bg': 'color0',
    'ui.semantic_highlight_bg': 'color0',
    'ui.terminal_bg': 'Default',
    'ui.terminal_fg': 'Default',
    'ui.status_warning_indicator_bg': 'color3',
    'ui.status_warning_indicator_fg': 'color0',
    'ui.status_error_indicator_bg': 'color1',
    'ui.status_error_indicator_fg': 'color7',
    'ui.status_warning_indicator_hover_bg': 'color3',
    'ui.status_warning_indicator_hover_fg': 'color0',
    'ui.status_error_indicator_hover_bg': 'color1',
    'ui.status_error_indicator_hover_fg': 'color7',
    'ui.tab_drop_zone_bg': 'color3',
    'ui.tab_drop_zone_border': 'color3',
    'search.match_bg': 'color3',
    'search.match_fg': 'color0',
    'diagnostic.error_fg': 'color1',
    'diagnostic.error_bg': 'color8',
    'diagnostic.warning_fg': 'color3',
    'diagnostic.warning_bg': 'color8',
    'diagnostic.info_fg': 'color6',
    'diagnostic.info_bg': 'color8',
    'diagnostic.hint_fg': 'color8',
    'diagnostic.hint_bg': 'color0',
    'syntax.keyword': 'color3',
    'syntax.string': 'color2',
    'syntax.function': 'color6',
    'syntax.type': 'color1',
    'syntax.variable': 'color7',
    'syntax.constant': 'color2',
    'syntax.operator': 'color5',
    'syntax.comment': 'color8',
}


def hex_to_rgb(hex_str):
    hex_str = hex_str.lstrip('#')
    return [int(hex_str[i:i + 2], 16) for i in (0, 2, 4)]


def parse_palettes(md_path):
    with open(md_path) as f:
        content = f.read()
    sections = re.split(r'<h2[^>]*>([^<]+)</h2>', content)
    palettes = {}
    for i in range(1, len(sections), 2):
        name = sections[i].strip()
        block = sections[i + 1]
        json_match = re.search(r'\{[^}]+\}', block, re.DOTALL)
        if json_match:
            palettes[name.lower()] = json.loads(json_match.group())
    return palettes


def build_theme(palette, name):
    sections = {}
    for section_key in ['editor', 'ui', 'search', 'diagnostic', 'syntax']:
        sections[section_key] = {}
    for full_path, color_key in FIELD_MAP.items():
        section, field = full_path.split('.', 1)
        if color_key == 'Default':
            sections[section][field] = 'Default'
        else:
            sections[section][field] = hex_to_rgb(palette[color_key])
    result = {'name': name}
    result.update(sections)
    return result


def main():
    palettes = parse_palettes(COLORS_FILE)
    os.makedirs(DIST_DIR, exist_ok=True)
    generated = []
    for name in sorted(palettes):
        theme = build_theme(palettes[name], name)
        filepath = os.path.join(DIST_DIR, f'{name}.json')
        with open(filepath, 'w') as f:
            json.dump(theme, f, indent=2)
            f.write('\n')
        generated.append(name)
    print(f'Generated {len(generated)} themes in {DIST_DIR}:')
    for name in generated:
        print(f'  {name}.json')


if __name__ == '__main__':
    main()
