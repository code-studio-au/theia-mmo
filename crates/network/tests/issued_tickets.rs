//! Actual encrypted UDP for the memory-only issuer. Transport connection is not
//! an authenticated MMO character session; the next admission policy owns that.
use renet::{RenetClient, RenetServer, ServerEvent};
use renet_netcode::ConnectToken;
use std::{net::UdpSocket, thread, time::Duration};
use theia_network::{
    renet_runtime::{now, secure_issuer},
    renet_transport,
};
#[test]
fn issued_user_data_survives_secure_transport_and_tampered_private_data_is_denied() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (mut transport, issuer) = secure_issuer(socket, vec![address]).unwrap();
    let data = [91; renet_netcode::NETCODE_USER_DATA_BYTES];
    let issued = issuer.issue(1, Some(&data), 20).unwrap();
    assert_eq!(issued.address(), address);
    assert!(issued.bytes().len() <= 2048);
    assert_eq!(format!("{issued:?}"), "IssuedTicket([redacted])");
    let token = ConnectToken::read(&mut std::io::Cursor::new(issued.bytes())).unwrap();
    let bad = issuer
        .issue(2, Some(&[92; renet_netcode::NETCODE_USER_DATA_BYTES]), 20)
        .unwrap();
    let mut bad = ConnectToken::read(&mut std::io::Cursor::new(bad.bytes())).unwrap();
    bad.private_data[0] ^= 1;
    let mut client = RenetClient::new(renet_transport::connection_config());
    let mut client_transport = renet_transport::secure_client(
        UdpSocket::bind("127.0.0.1:0").unwrap(),
        now().unwrap(),
        token,
    )
    .unwrap();
    let mut invalid = RenetClient::new(renet_transport::connection_config());
    let mut invalid_transport = renet_transport::secure_client(
        UdpSocket::bind("127.0.0.1:0").unwrap(),
        now().unwrap(),
        bad,
    )
    .unwrap();
    let mut server = RenetServer::new(renet_transport::connection_config());
    let mut admitted = Vec::new();
    for _ in 0..200 {
        let delta = Duration::from_millis(10);
        client.update(delta);
        client_transport.update(delta, &mut client).unwrap();
        client_transport.send_packets(&mut client).unwrap();
        invalid.update(delta);
        let _ = invalid_transport.update(delta, &mut invalid);
        let _ = invalid_transport.send_packets(&mut invalid);
        server.update(delta);
        transport.update(delta, &mut server).unwrap();
        while let Some(event) = server.get_event() {
            if let ServerEvent::ClientConnected { client_id } = event {
                admitted.push(client_id);
            }
        }
        transport.send_packets(&mut server);
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(admitted, vec![1]);
    assert_eq!(transport.user_data(1), Some(data));
    assert_eq!(transport.user_data(2), None);
    transport.disconnect_all(&mut server);
}
#[test]
fn issuer_limits_endpoints_client_ids_and_lifetimes_without_remote_io() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    assert!(secure_issuer(socket, vec!["203.0.113.1:1234".parse().unwrap()]).is_err());
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let (_transport, issuer) = secure_issuer(socket, vec![address]).unwrap();
    for (id, lifetime) in [(0, 20), (9, 20), (1, 0), (1, 121)] {
        assert!(issuer.issue(id, None, lifetime).is_err());
    }
}
