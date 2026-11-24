use crate::arch::pit;
use crate::drivers::e1000::{get_mac, is_up};
use crate::net::smoltcp_adapter::E1000Device;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::socket::dhcpv4::{self, Event as DhcpEvent};
use smoltcp::socket::dns::{self, GetQueryResultError};
use smoltcp::socket::tcp::{self, State as TcpState};
use smoltcp::socket::udp;
use smoltcp::time::Instant;
use smoltcp::wire::{
    DnsQueryType, EthernetAddress, HardwareAddress, IpAddress, IpCidr, IpEndpoint,
    Ipv4Address, Ipv4Cidr,
};
use spin::Mutex;

pub struct NetConfig {
    pub ip: Ipv4Address,
    pub subnet_mask: u8,
    pub gateway: Option<Ipv4Address>,
    pub dns_servers: Vec<Ipv4Address>,
    pub dhcp_active: bool,
    pub dhcp_configured: bool,
    pub internet_available: bool,
}

pub struct NetworkStack {
    pub interface: Interface,
    pub sockets: SocketSet<'static>,
    pub device: E1000Device,
    pub dhcp_handle: Option<SocketHandle>,
    pub config: NetConfig,
}

pub static NET_STACK: Mutex<Option<NetworkStack>> = Mutex::new(None);

pub fn now() -> Instant {
    Instant::from_millis(pit::uptime_ms() as i64)
}

