<h1 align="center">Fresh Xscriptor</h1>
<div align="center">
    <img src="https://xscriptor.github.io/badges/editors/fresh.svg" alt="Fresh Editor Badge" />
    <img src="https://xscriptor.github.io/badges/languages/shell.svg" alt="Shell Language Badge" />
    <img src="https://xscriptor.github.io/badges/licenses/mit.svg" alt="MIT License Badge" />
</div>

<p align="center"><em>Essential settings to improve accessibility of Fresh using the Xscriptor themes.</em></p>

<p align="center"><img src="./assets/icon.png" width="45" alt="Xscriptor logo"/></p>

<!-- Table of Contents -->
<table border="1">
    <tr>
        <th>Table of Contents</th>
    </tr>
    <tr>
        <td><a href="#overview">Overview</a></td>
    </tr>
    <tr>
        <td><a href="#installation">Installation</a></td>
    </tr>
    <tr>
        <td><a href="#uninstall">Uninstall</a></td>
    </tr>
    <tr>
        <td><a href="#manual-installation">Manual Installation</a></td>
    </tr>
    <tr>
        <td><a href="#available-themes">Available Themes</a></td>
    </tr>
    <tr>
        <td><a href="#notes">Notes</a></td>
    </tr>
    <tr>
        <td><a href="#related-documents">Related Documents</a></td>
    </tr>
    <tr>
        <td><a href="#x">X</a></td>
    </tr>
</table>

<!-- Previews -->
<h2 align="center"><em>Previews</em></h2>
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


<!-- Overview -->
<h2 align="center" id="overview">Overview</h2>
<p>Custom color themes for Fresh, the terminal text editor. This repository contains multiple JSON theme files compatible with Fresh.</p>
<p>Install them into <code>~/.config/fresh/themes</code>.</p>

<!-- Installation -->
<h2 align="center" id="installation">Installation</h2>
<p>Run the installer remotely with either curl or wget:</p>
<pre><code class="language-bash">curl -fsSL https://raw.githubusercontent.com/xscriptor/fresh/main/installer.sh | bash
</code></pre>
<pre><code class="language-bash">wget -qO- https://raw.githubusercontent.com/xscriptor/fresh/main/installer.sh | bash
</code></pre>
<p>The installer creates <code>~/.config/fresh/themes</code> if it does not exist and copies all themes there.</p>

<!-- Uninstall -->
<h2 align="center" id="uninstall">Uninstall</h2>
<p>Remove the installed themes with:</p>
<pre><code class="language-bash">curl -fsSL https://raw.githubusercontent.com/xscriptor/fresh/main/uninstaller.sh | bash
</code></pre>
<pre><code class="language-bash">wget -qO- https://raw.githubusercontent.com/xscriptor/fresh/main/uninstaller.sh | bash
</code></pre>
<p>This removes the themes provided by this repository. If the directory becomes empty, it is removed.</p>

<!-- Manual Installation -->
<h2 align="center" id="manual-installation">Manual Installation</h2>
<pre><code class="language-bash">mkdir -p ~/.config/fresh/themes
cp -f themes/*.json ~/.config/fresh/themes/
</code></pre>

<!-- Available Themes -->
<h2 align="center" id="available-themes">Available Themes</h2>
<ul>
    <li>berlin</li>
    <li>bogota</li>
    <li>helsinki</li>
    <li>lahabana</li>
    <li>madrid</li>
    <li>miami</li>
    <li>oslo</li>
    <li>paris</li>
    <li>praha</li>
    <li>x</li>
</ul>

<!-- Notes -->
<h2 align="center" id="notes">Notes</h2>
<ul>
    <li>Fresh should read themes from <code>~/.config/fresh/themes</code>. Refer to Fresh’s documentation for selecting a theme inside the editor.</li>
    <li>No elevated privileges are required. All operations target your home directory.</li>
</ul>


<h2 align="center" id="related-documents">Related Documents</h2>

<ul>
  <li><a href="./LICENSE">License</a></li>
  <li><a href="./CODE_OF_CONDUCT.md">Code of Conduct</a></li>
  <li><a href="./CONTRIBUTING.md">Contributions</a></li>
  <li><a href="./ROADMAP.md">Roadmap</a></li>
</ul>


<div align="center">
<h2 align="center" id="x">X</h2>

<a href="https://github.com/xscriptor">XGitHub</a> &middot;
<a href="https://dev.xscriptor.com">XWeb</a>
</div>