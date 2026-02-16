# fswhy

A terminal-based disk usage analyzer with interactive tree navigation and gradient coloring.

## Features

- **Full Directory Scan**: Recursively scans directories and calculates cumulative sizes
- **Interactive Navigation**: Keyboard-driven tree expand/collapse with cursor movement
- **Size-based Sorting**: Sort by size (descending) or name, directories always first
- **Gradient Coloring**: Visual size indication via color gradients for dirs/files
- **Viewport Scrolling**: Handles large directories with scrollable viewport
- **Customizable Themes**: TOML-based theme with preset and RGB color support


## Usage

```bash
# Scan current directory
fswhy

# Scan specific path
fswhy /path/to/directory
```

## Controls

| Key | Action |
|-----|--------|
| `↑` / `k` | Move cursor up |
| `↓` / `j` | Move cursor down |
| `Enter` / `t` | Toggle expand/collapse at cursor |
| `0-9` + `Enter` | Toggle by index number |
| `s` | Toggle sort mode (size/name) |
| `Backspace` | Clear input buffer |
| `q` / `Ctrl+C` | Quit |

## Theme Configuration

Create `theme.toml` in the working directory or set `FSWHY_THEME` environment variable.

### Preset Colors

`reset`, `fg_reset`, `invert`, `red`, `yellow`, `blue`, `green`, `cyan`, `magenta`, `white`

### Example

```toml
reset = { name = "reset" }
fg_reset = { name = "fg_reset" }

dir = { name = "blue" }
file = { name = "white" }
error = { name = "red" }

highlight_start = { name = "invert" }
highlight_end = { name = "reset" }

# Gradient colors (RGB)
dir_gradient_start = { r = 58, g = 123, b = 213 }
dir_gradient_end = { r = 0, g = 210, b = 255 }
file_gradient_start = { r = 180, g = 180, b = 180 }
file_gradient_end = { r = 255, g = 200, b = 120 }
```

## Roadmap

- [x] Recursive filesystem scanning
- [x] Logical folding and node indexing
- [x] Interactive navigation (cursor, viewport)
- [x] Size-based sorting with gradient colors
- [x] Customizable theme system
- [ ] Performance optimization (parallel scan, MFT)
- [ ] Filter and search
- [ ] Export reports (heat map maybe?)

## License

MIT