pub fn init() -> bool {
    if !is_up() {
        crate::serial_println!("[net] Network hardware not up, skipping network stack init");
        return false;
    }

    let mac = get_mac();
    let hw_addr = HardwareAddress::Ethernet(EthernetAddress(mac));
    let mut config = Config::new(hw_addr);
    config.random_seed = pit::get_ticks();

    let mut device = E1000Device;
    let mut interface = Interface::new(config, &mut device, now());

    // Set fallback default IP for VirtualBox NAT (10.0.2.15/24)
    let default_ip = Ipv4Address::new(10, 0, 2, 15);
    let default_gateway = Ipv4Address::new(10, 0, 2, 2);
    let default_dns = Ipv4Address::new(10, 0, 2, 3);

    interface.update_ip_addrs(|addrs| {
        let _ = addrs.push(IpCidr::Ipv4(Ipv4Cidr::new(default_ip, 24)));
    });
    interface.routes_mut().add_default_ipv4_route(default_gateway).unwrap();

    let mut sockets = SocketSet::new(vec![]);

    // Register DHCP client
    let dhcp_socket = dhcpv4::Socket::new();
    let dhcp_handle = sockets.add(dhcp_socket);

    crate::serial_println!(
        "[net] Interface created for MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );

    let net_config = NetConfig {
        ip: default_ip,
        subnet_mask: 24,
        gateway: Some(default_gateway),
        dns_servers: vec![default_dns, Ipv4Address::new(8, 8, 8, 8)],
        dhcp_active: true,
        dhcp_configured: false,
        internet_available: false,
    };

    *NET_STACK.lock() = Some(NetworkStack {
        interface,
        sockets,
        device,
        dhcp_handle: Some(dhcp_handle),
        config: net_config,
    });

    crate::serial_println!("[net] Network stack initialized with DHCP active");
    true
}

pub fn poll() {
    let mut stack_lock = NET_STACK.lock();
    let stack = match *stack_lock {
        Some(ref mut s) => s,
        None => return,
    };

    let timestamp = now();
    stack.interface.poll(timestamp, &mut stack.device, &mut stack.sockets);

    // Poll DHCP
    if let Some(handle) = stack.dhcp_handle {
        let event = {
            let socket = stack.sockets.get_mut::<dhcpv4::Socket>(handle);
            socket.poll()
        };

        if let Some(event) = event {
            match event {
                DhcpEvent::Configured(config) => {
                    stack.config.ip = config.address.address();
                    stack.config.subnet_mask = config.address.prefix_len();
                    stack.config.gateway = config.router;
                    stack.config.dns_servers = config.dns_servers.to_vec();
                    stack.config.dhcp_configured = true;

                    // Update interface IP
                    stack.interface.update_ip_addrs(|addrs| {
                        addrs.clear();
                        let _ = addrs.push(IpCidr::Ipv4(config.address));
                    });

                    if let Some(router) = config.router {
                        stack.interface.routes_mut().add_default_ipv4_route(router).unwrap();
                    }

                    crate::serial_println!(
                        "[net] DHCP Configured: IP {}/{} Gateway {:?}",
                        stack.config.ip,
                        stack.config.subnet_mask,
                        stack.config.gateway
                    );
                }
                DhcpEvent::Deconfigured => {
                    crate::serial_println!("[net] DHCP Deconfigured");
                    stack.config.dhcp_configured = false;
                }
            }
        }
    }
}

pub fn get_info() -> Option<NetConfig> {
    let stack_lock = NET_STACK.lock();
    if let Some(ref s) = *stack_lock {
        Some(NetConfig {
            ip: s.config.ip,
            subnet_mask: s.config.subnet_mask,
            gateway: s.config.gateway,
            dns_servers: s.config.dns_servers.clone(),
            dhcp_active: s.config.dhcp_active,
            dhcp_configured: s.config.dhcp_configured,
            internet_available: s.config.internet_available,
        })
    } else {
        None
    }
}

pub fn resolve_domain(name: &str) -> Option<Ipv4Address> {
    let mut stack_lock = NET_STACK.lock();
    let stack = match *stack_lock {
        Some(ref mut s) => s,
        None => return None,
    };

    let dns_server = stack.config.dns_servers.first().copied().unwrap_or(Ipv4Address::new(8, 8, 8, 8));
    let servers = [IpAddress::Ipv4(dns_server)];

    let mut dns_socket = dns::Socket::new(&servers, vec![]);
    let query_handle = match dns_socket.start_query(stack.interface.context(), name, DnsQueryType::A) {
        Ok(q) => q,
        Err(_) => return None,
    };

    let handle = stack.sockets.add(dns_socket);

    let start = pit::uptime_ms();
    let mut result_ip = None;

    while pit::uptime_ms() - start < 3000 {
        let timestamp = now();
        stack.interface.poll(timestamp, &mut stack.device, &mut stack.sockets);

        let socket = stack.sockets.get_mut::<dns::Socket>(handle);
        match socket.get_query_result(query_handle) {
            Ok(addrs) => {
                for addr in addrs {
                    if let IpAddress::Ipv4(ipv4) = addr {
                        result_ip = Some(ipv4);
                        break;
                    }
                }
                break;
            }
            Err(GetQueryResultError::Pending) => {
                pit::sleep_ms(10);
            }
            Err(_) => break,
        }
    }

    stack.sockets.remove(handle);
    result_ip
}

pub fn http_get(ip: Ipv4Address, port: u16, host: &str, path: &str) -> Result<String, &'static str> {
    let mut stack_lock = NET_STACK.lock();
    let stack = match *stack_lock {
        Some(ref mut s) => s,
        None => return Err("Network stack not available"),
    };

    let rx_buffer = tcp::SocketBuffer::new(vec![0; 4096]);
    let tx_buffer = tcp::SocketBuffer::new(vec![0; 4096]);
    let tcp_socket = tcp::Socket::new(rx_buffer, tx_buffer);
    let handle = stack.sockets.add(tcp_socket);

    let remote_endpoint = IpEndpoint::new(IpAddress::Ipv4(ip), port);
    let local_port = 49152 + (pit::get_ticks() % 16384) as u16;

    {
        let socket = stack.sockets.get_mut::<tcp::Socket>(handle);
        if let Err(_) = socket.connect(stack.interface.context(), remote_endpoint, local_port) {
            stack.sockets.remove(handle);
            return Err("Failed to initiate TCP connection");
        }
    }

    let start = pit::uptime_ms();
    let mut connected = false;

    // Connect loop
    while pit::uptime_ms() - start < 4000 {
        let timestamp = now();
        stack.interface.poll(timestamp, &mut stack.device, &mut stack.sockets);

        let socket = stack.sockets.get_mut::<tcp::Socket>(handle);
        if socket.is_active() && socket.state() == TcpState::Established {
            connected = true;
            break;
        } else if socket.state() == TcpState::Closed {
            break;
        }
        pit::sleep_ms(10);
    }

    if !connected {
        stack.sockets.remove(handle);
        return Err("Connection timed out");
    }

    // Send HTTP GET request
    let request = alloc::format!("GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: LifeOS/1.0\r\nConnection: close\r\n\r\n", path, host);
    {
        let socket = stack.sockets.get_mut::<tcp::Socket>(handle);
        let _ = socket.send_slice(request.as_bytes());
    }

    // Receive response
    let mut response_data = Vec::new();
    let mut buf = [0u8; 1024];
    let start_rx = pit::uptime_ms();

    while pit::uptime_ms() - start_rx < 4000 {
        let timestamp = now();
        stack.interface.poll(timestamp, &mut stack.device, &mut stack.sockets);

        let socket = stack.sockets.get_mut::<tcp::Socket>(handle);
        if socket.can_recv() {
            if let Ok(n) = socket.recv_slice(&mut buf) {
                if n > 0 {
                    response_data.extend_from_slice(&buf[..n]);
                }
            }
        }

        if socket.state() == TcpState::Closed || socket.state() == TcpState::CloseWait {
            break;
        }
        pit::sleep_ms(10);
    }

    stack.sockets.remove(handle);

    if response_data.is_empty() {
        Err("Empty response from server")
    } else {
        Ok(String::from_utf8_lossy(&response_data).into_owned())
    }
}

