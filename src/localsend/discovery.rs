use crate::events::AppEvent;
use crate::localsend::protocol::{
    DeviceType, LOCALSEND_DEFAULT_PORT, LOCALSEND_MULTICAST_ADDR, PROTOCOL_VERSION, Peer,
    RegisterDto,
};
use reqwest::Client;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{self, Duration};

pub struct DiscoveryEngine {
    alias: String,
    fingerprint: String,
    port: u16,
    event_tx: UnboundedSender<AppEvent>,
    http_client: Client,
}

impl DiscoveryEngine {
    pub fn new(
        alias: String,
        fingerprint: String,
        port: u16,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Self {
        // LocalSend v2 protocol uses self-signed TLS certificates for P2P transfers
        let http_client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            alias,
            fingerprint,
            port,
            event_tx,
            http_client,
        }
    }

    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bind_addr: SocketAddr = format!("0.0.0.0:{}", self.port).parse()?;

        // Configure socket with SO_REUSEADDR
        let socket2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket2.set_reuse_address(true)?;
        socket2.set_nonblocking(true)?;
        socket2.set_broadcast(true)?;
        socket2.bind(&bind_addr.into())?;

        let std_socket: std::net::UdpSocket = socket2.into();
        let multicast_addr: Ipv4Addr = LOCALSEND_MULTICAST_ADDR.parse()?;

        // Join multicast on unspecified interface
        let _ = std_socket.join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED);

        // Join multicast explicitly on ALL active local IPv4 interface IPs (e.g. wlan0 10.199.210.67, eth0)
        let local_ips = get_local_v4_ips();
        for ip in &local_ips {
            let _ = std_socket.join_multicast_v4(&multicast_addr, ip);
        }

        let socket = Arc::new(UdpSocket::from_std(std_socket)?);

        // Spawn periodic announcement task
        let engine_clone = self.clone();
        let socket_send = socket.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                if let Err(e) = engine_clone.announce(&socket_send).await {
                    eprintln!("Discovery announce error: {}", e);
                }
            }
        });

        // Spawn UDP listener task
        let mut buf = [0u8; 65535];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src_addr)) => {
                    let data = &buf[..len];
                    if let Ok(msg) = serde_json::from_slice::<RegisterDto>(data) {
                        if msg.fingerprint == self.fingerprint {
                            continue;
                        }

                        let peer_port = msg.port.unwrap_or(LOCALSEND_DEFAULT_PORT);
                        let peer_protocol = msg.protocol.unwrap_or_else(|| "https".to_string());
                        let alias = if msg.alias.is_empty() {
                            format!("Device ({})", src_addr.ip())
                        } else {
                            msg.alias.clone()
                        };
                        let fingerprint = if msg.fingerprint.is_empty() {
                            format!("{}:{}", src_addr.ip(), peer_port)
                        } else {
                            msg.fingerprint.clone()
                        };

                        let peer = Peer {
                            alias,
                            version: msg.version.clone(),
                            device_model: msg.device_model.clone(),
                            device_type: msg.device_type.clone(),
                            fingerprint,
                            ip: src_addr.ip().to_string(),
                            port: peer_port,
                            protocol: peer_protocol.clone(),
                        };
                        let _ = self.event_tx.send(AppEvent::PeerDiscovered(peer));

                        // Spec Section 3.1: Reply via HTTP POST /api/localsend/v2/register
                        if msg.announce == Some(true) {
                            let engine_reply = self.clone();
                            let target_ip = src_addr.ip().to_string();
                            tokio::spawn(async move {
                                engine_reply
                                    .reply_register(&target_ip, peer_port, &peer_protocol)
                                    .await;
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("UDP receive error: {}", e);
                    time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }

    pub async fn announce(
        &self,
        socket: &UdpSocket,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let register_msg = RegisterDto {
            alias: self.alias.clone(),
            version: PROTOCOL_VERSION.to_string(),
            device_model: Some("vib TUI Client".to_string()),
            device_type: Some(DeviceType::Desktop),
            fingerprint: self.fingerprint.clone(),
            port: Some(self.port),
            protocol: Some("https".to_string()),
            download: Some(true),
            announce: Some(true),
        };

        let json_bytes = serde_json::to_vec(&register_msg)?;

        let multicast_target: SocketAddr =
            format!("{}:{}", LOCALSEND_MULTICAST_ADDR, LOCALSEND_DEFAULT_PORT).parse()?;
        let broadcast_target: SocketAddr =
            format!("255.255.255.255:{}", LOCALSEND_DEFAULT_PORT).parse()?;

        let _ = socket.send_to(&json_bytes, multicast_target).await;
        let _ = socket.send_to(&json_bytes, broadcast_target).await;

        // Also broadcast out on subnet directed broadcast for each local IPv4 interface (e.g. 10.199.210.255)
        let local_ips = get_local_v4_ips();
        for ip in local_ips {
            let octets = ip.octets();
            let subnet_bcast: SocketAddr = format!(
                "{}.{}.{}.255:{}",
                octets[0], octets[1], octets[2], LOCALSEND_DEFAULT_PORT
            )
            .parse()
            .unwrap_or(broadcast_target);
            let _ = socket.send_to(&json_bytes, subnet_bcast).await;
        }

        Ok(())
    }

    pub async fn reply_register(&self, target_ip: &str, target_port: u16, target_protocol: &str) {
        let register_msg = RegisterDto {
            alias: self.alias.clone(),
            version: PROTOCOL_VERSION.to_string(),
            device_model: Some("vib TUI Client".to_string()),
            device_type: Some(DeviceType::Desktop),
            fingerprint: self.fingerprint.clone(),
            port: Some(self.port),
            protocol: Some("https".to_string()),
            download: Some(true),
            announce: Some(false),
        };

        let url = format!(
            "{}://{}:{}/api/localsend/v2/register",
            target_protocol, target_ip, target_port
        );
        let _ = self.http_client.post(&url).json(&register_msg).send().await;
    }
}

#[cfg(unix)]
pub fn get_local_v4_ips() -> Vec<Ipv4Addr> {
    let mut ips = Vec::new();
    unsafe {
        let mut ifap: *mut libc::ifaddrs = std::ptr::null_mut();
        if libc::getifaddrs(&mut ifap) == 0 && !ifap.is_null() {
            let mut curr = ifap;
            while !curr.is_null() {
                let addr = (*curr).ifa_addr;
                if !addr.is_null() && (*addr).sa_family as i32 == libc::AF_INET {
                    let sockaddr_in = addr as *const libc::sockaddr_in;
                    let ip_bytes = (*sockaddr_in).sin_addr.s_addr.to_ne_bytes();
                    let ip = Ipv4Addr::from(ip_bytes);
                    if !ip.is_loopback() {
                        ips.push(ip);
                    }
                }
                curr = (*curr).ifa_next;
            }
            libc::freeifaddrs(ifap);
        }
    }
    ips
}

#[cfg(not(unix))]
pub fn get_local_v4_ips() -> Vec<Ipv4Addr> {
    let mut ips = Vec::new();
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                if let std::net::IpAddr::V4(ipv4) = local_addr.ip() {
                    if !ipv4.is_loopback() {
                        ips.push(ipv4);
                    }
                }
            }
        }
    }
    ips
}
