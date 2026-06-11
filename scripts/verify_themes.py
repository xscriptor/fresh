import json, re, os, sys

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
COLORS_FILE = os.path.join(BASE_DIR, 'colors.md')
THEMES_DIR = os.path.join(BASE_DIR, 'themes')


def hex_to_rgb(hex_str):
    hex_str = hex_str.lstrip('#')
    return tuple(int(hex_str[i:i + 2], 16) for i in (0, 2, 4))


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


def find_all_rgb(obj):
    results = []
    if isinstance(obj, list):
        if len(obj) == 3 and all(isinstance(v, int) and 0 <= v <= 255 for v in obj):
            results.append(tuple(obj))
        else:
            for v in obj:
                results.extend(find_all_rgb(v))
    elif isinstance(obj, dict):
        for v in obj.values():
            results.extend(find_all_rgb(v))
    return results


def main():
    palettes = parse_palettes(COLORS_FILE)
    exit_code = 0

    for fname in sorted(os.listdir(THEMES_DIR)):
        if not fname.endswith('.json'):
            continue
        name = fname.replace('.json', '')
        if name not in palettes:
            print(f'{name}: SKIPPED (no palette in colors.md)')
            continue

        palette = palettes[name]
        palette_rgbs = set()
        for i in range(16):
            palette_rgbs.add(hex_to_rgb(palette[f'color{i}']))

        with open(os.path.join(THEMES_DIR, fname)) as f:
            theme = json.load(f)

        all_rgbs = find_all_rgb(theme)
        mismatches = [rgb for rgb in all_rgbs if rgb not in palette_rgbs]

        if mismatches:
            print(f'{name}: FAIL ({len(mismatches)} color(s) not in palette)')
            for rgb in mismatches:
                print(f'  [{rgb[0]}, {rgb[1]}, {rgb[2]}] #{rgb[0]:02x}{rgb[1]:02x}{rgb[2]:02x}')
            exit_code = 1
        else:
            print(f'{name}: OK')

    sys.exit(exit_code)


if __name__ == '__main__':
    main()