pub fn query_host_wifi_bridge() -> Result<String, &'static str> {
    let mut stack_lock = NET_STACK.lock();
    let stack = match *stack_lock {
        Some(ref mut s) => s,
        None => return Err("Network stack not available"),
    };

    let rx_buffer = udp::PacketBuffer::new(
        vec![udp::PacketMetadata::EMPTY; 4],
        vec![0; 2048],
    );
    let tx_buffer = udp::PacketBuffer::new(
        vec![udp::PacketMetadata::EMPTY; 4],
        vec![0; 2048],
    );

    let udp_socket = udp::Socket::new(rx_buffer, tx_buffer);
    let handle = stack.sockets.add(udp_socket);

    let local_port = 50000 + (pit::get_ticks() % 1000) as u16;
    let bridge_host = stack.config.gateway.unwrap_or(Ipv4Address::new(10, 0, 2, 2));
    let bridge_endpoint = IpEndpoint::new(IpAddress::Ipv4(bridge_host), 18888);

    {
        let socket = stack.sockets.get_mut::<udp::Socket>(handle);
        if let Err(_) = socket.bind(local_port) {
            stack.sockets.remove(handle);
            return Err("UDP bind failed");
        }
        let _ = socket.send_slice(b"SCAN", bridge_endpoint);
    }

    let start = pit::uptime_ms();
    let mut response_str = None;

    while pit::uptime_ms() - start < 1500 {
        let timestamp = now();
        stack.interface.poll(timestamp, &mut stack.device, &mut stack.sockets);

        let socket = stack.sockets.get_mut::<udp::Socket>(handle);
        if socket.can_recv() {
            let mut buf = [0u8; 1500];
            if let Ok((n, _)) = socket.recv_slice(&mut buf) {
                if n > 0 {
                    response_str = Some(String::from_utf8_lossy(&buf[..n]).into_owned());
                    break;
                }
            }
        }
        pit::sleep_ms(10);
    }

    stack.sockets.remove(handle);

    match response_str {
        Some(res) => Ok(res),
        None => Err("No response from Windows Wi-Fi Bridge (Is helper running on host?)"),
    }
}
