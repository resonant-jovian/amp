# Quick Start Guide - AMP TUI

## ⚡ 30-Second Setup

```bash
# Navigate to server directory
cd server

# Run the TUI application
cargo run
```

That's it! Your interactive TUI is running.

## 📊 What You See

A terminal UI with 5 tabs:

```
[Dashboard] [Correlate] [Test] [Benchmark] [Updates]
┌────────────────────┐
│ AMP TUI - Address Parking Mapper │
│                                │
│ Tab content here...            │
│                                │
└────────────────────┘
 Status: Ready | cutoff: 20.0m
```

## ⌨️ Essential Keyboard Shortcuts

| Key | What it does |
|-----|-------------|
| `1` - `5` | Jump to specific tab |
| `←` `→` | Navigate between tabs |
| `a` | Cycle through algorithms |
| `+` / `-` | Increase/decrease cutoff distance |
| `Enter` | Run operation in current tab |
| `q` | Quit application |

## 📁 What Each Tab Does

### 1️⃣ Dashboard
- Welcome screen
- Quick reference guide
- Status at a glance

### 2️⃣ Correlate
- Select algorithm: `[a]`
- Set distance cutoff: `[+] [-]`
- Press `Enter` to run
- See live progress bar
- Results persist when you switch tabs

### 3️⃣ Test (Browser)
- Visualize correlations in your browser
- Same algorithm/cutoff controls
- `Enter` launches browser

### 4️⃣ Benchmark
- Tests all 6 algorithms
- Compares performance
- Shows timing stats
- `Enter` to start

### 5️⃣ Check Updates
- Checks Malmö data portal
- Detects new data
- `Enter` to check

## 🔧 Algorithms Available

Press `a` to cycle through:

- **KD-Tree** ☀️ (Fast, default)
- **R-Tree** (Efficient)
- **Grid** (Simple)
- **Distance** (Basic)
- **Raycasting** (Polygon test)
- **Overlapping Chunks** (Advanced)

## 💡 Pro Tips

1. **Results persist** - Switch tabs, come back, your data's still there
2. **Adjust cutoff** - Use `+` and `-` to find the sweet spot
3. **Try each algorithm** - Press `a` to see which is fastest
4. **Use Tab 3** - Visual browser test is super helpful
5. **Check benchmark** - Tab 4 shows which algorithm wins

## 🛠️ Build from Source

```bash
# Development (faster compilation)
cargo run

# Release (optimized)
cargo build --release
./target/release/amp-server
```

## 🎣 Troubleshooting

### "Terminal too small"
- Resize your terminal window to at least 80x20
- The UI adapts automatically

### "No data loaded"
- Check internet connection (API calls to Malmö data)
- Try again - data portal might be temporarily down

### "Weird characters in output"
- Make sure your terminal supports UTF-8
- Most modern terminals do

## 📋 File Structure

```
server/
├── src/
│  ├── main.rs          ← Entry point (15 lines)
│  ├── ui.rs            ← Main TUI logic (800 lines)
│  ├── tui.rs           ← Terminal setup
│  ├── classification.rs ← Algorithms
│  └── app.rs           ← Module exports
├── Cargo.toml
└── README.md
```

## 🔤 How It Works (Elm Architecture)

```rust
User Input (key press)
    ↓
 Key Handler (on_key)
    ↓
State Update (msg -> state)
    ↓
Render UI (draw)
    ↓
Terminal Output
```

This is called **Elm architecture** - used by web frameworks like React.
Functional, predictable, easy to test.

## 🚀 Next: Explore the Code

Ready to dive deeper?

1. Read `IMPLEMENTATION_SUMMARY.md` for architecture details
2. Check `ui.rs` for the rendering logic
3. Look at `classification.rs` for algorithm integration

## 📧 Need Help?

- Check GitHub: https://github.com/resonant-jovian/amp
- Read Ratatui docs: https://ratatui.rs
- Learn Elm: https://guide.elm-lang.org

---

**Enjoy your new TUI!** 🐁
