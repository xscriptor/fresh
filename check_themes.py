import json
import re
import os

def parse_palettes(md_path):
    with open(md_path) as f:
        content = f.read()
    sections = re.split(r'<h2[^>]*>([^<]+)</h2>', content)
    palettes = {}
    for i in range(1, len(sections), 2):
        name = sections[i].strip()
        block = sections[i+1]
        json_match = re.search(r'\{[^}]+\}', block, re.DOTALL)
        if json_match:
            palettes[name.lower()] = json.loads(json_match.group())
    return palettes

def hex_to_rgb(hex_str):
    hex_str = hex_str.lstrip('#')
    return [int(hex_str[i:i+2], 16) for i in (0, 2, 4)]

def rgb_to_hex(rgb):
    return f'#{rgb[0]:02x}{rgb[1]:02x}{rgb[2]:02x}'

def find_all_rgb(obj, path=""):
    results = []
    if isinstance(obj, list):
        if len(obj) == 3 and all(isinstance(v, int) and 0 <= v <= 255 for v in obj):
            results.append((path, obj))
        else:
            for i, v in enumerate(obj):
                results.extend(find_all_rgb(v, f"{path}[{i}]"))
    elif isinstance(obj, dict):
        for k, v in obj.items():
            results.extend(find_all_rgb(v, f"{path}.{k}"))
    return results

palettes = parse_palettes('colors.md')

themes_dir = 'themes'
for fname in sorted(os.listdir(themes_dir)):
    if not fname.endswith('.json'):
        continue
    theme_name = fname.replace('.json', '')
    if theme_name not in palettes:
        print(f"\n=== {theme_name} === NO PALETTE IN colors.md (skipping)")
        continue

    palette = palettes[theme_name]
    palette_rgbs = set()
    for i in range(16):
        key = f'color{i}'
        if key in palette:
            palette_rgbs.add(tuple(hex_to_rgb(palette[key])))

    with open(os.path.join(themes_dir, fname)) as f:
        theme = json.load(f)

    all_rgbs = find_all_rgb(theme)

    c0 = hex_to_rgb(palette.get('color0', '#000000'))
    is_light = c0[0] > 180 and c0[1] > 180 and c0[2] > 180

    mismatches = []
    for path_str, rgb in all_rgbs:
        if tuple(rgb) not in palette_rgbs:
            hex_str = rgb_to_hex(rgb)
            min_dist = float('inf')
            closest = None
            for prgb in palette_rgbs:
                dist = sum((a-b)**2 for a,b in zip(rgb, prgb))
                if dist < min_dist:
                    min_dist = dist
                    closest = prgb
            mismatches.append((path_str, rgb, hex_str, list(closest), rgb_to_hex(closest)))

    if mismatches:
        print(f"\n=== {theme_name} === LIGHT={is_light} ({len(mismatches)} mismatches)")
        for path_str, rgb, hex_str, closest_rgb, closest_hex in mismatches:
            print(f"  {path_str}: {hex_str} -> closest palette: {closest_hex} {closest_rgb}")
    else:
        print(f"\n=== {theme_name} === LIGHT={is_light} ALL COLORS MATCH PALETTE ✓")
