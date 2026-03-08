# ReadFlow

<p align="center">
  <a href="https://github.com/irfancode/readflow/releases/latest">
    <img src="https://img.shields.io/github/v/release/readflow/readflow?color=blue&style=flat-square" alt="Version"/>
  </a>
  <a href="https://github.com/irfancode/readflow/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/readflow/readflow/build.yml?color=blue&style=flat-square" alt="Build"/>
  </a>
  <a href="https://crates.io/crates/readflow">
    <img src="https://img.shields.io/crates/d/readflow?color=blue&style=flat-square" alt="Downloads"/>
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/readflow/readflow?color=blue&style=flat-square" alt="License"/>
  </a>
</p>

---

## What is ReadFlow?

**ReadFlow** is a modern, lightweight TUI (Terminal User Interface) web browser written in Rust. Designed for power users, developers, and anyone who prefers a keyboard-driven browsing experience.

## Features

### 🚀 Core Features
- **Keyboard-driven browsing** - Full control without touching the mouse
- **Reader Mode** - Distraction-free article reading with clean formatting
- **Multiple Themes** - Dark, Light, and Sepia themes for comfortable reading
- **Page Search** - Find text within pages instantly
- **History Navigation** - Full back/forward navigation support
- **Bookmarks** - Save your favorite sites
- **Feed Reader** - Subscribe to RSS/Atom feeds
- **Article Export** - Export articles to Markdown, HTML, or plain text
- **Tabbed Browsing** - Open multiple pages in tabs
- **Mouse Support** - Click links and scroll with mouse

### 🔧 Technical Features
- **Cross-platform** - Works on Linux, macOS, and Windows
- **Fast** - Built with Rust for maximum performance
- **SQLite Storage** - Local database for history, bookmarks, and feeds
- **AI Integration** - Optional Ollama integration for article summarization
- **Minimal Dependencies** - Runs anywhere Rust compiles

## Installation

### Quick Install (One-Line)

```bash
# Linux/macOS
curl -sL https://raw.githubusercontent.com/irfancode/readflow/main/install.sh | bash

# Uninstall
curl -sL https://raw.githubusercontent.com/irfancode/readflow/main/install.sh | bash -s -- --uninstall

# Windows (PowerShell)
irm https://raw.githubusercontent.com/irfancode/readflow/main/install-windows.ps1 | iex
```

### From Source

```bash
# Clone the repository
git clone https://github.com/irfancode/readflow.git
cd readflow

# Build
cargo build --release

# Install
cargo install --path .
```

### Uninstall

```bash
# Linux/macOS (if installed via our installer)
curl -sL https://raw.githubusercontent.com/irfancode/readflow/main/install.sh | bash -s -- --uninstall

# From source (if installed via cargo install)
cargo uninstall readflow
```

## Usage

### Command Line Options

```bash
readflow [OPTIONS]

Options:
  -u, --url <URL>       URL to open on startup
  -t, --theme <THEME>   Theme: dark, light, sepia (default: dark)
  -k, --insecure        Allow invalid SSL certificates
  -d, --debug           Enable debug logging
  -h, --help           Show help
```

### Examples

```bash
# Open a website
readflow --url example.com

# Start with light theme
readflow --url wikipedia.org --theme light

# Debug mode
readflow --debug
```

## Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `o` | Open URL |
| `O` | Edit current URL |
| `h` | Go back |
| `l` | Go forward |
| `R` | Reload page |
| `g` | Go to top |
| `G` | Go to bottom |

### Tabs
| Key | Action |
|-----|--------|
| `Ctrl+t` | New tab |
| `Ctrl+w` | Close tab |
| `Ctrl+n` | New tab |
| `1-9` | Switch to tab number |

### Scrolling
| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| Mouse Wheel | Scroll up/down |

### Links
| Key | Action |
|-----|--------|
| `Tab` | Next link |
| `Shift+Tab` | Previous link |
| `n` | Next link |
| `N` | Previous link |
| `Enter` | Follow selected link |
| Mouse Click | Follow link |

### Search & Actions
| Key | Action |
|-----|--------|
| `/` | Search page |
| `n` | Next search result |
| `N` | Previous search result |
| `r` | Toggle reader mode |
| `t` | Cycle theme |
| `b` | Add bookmark |
| `s` | Save article |

### Views
| Key | Action |
|-----|--------|
| `Ctrl+b` | Bookmarks |
| `f` | Feeds |
| `h` | Browser |
| `?` | Help |

### Quit
| Key | Action |
|-----|--------|
| `q` | Quit |
| `Ctrl+c` | Force quit |

## Configuration

### Location
- **Linux**: `~/.config/readflow/config.toml`
- **macOS**: `~/.config/readflow/config.toml`
- **Windows**: `%APPDATA%\readflow\config.toml`

### Example Config

```toml
# ReadFlow Configuration

# Theme: dark, light, sepia
theme = "dark"

# Default URL to open on startup
default_url = ""

# Enable cookies
enable_cookies = true

# Default search engine (for URL bar)
search_engine = "duckduckgo"

# Proxy settings (optional)
# proxy = "socks5://127.0.0.1:1080"

# Reader mode settings
[reader]
font_size = 16
line_width = 80
```

## Data Storage

ReadFlow stores all data locally:

| Data | Location |
|------|----------|
| Database | `~/.local/share/readflow/readflow.db` |
| Cache | `~/.cache/readflow` |
| Logs | `~/.local/share/readflow/logs` |

## Architecture

```
readflow/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Core types and traits
│   ├── browser/          # Web fetching & HTML parsing
│   │   ├── fetcher.rs   # HTTP client
│   │   ├── parser.rs    # HTML parsing
│   │   └── renderer.rs  # Content rendering
│   ├── reader/           # Article extraction
│   │   └── extractor.rs # Readability algorithm
│   ├── storage/          # SQLite database
│   │   └── database.rs  # Database operations
│   ├── feeds/           # RSS/Atom parsing
│   │   └── parser.rs    # Feed parsing
│   ├── ui/              # TUI interface
│   │   ├── app.rs       # Application state
│   │   └── runner.rs    # Main loop & rendering
│   ├── ai/              # AI integration
│   │   └── summarizer.rs # Ollama integration
│   └── export/          # Export functionality
│       └── exporter.rs  # Article export
├── scripts/             # Installation scripts
├── install.sh           # One-click installer
└── Cargo.toml          # Dependencies
```

## Development

### Prerequisites
- Rust 1.70+
- Cargo
- OpenSSL development headers (Linux)

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# With all features
cargo build --release --all-features
```

### Run Tests

```bash
cargo test
```

### Code Format

```bash
cargo fmt
cargo clippy
```

## FAQ

### Q: Does ReadFlow support JavaScript?
**A:** No. ReadFlow is a lightweight browser that displays raw HTML. Modern sites requiring JavaScript won't work properly.

### Q: Can I watch videos?
**A:** No. ReadFlow is a text-based browser. For videos, use a traditional browser.

### Q: How do I update ReadFlow?
**A:** 
```bash
# If installed from source
cd readflow && git pull && cargo build --release

# If installed via package manager
# Use your package manager's update command
```

### Q: Does ReadFlow support proxies?
**A:** Not currently, but you can use environment variables:
```bash
HTTP_PROXY=http://proxy:8080 HTTPS_PROXY=http://proxy:8080 readflow
```

### Q: How do I report bugs?
**A:** Please open an issue at https://github.com/irfancode/readflow/issues

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with ❤️ by the ReadFlow Team
</p>
