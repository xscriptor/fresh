<h1 align="center">Theme system</h1>

<h2 align="center">Palette reference</h2>

<p>Each theme has a 16-color palette defined in <code>colors.md</code>. These are the only colors used across all theme files.</p>

<table align="center">
  <tr><th>Index</th><th>Dark theme</th><th>Light theme</th></tr>
  <tr><td><code>color0</code></td><td>Background (darkest)</td><td>Background (lightest)</td></tr>
  <tr><td><code>color1</code></td><td>Red</td><td>Red</td></tr>
  <tr><td><code>color2</code></td><td>Green</td><td>Green</td></tr>
  <tr><td><code>color3</code></td><td>Yellow</td><td>Yellow</td></tr>
  <tr><td><code>color4</code></td><td>Blue</td><td>Blue</td></tr>
  <tr><td><code>color5</code></td><td>Purple</td><td>Purple</td></tr>
  <tr><td><code>color6</code></td><td>Cyan</td><td>Cyan</td></tr>
  <tr><td><code>color7</code></td><td>Foreground (lightest)</td><td>Foreground (darkest)</td></tr>
  <tr><td><code>color8</code></td><td>Bright black (dim)</td><td>Dim / medium</td></tr>
  <tr><td><code>color9</code></td><td>Bright red</td><td>Bright red</td></tr>
  <tr><td><code>color10</code></td><td>Bright green</td><td>Bright green</td></tr>
  <tr><td><code>color11</code></td><td>Bright yellow</td><td>Bright yellow</td></tr>
  <tr><td><code>color12</code></td><td>Bright blue</td><td>Bright blue</td></tr>
  <tr><td><code>color13</code></td><td>Bright purple</td><td>Bright purple</td></tr>
  <tr><td><code>color14</code></td><td>Bright cyan</td><td>Bright cyan</td></tr>
  <tr><td><code>color15</code></td><td>Bright white</td><td>Foreground (darkest)</td></tr>
</table>

<p>A theme is <strong>light</strong> when <code>color0</code> is a light color (all RGB components &gt; 180), and <strong>dark</strong> when <code>color0</code> is dark.</p>

<h2 align="center">Scripts</h2>

<table align="center">
  <tr><th>Script</th><th>Description</th></tr>
  <tr><td><code>python3 scripts/generate_themes.py</code></td><td>Regenerate all theme JSON files from <code>colors.md</code> into <code>dist/</code></td></tr>
  <tr><td><code>python3 scripts/verify_themes.py</code></td><td>Verify all colors in <code>themes/*.json</code> match the palettes in <code>colors.md</code></td></tr>
</table>

<h2 align="center">Default field mapping</h2>

<p>When generating themes with <code>generate_themes.py</code>, each theme field is mapped to a palette index as follows:</p>

<table align="center">
  <tr><th>Field</th><th>Palette</th><th>Field</th><th>Palette</th></tr>
  <tr><td>editor.bg</td><td>color0</td><td>syntax.keyword</td><td>color3</td></tr>
  <tr><td>editor.fg</td><td>color7</td><td>syntax.string</td><td>color2</td></tr>
  <tr><td>editor.cursor</td><td>color7</td><td>syntax.function</td><td>color6</td></tr>
  <tr><td>editor.inactive_cursor</td><td>color8</td><td>syntax.type</td><td>color1</td></tr>
  <tr><td>editor.selection_bg</td><td>color8</td><td>syntax.variable</td><td>color7</td></tr>
  <tr><td>editor.current_line_bg</td><td>color0</td><td>syntax.constant</td><td>color2</td></tr>
  <tr><td>editor.line_number_fg</td><td>color8</td><td>syntax.operator</td><td>color5</td></tr>
  <tr><td>editor.line_number_bg</td><td>color0</td><td>syntax.comment</td><td>color8</td></tr>
  <tr><td>editor.diff_add_bg</td><td>color2</td><td>diagnostic.error_fg</td><td>color1</td></tr>
  <tr><td>editor.diff_remove_bg</td><td>color1</td><td>diagnostic.error_bg</td><td>color8</td></tr>
  <tr><td>editor.diff_modify_bg</td><td>color3</td><td>diagnostic.warning_fg</td><td>color3</td></tr>
  <tr><td>ui.tab_active_fg</td><td>color3</td><td>diagnostic.warning_bg</td><td>color8</td></tr>
  <tr><td>ui.tab_active_bg</td><td>color0</td><td>diagnostic.info_fg</td><td>color6</td></tr>
  <tr><td>ui.tab_inactive_fg</td><td>color8</td><td>diagnostic.info_bg</td><td>color8</td></tr>
  <tr><td>ui.tab_inactive_bg</td><td>color0</td><td>diagnostic.hint_fg</td><td>color8</td></tr>
  <tr><td>ui.tab_close_hover_fg</td><td>color1</td><td>diagnostic.hint_bg</td><td>color0</td></tr>
  <tr><td>ui.menu_bg</td><td>color0</td><td>search.match_bg</td><td>color3</td></tr>
  <tr><td>ui.menu_fg</td><td>color7</td><td>search.match_fg</td><td>color0</td></tr>
  <tr><td>ui.menu_highlight_bg</td><td>color3</td><td>ui.status_bar_fg</td><td>color7</td></tr>
  <tr><td>ui.menu_highlight_fg</td><td>color0</td><td>ui.status_bar_bg</td><td>color0</td></tr>
  <tr><td>ui.menu_border_fg</td><td>color8</td><td>ui.help_key_fg</td><td>color3</td></tr>
  <tr><td>ui.menu_disabled_fg</td><td>color8</td><td>ui.help_indicator_fg</td><td>color1</td></tr>
  <tr><td>ui.prompt_selection_bg</td><td>color3</td><td>ui.status_warning_indicator_bg</td><td>color3</td></tr>
  <tr><td>ui.popup_selection_bg</td><td>color3</td><td>ui.status_error_indicator_bg</td><td>color1</td></tr>
  <tr><td>ui.suggestion_selected_bg</td><td>color3</td><td>ui.scrollbar_thumb_fg</td><td>color8</td></tr>
  <tr><td>ui.split_separator_hover_fg</td><td>color3</td><td>ui.scrollbar_thumb_hover_fg</td><td>color3</td></tr>
</table>

<p>The full mapping is defined in <code>FIELD_MAP</code> inside <code>scripts/generate_themes.py</code>. The curated themes in <code>themes/</code> may use different palette indices per field &mdash; that is intentional design.</p>

<h2 align="center">Adding a new theme</h2>

<ol>
  <li>Add a new section in <code>colors.md</code> with the 16-color palette.</li>
  <li>Run <code>python3 scripts/generate_themes.py</code> to generate the base theme into <code>dist/</code>.</li>
  <li>Copy the generated file to <code>themes/</code> and adjust syntax/UI field mappings as desired.</li>
  <li>Run <code>python3 scripts/verify_themes.py</code> to ensure all colors stay within the palette.</li>
  <li>Add the theme name to <code>README.md</code> and <code>uninstaller.sh</code>.</li>
</ol>
