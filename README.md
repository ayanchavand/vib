# vib

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

![GitHub Release](https://img.shields.io/github/v/release/ayanchavand/vib?style=for-the-badge)
![Last Commit](https://img.shields.io/github/last-commit/ayanchavand/vib?style=for-the-badge)

A terminal file browser with LocalSend built in. Manage, organize, and move files around your machine, then send or receive them across your other devices without ever leaving the terminal.

- Dual-pane browsing, each pane its own tab
- Bookmark directories for quick access
- Send and receive files via LocalSend without leaving vib
- Cut, copy, paste, create folders, and rename files or folders
- Multi-select for cut, copy, or send via LocalSend
- Text file preview
- Catppuccin-themed UI, built for ricing

---

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Switch between dual panes |
| `Up` / `Down` / `k` / `j` | Navigate the current pane |
| `Enter` / `l` / `Right` | Open directory |
| `Backspace` / `h` / `Left` | Go up one directory |
| `Space` | Select / deselect highlighted file or directory |
| `v` | Select / deselect all items in current directory |
| `b` | Bookmark current directory |
| `B` | Jump to a bookmarked directory |
| `x` | Cut selected item(s) |
| `c` | Copy selected item(s) |
| `p` | Paste |
| `n` | Create new folder |
| `R` | Rename file or folder |
| `Space` (preview pane) | Toggle text preview of highlighted file |
| `s` | Send selected item(s) via LocalSend |
| `r` | Rescan network for LocalSend devices |
| `y` / `Enter` | Accept incoming LocalSend transfer (saves to `~/Downloads`) |
| `d` | Decline incoming LocalSend transfer |
| `q` / `Ctrl+C` | Exit |

---

## Building from Source

Requires [Rust](https://rustup.rs/) 1.80+.

```bash
git clone https://github.com/ayanchavand/vib.git
cd vib
cargo build --release
./target/release/vib
```

---

## Troubleshooting

### Device not appearing during send

**Linux firewall.** Port `53317` is often blocked by default.

- UFW:
  ```bash
  sudo ufw allow 53317/tcp
  sudo ufw allow 53317/udp
  ```
- FirewallD:
  ```bash
  sudo firewall-cmd --add-port=53317/tcp --permanent
  sudo firewall-cmd --add-port=53317/udp --permanent
  sudo firewall-cmd --reload
  ```
- iptables:
  ```bash
  sudo iptables -A INPUT -p tcp --dport 53317 -j ACCEPT
  sudo iptables -A INPUT -p udp --dport 53317 -j ACCEPT
  ```

**Multiple network interfaces.** Docker or Tailscale can route multicast traffic to a virtual bridge (`docker0`, `tailscale0`) instead of Wi-Fi (`wlan0`). Press `r` to force a multi-interface subnet scan, and confirm both devices are on the same access point.

### Phone stuck on "Waiting for response..."

LocalSend requires manual confirmation. When a transfer is initiated, `vib` prompts you automatically — press `y` to accept.

---

## License

MIT