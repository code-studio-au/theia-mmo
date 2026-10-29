//! Bounded development ticket issuer. Keys stay in server memory.
//! Secure Renet signing key stays in server memory. Issuance is a trusted backend
//! operation and does not itself authorize a character or enforce single use.
use super::{MAX_PLAYERS, PROTOCOL_ID, Result, now};
use renet_netcode::{
    ConnectToken, NETCODE_KEY_BYTES, NETCODE_USER_DATA_BYTES, NetcodeServerTransport,
    ServerAuthentication, ServerConfig, generate_random_bytes,
};
use std::{
    fmt,
    net::{SocketAddr, UdpSocket},
};
pub struct TicketIssuer {
    key: [u8; NETCODE_KEY_BYTES],
    advertised: Vec<SocketAddr>,
}
pub struct IssuedTicket {
    bytes: Vec<u8>,
    address: SocketAddr,
}
impl fmt::Debug for IssuedTicket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IssuedTicket([redacted])")
    }
}
impl IssuedTicket {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn address(&self) -> SocketAddr {
        self.address
    }
}
pub fn secure_issuer(
    socket: UdpSocket,
    advertised: Vec<SocketAddr>,
) -> Result<(NetcodeServerTransport, TicketIssuer)> {
    if !socket.local_addr()?.ip().is_loopback()
        || advertised.is_empty()
        || advertised.len() > MAX_PLAYERS
        || advertised
            .iter()
            .any(|a| !a.ip().is_loopback() || a.port() == 0)
    {
        return Err("Development ticket issuer requires bounded loopback endpoints".into());
    }
    let key = generate_random_bytes();
    let current = now()?;
    let transport = NetcodeServerTransport::new(
        ServerConfig {
            current_time: current,
            max_clients: MAX_PLAYERS,
            protocol_id: PROTOCOL_ID,
            public_addresses: advertised.clone(),
            authentication: ServerAuthentication::Secure { private_key: key },
        },
        socket,
    )?;
    Ok((transport, TicketIssuer { key, advertised }))
}
impl TicketIssuer {
    /// Character/session checks and nonce registration must complete before this
    /// call. Only the matching transport can decrypt the authenticated user data.
    pub fn issue(
        &self,
        client_id: u64,
        data: Option<&[u8; NETCODE_USER_DATA_BYTES]>,
        seconds: u64,
    ) -> Result<IssuedTicket> {
        if !(1..=MAX_PLAYERS as u64).contains(&client_id) || !(1..=120).contains(&seconds) {
            return Err("Ticket identity or lifetime outside development bounds".into());
        }
        let current = now()?;
        current
            .as_secs()
            .checked_add(seconds)
            .ok_or("Ticket expiry overflow")?;
        let address = self.advertised[(client_id as usize - 1) % self.advertised.len()];
        let token = ConnectToken::generate(
            current,
            PROTOCOL_ID,
            seconds,
            client_id,
            5,
            vec![address],
            data,
            &self.key,
        )?;
        let mut bytes = Vec::with_capacity(2048);
        token.write(&mut bytes)?;
        if bytes.len() > 2048 {
            return Err("Signed ticket exceeds bounded size".into());
        }
        Ok(IssuedTicket { bytes, address })
    }
}
