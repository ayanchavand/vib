# vib-send

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

![GitHub Release](https://img.shields.io/github/v/release/ayanchavand/vib?style=for-the-badge)
![Last Commit](https://img.shields.io/github/last-commit/ayanchavand/vib?style=for-the-badge)

A terminal-based file browser and cross-platform LocalSend Protocol (v2.1) client. Navigate your file system, tag files or folders, discover nearby LocalSend devices, and send or receive files over your local network without a cloud server.

---

## Features

- Full TUI file browser with fast navigation and metadata preview
- Automatic LocalSend peer discovery over UDP multicast (`224.0.0.167:53317`) and broadcast
- Interactive receive queue with file previews, accept/decline actions
- Real-time transfer progress with speed and byte counts
- Self-signed X.509 TLS certificate generation with SHA-256 fingerprint verification
- Multi-interface support (Docker bridges, Tailscale, VPNs), binding across all active IPv4 interfaces

---

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `L` / `Shift+L` | Toggle between File Explorer mode and LocalSend UI mode |
| `Tab` / `Shift+Tab` | Switch tabs (`Files`, `Peers`, `Receive`, `Transfers`, `Settings`) |
| `Space` | Tag / untag highlighted file or directory |
| `v` | Tag / untag all items in current directory |
| `s` | Open send modal to choose a destination device |
| `r` / `R` | Rescan network for LocalSend devices |
| `y` / `Enter` | Accept incoming transfer (saves to `~/Downloads`) |
| `n` / `d` | Decline incoming transfer |
| `Up` / `Down` / `k` / `j` | Navigate lists |
| `Enter` / `l` / `Right` | Open directory / select peer |
| `Backspace` / `h` / `Left` | Go up one directory |
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

### Device not appearing in Peers

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

LocalSend requires manual confirmation. When a transfer is initiated, `vib-send` switches to the `Receive` tab automatically — press `y` to accept.

---

## License

MIT