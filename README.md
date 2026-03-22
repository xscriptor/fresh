<h1 align="center"> Fresh Xscriptor </h1>



<div align="center">

![fresh](https://xscriptor.github.io/badges/editors/fresh.svg) ![shell](https://xscriptor.github.io/badges/languages/shell.svg) ![mit](https://xscriptor.github.io/badges/licenses/mit.svg)

Essential settings to improve accessibility of Fresh using the Xscriptor themes.


</div>

<p align="center"><img src="./assets/icon.png" width="100" alt="Xscriptor logo" /></p>


# Previews

<p align="center">
  <a href="./assets/previews/preview1.jpg">
    <img src="./assets/previews/preview1.jpg" alt="Main preview" width="850"/>
  </a>
</p>

<details>
  <summary>More</summary>

  <table>
    <tr>
      <td align="center">
        <a href="./assets/previews/preview2.jpg">
          <img src="./assets/previews/preview2.jpg" alt="Preview 3" width="380"/>
        </a>
      </td>
      <td align="center">
        <a href="./assets/previews/preview3.jpg">
          <img src="./assets/previews/preview3.jpg" alt="Preview 4" width="380"/>
        </a>
      </td>
      <td align="center">
        <a href="./assets/previews/preview4.jpg">
          <img src="./assets/previews/preview4.jpg" alt="Preview 5" width="380"/>
        </a>
      </td>
      <td align="center">
        <a href="./assets/previews/preview5.jpg">
          <img src="./assets/previews/preview5.jpg" alt="Preview 6" width="380"/>
        </a>
      </td>
      <td align="center">
        <a href="./assets/previews/preview6.jpg">
          <img src="./assets/previews/preview6.jpg" alt="Preview 7" width="380"/>
        </a>
      </td>
      <td align="center">
        <a href="./assets/previews/preview7.jpg">
          <img src="./assets/previews/preview7.jpg" alt="Preview 8" width="380"/>
        </a>
      </td>
    </tr>
  </table>
</details>



## Overview

Custom color themes for Fresh, the terminal text editor. This repository contains multiple JSON theme files compatible with Fresh. Install them into `~/.config/fresh/themes`.

## Installation

Run the installer remotely with either curl or wget:

```bash
curl -fsSL https://raw.githubusercontent.com/xscriptor/fresh/main/installer.sh | bash
```

```bash
wget -qO- https://raw.githubusercontent.com/xscriptor/fresh/main/installer.sh | bash
```

The installer creates `~/.config/fresh/themes` if it does not exist and copies all themes there.

## Uninstall

Remove the installed themes with:

```bash
curl -fsSL https://raw.githubusercontent.com/xscriptor/fresh/main/uninstaller.sh | bash
```

```bash
wget -qO- https://raw.githubusercontent.com/xscriptor/fresh/main/uninstaller.sh | bash
```

This removes the themes provided by this repository. If the directory becomes empty, it is removed.

## Manual installation

```bash
mkdir -p ~/.config/fresh/themes
cp -f themes/*.json ~/.config/fresh/themes/
```

## Available themes

- berlin
- bogota
- helsinki
- lahabana
- madrid
- miami
- oslo
- paris
- praha
- x

## Notes

- Fresh should read themes from `~/.config/fresh/themes`. Refer to Fresh’s documentation for selecting a theme inside the editor.
- No elevated privileges are required. All operations target your home directory.
