//! Synthetic bridge acknowledgements only; actual ownership/COMMIT is qualified
//! separately in persistence. No native reader or connected owner is claimed.
use super::*;
use crate::inventory_wire::{Outcome, Reason, Reply, State};
use theia_core::{
    documents::OpenCommand,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::{Command, Operation},
};
const ITEMS: &[u8] = include_bytes!("../../../../../content/gameplay/development-items.json");
const DOCUMENTS: &[u8]=br#"{"schema":1,"id":"notes","revision":1,"item_catalog":"theia.dev.backpack-items","documents":[{"id":"note","item":"theia.dev.training-stone","title":"Orders","pages":["Orders."]}]}"#;
fn actor() -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(1).unwrap(),
        entity: EntityId::new(2).unwrap(),
    }
}
fn command(id: u128) -> OpenCommand {
    OpenCommand {
        request_id: std::num::NonZeroU128::new(id).unwrap(),
        expected_revision: 0,
        slot: 0,
        document: 0,
    }
}
fn profile() -> Profile {
    Profile::from_json(ITEMS)
        .unwrap()
        .with_documents(DOCUMENTS)
        .unwrap()
}
fn state(profile: &Profile, bag: Bag, reply: Option<Reply>) -> Vec<u8> {
    let view = State {
        actor: actor(),
        catalog_hash: profile.hash(),
        bag,
        reply,
    };
    match profile.documents() {
        Some(documents) => view.encode_with_documents(documents),
        None => view.encode(profile.catalog(), profile.hash()),
    }
    .unwrap()
}
#[tokio::test]
async fn original_document_shares_pending_order_and_approves_only_matching_success_at_same_bag_revision()
 {
    let profile = profile();
    let docs = profile.documents().unwrap().clone();
    let bag = Bag::empty(profile.catalog());
    let (commands, receiver) = mpsc::channel(1);
    let mut client = ClientState::new(Client {
        profile: profile.clone(),
        commands: receiver,
        pickups: None,
    })
    .unwrap();
    client
        .receive(&state(&profile, bag.clone(), None), actor())
        .unwrap();
    let (requests, mut queue) = mpsc::channel(1);
    let (publication, observations) = watch::channel(Observation {
        bag: Some(bag.clone()),
        ..Default::default()
    });
    let mut peer = Peer::new(Port {
        documents: Some(Arc::new(docs.clone())),
        vendor: None,
        rewards: None,
        opportunities: None,
        catalog: Arc::new(profile.catalog().clone()),
        hash: profile.hash(),
        requests,
        state: observations,
    })
    .unwrap();
    commands.try_send(command(1).into()).unwrap();
    let frame = client
        .next(actor(), tokio::time::Instant::now())
        .unwrap()
        .unwrap();
    assert!(client.approved_document.is_none());
    assert!(client.pending());
    let mut foreign = actor();
    foreign.entity = EntityId::new(9).unwrap();
    assert!(peer.request(foreign, &frame).is_err());
    assert!(queue.try_recv().is_err());
    assert!(peer.request(actor(), &frame).unwrap());
    let pending = queue.recv().await.unwrap();
    assert_eq!(pending.command, command(1).into());
    assert!(!peer.request(actor(), &frame).unwrap());
    assert!(queue.try_recv().is_err());
    commands
        .try_send(
            Command {
                expected_revision: 0,
                operation: Operation::Move { from: 0, to: 1 },
            }
            .into(),
        )
        .unwrap();
    assert!(
        client
            .next(actor(), tokio::time::Instant::now())
            .unwrap()
            .is_none()
    );
    publication.send_modify(|s| {
        s.reply = Some(Reply {
            sequence: pending.sequence,
            outcome: Outcome::Applied,
        })
    });
    let View::State(view) = peer.view(actor()).unwrap() else {
        panic!("missing view")
    };
    client
        .receive(&view.encode_with_documents(&docs).unwrap(), actor())
        .unwrap();
    peer.sent(*view);
    assert_eq!(
        client.approved_document,
        Some(crate::document_wire::Approved {
            command: command(1),
            profile_hash: docs.hash()
        })
    );
    assert!(!client.pending());
    assert_eq!(client.view.as_ref().unwrap().bag, bag);
    let movement = client
        .next(actor(), tokio::time::Instant::now())
        .unwrap()
        .unwrap();
    assert!(peer.request(actor(), &movement).unwrap());
    let pending = queue.recv().await.unwrap();
    // A normal mutation cannot use the document exception to claim same-revision success.
    assert!(
        client
            .receive(
                &state(
                    &profile,
                    bag,
                    Some(Reply {
                        sequence: pending.sequence,
                        outcome: Outcome::Applied
                    })
                ),
                actor()
            )
            .is_err()
    );
}
#[tokio::test]
async fn rejected_or_unsent_open_never_approves_and_replays_do_not_become_new_intents() {
    let profile = profile();
    let bag = Bag::empty(profile.catalog());
    let (commands, receiver) = mpsc::channel(1);
    let mut client = ClientState::new(Client {
        profile: profile.clone(),
        commands: receiver,
        pickups: None,
    })
    .unwrap();
    client
        .receive(&state(&profile, bag.clone(), None), actor())
        .unwrap();
    assert!(
        client
            .receive(
                &state(
                    &profile,
                    bag.clone(),
                    Some(Reply {
                        sequence: 1,
                        outcome: Outcome::Applied
                    })
                ),
                actor()
            )
            .is_err()
    );
    commands.try_send(command(11).into()).unwrap();
    client.next(actor(), tokio::time::Instant::now()).unwrap();
    let refused = state(
        &profile,
        bag.clone(),
        Some(Reply {
            sequence: 1,
            outcome: Outcome::Rejected(Reason::Stale),
        }),
    );
    client.receive(&refused, actor()).unwrap();
    assert!(client.approved_document.is_none());
    assert!(!client.pending());
    assert_eq!(
        client.document_reply,
        Some(crate::document_wire::Completion {
            command: command(11),
            profile_hash: profile.documents().unwrap().hash(),
            outcome: Outcome::Rejected(Reason::Stale)
        })
    );
    // Repeated observation cannot flip the original decision for the same sequence.
    client.receive(&refused, actor()).unwrap();
    assert!(client.approved_document.is_none());
    commands.try_send(command(11).into()).unwrap();
    client.next(actor(), tokio::time::Instant::now()).unwrap();
    client
        .receive(
            &state(
                &profile,
                bag,
                Some(Reply {
                    sequence: 2,
                    outcome: Outcome::Replayed,
                }),
            ),
            actor(),
        )
        .unwrap();
    assert_eq!(
        client.approved_document.unwrap().command.request_id,
        command(11).request_id
    );
}
#[tokio::test]
async fn old_profile_cannot_send_or_forward_opens_and_document_attachment_preserves_inventory_identity()
 {
    let original = Profile::from_json(ITEMS).unwrap();
    let bound = profile();
    assert_eq!(original.hash(), bound.hash());
    assert_eq!(original.catalog(), bound.catalog());
    assert!(bound.clone().with_documents(DOCUMENTS).is_err());
    let bag = Bag::empty(original.catalog());
    let (commands, receiver) = mpsc::channel(1);
    let mut client = ClientState::new(Client {
        profile: original.clone(),
        commands: receiver,
        pickups: None,
    })
    .unwrap();
    client
        .receive(&state(&original, bag.clone(), None), actor())
        .unwrap();
    commands.try_send(command(1).into()).unwrap();
    assert!(client.next(actor(), tokio::time::Instant::now()).is_err());
    let (requests, mut queue) = mpsc::channel(1);
    let (_keepalive, observations) = watch::channel(Observation {
        bag: Some(bag),
        ..Default::default()
    });
    let mut peer = Peer::new(Port {
        documents: None,
        vendor: None,
        rewards: None,
        opportunities: None,
        catalog: Arc::new(original.catalog().clone()),
        hash: original.hash(),
        requests,
        state: observations,
    })
    .unwrap();
    let frame = crate::document_wire::Request {
        actor: actor(),
        sequence: 1,
        command: command(1),
    }
    .encode(bound.documents().unwrap())
    .unwrap();
    assert!(peer.request(actor(), &frame).is_err());
    assert!(queue.try_recv().is_err());
}
