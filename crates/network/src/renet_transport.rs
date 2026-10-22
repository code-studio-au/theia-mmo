//! Secure Renet channels and transport constructors used by the default launcher.
use renet::{ChannelConfig, ConnectionConfig, SendType};
use renet_netcode::{
    ClientAuthentication, ConnectToken, NETCODE_KEY_BYTES, NetcodeClientTransport,
    NetcodeServerTransport, ServerAuthentication, ServerConfig,
};
use std::{net::UdpSocket, time::Duration};

pub const MOVEMENT_CHANNEL: u8 = 0;
pub const EVENT_CHANNEL: u8 = 1;
pub const CHANNEL_MEMORY_BYTES: usize = 8192;
pub const PROTOCOL_ID: u64 = 0x5448_4549_4100_0000 | crate::wire::VERSION as u64;

/// Explicit bounded channels; reliable events have priority over transient poses.
pub fn connection_config() -> ConnectionConfig {
    let channels = vec![
        ChannelConfig {
            channel_id: EVENT_CHANNEL,
            max_memory_usage_bytes: CHANNEL_MEMORY_BYTES,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(100),
            },
        },
        ChannelConfig {
            channel_id: MOVEMENT_CHANNEL,
            max_memory_usage_bytes: CHANNEL_MEMORY_BYTES,
            send_type: SendType::Unreliable,
        },
    ];
    ConnectionConfig {
        available_bytes_per_tick: 2400,
        server_channels_config: channels.clone(),
        client_channels_config: channels,
    }
}

/// Server key stays with the issuer/server; it is never supplied to clients.
pub fn secure_server(
    socket: UdpSocket,
    now: Duration,
    private_key: [u8; NETCODE_KEY_BYTES],
) -> crate::Result<NetcodeServerTransport> {
    let address = socket.local_addr()?;
    let config = ServerConfig {
        current_time: now,
        max_clients: crate::wire::MAX_PLAYERS,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![address],
        authentication: ServerAuthentication::Secure { private_key },
    };
    Ok(NetcodeServerTransport::new(config, socket)?)
}

pub fn secure_client(
    socket: UdpSocket,
    now: Duration,
    token: ConnectToken,
) -> crate::Result<NetcodeClientTransport> {
    Ok(NetcodeClientTransport::new(
        now,
        ClientAuthentication::Secure {
            connect_token: token,
        },
        socket,
    )?)
}
