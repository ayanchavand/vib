# `vib-send` • LocalSend TUI Client & File Browser

`vib-send` is a fast, terminal-based file browser and cross-platform **LocalSend Protocol (v2.1)** client built with Rust and [Ratatui](https://github.com/ratatui/ratatui).

It allows you to navigate local file systems, tag multiple files/folders, discover nearby LocalSend devices (Android, iOS, macOS, Windows, Linux), and seamlessly send or receive files over your local Wi-Fi / Ethernet network without third-party cloud servers.

---

## Features

- 📂 **Full TUI File Browser**: Fast navigation, file metadata preview, visual directory tree.
- 📡 **Automatic LocalSend Peer Discovery**: Listens and announces on UDP multicast (`224.0.0.167:53317`) and broadcast networks.
- 📥 **Interactive Receive Queue**: Dedicated incoming transfer tab with file previews, `[y] Accept` and `[n] Decline` options.
- ⚡ **Real-Time Transfer Progress**: Live progress bars showing upload/download speeds and total bytes transferred.
- 🔒 **Secure Local Encryption**: Dynamic self-signed X.509 TLS certificate generation with SHA-256 fingerprint verification.
- 🌐 **Multi-Interface Support**: Handles complex network setups (Docker bridges, Tailscale, VPNs) by binding across all active IPv4 interfaces (`wlan0`, `eth0`).

---

## Keyboard Shortcuts & Controls

| Key | Action |
| --- | --- |
| `L` / `Shift+L` | **Toggle View Mode**: Switch between **Default File Explorer Mode** and **LocalSend UI Mode** |
| `Tab` / `Shift+Tab` | Switch between LocalSend tabs (`[1] Files`, `[2] Peers`, `[3] Receive`, `[4] Transfers`, `[5] Settings`) |
| `Space` | Tag / Untag highlighted file or directory for sending |
| `v` | Tag / Untag **all** items in current directory |
| `s` | Open **Send Modal** to select destination device for tagged files |
| `r` / `R` | Manually trigger a **Network Scan / Rescan** for LocalSend devices |
| `y` / `Enter` | **Accept** selected incoming file transfer (saves to `~/Downloads`) |
| `n` / `d` | **Decline** selected incoming file transfer |
| `Up` / `Down` / `k` / `j` | Navigate lists & file items |
| `Enter` / `l` / `Right` | Open directory / Select peer |
| `Backspace` / `h` / `Left` | Go up one directory level |
| `q` / `Ctrl+C` | Exit application |

---

## Common Hiccups & Troubleshooting

### 1. Device Discovery Issues / Mobile Phone Not Appearing

If your phone or another LocalSend app does not appear in the **`[2] Peers 📡`** tab, check the following:

#### A. Linux Firewall (UFW / FirewallD / iptables)
Linux firewalls often block incoming UDP and TCP traffic on port `53317` by default.

- **UFW (Ubuntu / Debian / Mint)**:
  ```bash
  sudo ufw allow 53317/tcp
  sudo ufw allow 53317/udp
  ```

- **FirewallD (Fedora / RHEL / Arch / CentOS)**:
  ```bash
  sudo firewall-cmd --add-port=53317/tcp --permanent
  sudo firewall-cmd --add-port=53317/udp --permanent
  sudo firewall-cmd --reload
  ```

- **iptables**:
  ```bash
  sudo iptables -A INPUT -p tcp --dport 53317 -j ACCEPT
  sudo iptables -A INPUT -p udp --dport 53317 -j ACCEPT
  ```

#### B. Multiple Network Interfaces (Docker, Tailscale, VPNs)
If Docker or Tailscale is active on your system, UDP multicast traffic may get routed to virtual network bridges (`docker0`, `tailscale0`) instead of your local Wi-Fi (`wlan0`).
- Press **`r`** inside `vib-send` to force a multi-interface subnet broadcast scan.
- Ensure both your computer and phone are connected to the same Wi-Fi access point.

#### C. Mobile App "Waiting for response..."
LocalSend requires manual confirmation by default. When a phone initiates a file transfer, `vib-send` will automatically switch to the **`[3] Receive 📥`** tab. Press **`y`** on your keyboard to accept and start the download.

---

## Building from Source

### Prerequisites
- Rust 1.80+ (`cargo` and `rustc`)

### Build & Run
```bash
# Clone the repository
git clone https://github.com/ayanchavand/vib.git
cd vib

# Check and build
cargo build --release

# Run the TUI client
./target/release/vib
```
