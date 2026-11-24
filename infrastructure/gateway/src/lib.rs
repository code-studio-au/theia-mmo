use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub mod aws;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

pub const IDLE_SECONDS: u64 = 300;
pub const START_SECONDS: u64 = 300;
pub const RESERVATION_SECONDS: u64 = 60;
pub const HEARTBEAT_SECONDS: u64 = 40;
pub const PUBLISH_SECONDS: u64 = 15;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_secs()
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub server: String,
    pub launcher_min: String,
    pub client: String,
    pub protocol: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    pub launcher: String,
    pub client: String,
    pub protocol: String,
    pub content: String,
}

fn version(s: &str) -> Option<[u32; 3]> {
    let mut parts = s.split('.');
    let result = [
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ];
    parts.next().is_none().then_some(result)
}

impl Release {
    pub fn valid(&self) -> bool {
        let safe = |s: &str| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b".-_".contains(&c))
        };
        version(&self.launcher_min).is_some()
            && [&self.server, &self.client, &self.protocol, &self.content]
                .iter()
                .all(|s| safe(s))
    }

    pub fn accepts(&self, build: &Build) -> bool {
        version(&build.launcher)
            .zip(version(&self.launcher_min))
            .is_some_and(|(installed, minimum)| installed >= minimum)
            && self.client == build.client
            && self.protocol == build.protocol
            && self.content == build.content
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instance {
    Stopped,
    Pending,
    Running,
    Stopping,
    Other,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Offline,
    Starting,
    Initialising,
    Ready,
    ShuttingDown,
    Maintenance,
    Error,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Reservation {
    pub subject: String,
    pub expires: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Host {
    pub boot: String,
    pub heartbeat: u64,
    pub ready: bool,
    pub occupied: bool,
    #[serde(default)]
    pub players: usize,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Realm {
    pub release: Option<Release>,
    pub host: Option<Host>,
    pub reservations: BTreeMap<String, Reservation>,
    pub start_lock: u64,
    pub start_deadline: u64,
    pub stopping: Option<String>,
    pub idle_since: Option<u64>,
    pub maintenance: bool,
    pub keep_alive_until: u64,
    pub failed: bool,
}

impl Realm {
    pub fn phase(&self, instance: Instance, now: u64) -> Phase {
        if self.maintenance {
            return Phase::Maintenance;
        }
        match instance {
            Instance::Stopped if self.start_lock > now => Phase::Starting,
            Instance::Stopped => Phase::Offline,
            Instance::Stopping => Phase::ShuttingDown,
            Instance::Other => Phase::Error,
            _ if self.failed => Phase::Error,
            Instance::Pending => Phase::Starting,
            Instance::Running if self.stopping.is_some() => Phase::ShuttingDown,
            Instance::Running
                if self.host.as_ref().is_some_and(|h| {
                    h.ready && h.heartbeat.saturating_add(HEARTBEAT_SECONDS) > now
                }) =>
            {
                Phase::Ready
            }
            Instance::Running if self.start_deadline != 0 && self.start_deadline <= now => {
                Phase::Error
            }
            Instance::Running => Phase::Initialising,
        }
    }

    pub fn reserve(&mut self, id: &str, subject: &str, now: u64) -> bool {
        self.expire_reservations(now);
        if self
            .reservations
            .get(id)
            .is_some_and(|r| r.subject != subject)
            || (!self.reservations.contains_key(id) && self.reservations.len() >= 8)
        {
            return false;
        }
        self.reservations.insert(
            id.into(),
            Reservation {
                subject: subject.into(),
                expires: now + RESERVATION_SECONDS,
            },
        );
        self.idle_since = None;
        true
    }

    fn expire_reservations(&mut self, now: u64) {
        self.reservations
            .retain(|_, reservation| reservation.expires > now);
    }

    pub fn release(&mut self, id: &str, subject: &str) {
        if self
            .reservations
            .get(id)
            .is_some_and(|r| r.subject == subject)
        {
            self.reservations.remove(id);
        }
    }

    pub fn claim_start(&mut self, instance: Instance, now: u64) -> bool {
        if instance != Instance::Stopped || self.maintenance || self.start_lock > now {
            return false;
        }
        self.host = None;
        self.stopping = None;
        self.failed = false;
        self.start_lock = now + 30;
        self.start_deadline = now + START_SECONDS;
        true
    }

    pub fn should_report(&self, host: &Host, now: u64) -> bool {
        self.host.as_ref().is_none_or(|previous| {
            previous.boot != host.boot
                || previous.ready != host.ready
                || previous.occupied != host.occupied
                || previous.players != host.players
                || now.saturating_sub(previous.heartbeat) >= PUBLISH_SECONDS
        }) || self
            .reservations
            .values()
            .any(|reservation| reservation.expires <= now)
            || (self.idle_eligible(host, now)
                && self
                    .idle_since
                    .is_none_or(|since| now.saturating_sub(since) >= IDLE_SECONDS))
    }

    fn idle_eligible(&self, host: &Host, now: u64) -> bool {
        host.ready
            && !host.occupied
            && !self.maintenance
            && !self.failed
            && self.keep_alive_until <= now
            && self.reservations.is_empty()
    }

    pub fn report(&mut self, host: Host, now: u64) -> bool {
        self.expire_reservations(now);
        if self.host.as_ref().is_none_or(|old| old.boot != host.boot) {
            self.idle_since = None;
            self.stopping = None;
        } else if self
            .host
            .as_ref()
            .is_some_and(|old| old.heartbeat.saturating_add(HEARTBEAT_SECONDS) <= now)
        {
            self.idle_since = None;
        }
        let eligible = self.idle_eligible(&host, now);
        self.host = Some(host);
        if !eligible {
            self.idle_since = None;
            return false;
        }
        let since = *self.idle_since.get_or_insert(now);
        if now.saturating_sub(since) < IDLE_SECONDS || self.stopping.is_some() {
            return false;
        }
        self.stopping = self.host.as_ref().map(|h| h.boot.clone());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(at: u64, occupied: bool) -> Host {
        Host {
            boot: "boot".into(),
            heartbeat: at,
            ready: true,
            occupied,
            players: usize::from(occupied),
        }
    }

    #[test]
    fn join_cancels_idle_and_only_owner_can_release() {
        let mut realm = Realm::default();
        assert!(!realm.report(host(100, false), 100));
        assert!(realm.reserve("attempt", "tester", 398));
        realm.release("attempt", "other");
        assert!(!realm.report(host(400, false), 400));
        realm.release("attempt", "tester");
        assert!(!realm.report(host(401, false), 401));
        for at in (410..701).step_by(10) {
            assert!(!realm.report(host(at, false), at));
        }
        assert!(realm.report(host(701, false), 701));
        assert_eq!(realm.phase(Instance::Running, 701), Phase::ShuttingDown);
    }

    #[test]
    fn idle_time_does_not_survive_missing_observations_or_maintenance() {
        let mut realm = Realm::default();
        assert!(!realm.report(host(10, false), 10));
        assert!(!realm.report(host(311, false), 311));
        assert_eq!(realm.idle_since, Some(311));
        realm.keep_alive_until = 700;
        assert!(!realm.report(host(610, false), 610));
        assert_eq!(realm.idle_since, None);
        realm.maintenance = true;
        assert!(!realm.report(host(710, false), 710));
        assert_eq!(realm.phase(Instance::Running, 710), Phase::Maintenance);
    }

    #[test]
    fn unchanged_observations_skip_writes_but_transitions_publish() {
        let mut realm = Realm::default();
        assert!(realm.should_report(&host(100, false), 100));
        realm.report(host(100, false), 100);
        assert!(!realm.should_report(&host(104, false), 104));
        assert!(realm.should_report(&host(104, true), 104));
        assert!(realm.should_report(&host(115, false), 115));
        realm.host = Some(host(396, false));
        assert!(realm.should_report(&host(400, false), 400));
    }

    #[test]
    fn heartbeat_prunes_expired_reservations_without_removing_live_joins() {
        let mut realm = Realm::default();
        assert!(realm.reserve("abandoned", "one", 10));
        assert!(realm.reserve("roster", "two", 40));
        assert!(!realm.report(host(70, false), 70));
        assert!(!realm.reservations.contains_key("abandoned"));
        assert!(realm.reservations.contains_key("roster"));
        assert_eq!(realm.idle_since, None);
        assert!(!realm.report(host(100, false), 100));
        assert!(realm.reservations.is_empty());
        assert_eq!(realm.idle_since, Some(100));
    }

    #[test]
    fn release_matches_only_the_deployed_client_protocol_and_content() {
        let release = Release {
            server: "server-a".into(),
            launcher_min: "0.1.7".into(),
            client: "0.1.0.12".into(),
            protocol: "PN02".into(),
            content: "millhaven-a".into(),
        };
        let mut build = Build {
            launcher: "0.1.10".into(),
            client: release.client.clone(),
            protocol: release.protocol.clone(),
            content: release.content.clone(),
        };
        assert!(release.valid() && release.accepts(&build));
        build.content = "millhaven-b".into();
        assert!(!release.accepts(&build));
        build.content = release.content.clone();
        build.launcher = "0.1.6".into();
        assert!(!release.accepts(&build));
    }

    #[test]
    fn readiness_expires_and_boots_are_serialized() {
        let mut realm = Realm::default();
        assert!(realm.claim_start(Instance::Stopped, 1));
        assert!(!realm.claim_start(Instance::Stopped, 2));
        assert!(!realm.claim_start(Instance::Pending, 100));
        assert!(!realm.report(host(110, true), 110));
        assert_eq!(realm.phase(Instance::Running, 149), Phase::Ready);
        assert_eq!(realm.phase(Instance::Running, 150), Phase::Initialising);
        assert_eq!(realm.phase(Instance::Running, 301), Phase::Error);
    }
}
