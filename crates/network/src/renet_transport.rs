//! Secure Renet channels and transport constructors used by the default launcher.
use renet::{ChannelConfig, ConnectionConfig, SendType};
use renet_netcode::{
    ClientAuthentication, ConnectToken, NETCODE_KEY_BYTES, NetcodeClientTransport,
    NetcodeServerTransport, ServerAuthentication, ServerConfig,
};
use std::{net::UdpSocket, time::Duration};

pub const MOVEMENT_CHANNEL: u8 = 0;
pub const EVENT_CHANNEL: u8 = 1;
pub const CRITICAL_PRESENTATION_CHANNEL: u8 = 2;
pub const CREATURE_ATTACK_CHANNEL: u8 = CRITICAL_PRESENTATION_CHANNEL;
pub const CHANNEL_MEMORY_BYTES: usize = 8192;
/// Revision 1 requires an encrypted reliable transport confirmation before
/// World admission. Keep the movement/Quinn wire version independent.
pub const TRANSPORT_REVISION: u8 = 1;
pub const PROTOCOL_ID: u64 =
    0x5448_4549_4100_0000 | ((TRANSPORT_REVISION as u64) << 16) | crate::wire::VERSION as u64;
/// No gameplay entitlement: proves that the peer completed netcode's handshake.
pub const CONNECTION_CONFIRMATION: &[u8; 4] = b"NC01";

/// Call after netcode update and before sending Renet packets. The marker is
/// reliable and sent once; it carries no account, actor or gameplay command.
pub fn confirm_connection(client: &mut renet::RenetClient, sent: &mut bool) -> crate::Result<()> {
    if client.is_connected() && !*sent {
        if !client.can_send_message(EVENT_CHANNEL, CONNECTION_CONFIRMATION.len()) {
            return Err("Transport confirmation queue unavailable".into());
        }
        client.send_message(EVENT_CHANNEL, CONNECTION_CONFIRMATION.as_slice());
        *sent = true;
    }
    Ok(())
}

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

/// Explicit opt-in critical timing channel. Renet sends channels in vector order.
/// Legacy peers keep the original two-channel configuration.
pub fn connection_config_with_public_presentation() -> ConnectionConfig {
    let mut config = connection_config();
    let critical = ChannelConfig {
        channel_id: CREATURE_ATTACK_CHANNEL,
        max_memory_usage_bytes: CHANNEL_MEMORY_BYTES,
        send_type: SendType::ReliableOrdered {
            resend_time: Duration::from_millis(100),
        },
    };
    config.server_channels_config.insert(0, critical.clone());
    config.client_channels_config.insert(0, critical);
    config
}

/// Compatibility entry point for the original public NPC timing mode.
pub fn connection_config_with_public_attacks() -> ConnectionConfig {
    connection_config_with_public_presentation()
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
