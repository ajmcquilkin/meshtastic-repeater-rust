use std::collections::HashSet;
use std::time::Duration;

use crate::{helpers::time::get_current_time, protobufs};

use crate::helpers::types::{CurrentTime, NodeNum, PacketId};

// I don't like these parameters not being configurable via params
pub const PACKET_HISTORY_FILL_THRESHOLD: f32 = 0.9;

#[cfg(not(test))]
pub const FLOOD_PACKET_TIMEOUT: Duration = Duration::from_secs(5 * 60); // 5 minutes
#[cfg(test)]
pub const FLOOD_PACKET_TIMEOUT: Duration = Duration::from_millis(1); // 1 ms

#[derive(Clone, Debug, Eq)]
struct PacketRecord {
    sender: NodeNum,
    id: PacketId,
    rx_time: CurrentTime,
}

impl PartialEq for PacketRecord {
    fn eq(&self, other: &Self) -> bool {
        self.sender == other.sender && self.id == other.id
    }
}

impl std::hash::Hash for PacketRecord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.sender.to_ne_bytes());
        state.write(&self.id.to_ne_bytes());
    }
}

pub struct PacketHistoryStore {
    recent_packets: HashSet<PacketRecord>,
}

impl super::PacketHistory for PacketHistoryStore {
    fn query_was_seen_recently(&mut self, packet: &protobufs::MeshPacket) -> bool {
        self.recent_packets.contains(&PacketRecord {
            sender: packet.from,
            id: packet.id,
            rx_time: get_current_time(),
        })
    }

    fn record_seen_recently(&mut self, packet: &protobufs::MeshPacket) {
        let new_record = PacketRecord {
            sender: packet.from,
            id: packet.id,
            rx_time: get_current_time(),
        };

        // Removes by id and returns whether the record matched by id and sender
        let _seen_recently = self.recent_packets.remove(&new_record);

        self.recent_packets.insert(new_record);

        if self.recent_packets.len() as f32
            > PACKET_HISTORY_FILL_THRESHOLD * self.recent_packets.capacity() as f32
        {
            self.clear_expired_packets();
        }
    }
}

impl PacketHistoryStore {
    fn clear_expired_packets(&mut self) {
        let current_time: CurrentTime = get_current_time();

        self.recent_packets
            .retain(|p| (current_time - p.rx_time) <= FLOOD_PACKET_TIMEOUT);
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use crate::router::{packet_history::PacketHistoryStore, PacketHistory};

    use super::*;

    #[test]
    fn hash_is_equal_for_same_id_and_sender() {
        let record_1 = PacketRecord {
            id: 1,
            sender: 2,
            rx_time: Instant::now(),
        };

        let record_2 = PacketRecord {
            id: 1,
            sender: 2,
            rx_time: Instant::now(),
        };

        let mut test_set = HashSet::new();
        test_set.insert(record_1);

        assert!(test_set.contains(&record_2));
    }

    #[test]
    fn hash_is_unique_for_same_id_diff_sender() {
        let record_1 = PacketRecord {
            id: 1,
            sender: 4,
            rx_time: Instant::now(),
        };

        let record_2 = PacketRecord {
            id: 1,
            sender: 2,
            rx_time: Instant::now(),
        };

        let mut test_set = HashSet::new();
        test_set.insert(record_1);

        assert!(!test_set.contains(&record_2));
    }

    #[test]
    fn hash_is_unique_for_diff_id_same_sender() {
        let record_1 = PacketRecord {
            id: 1,
            sender: 2,
            rx_time: Instant::now(),
        };

        let record_2 = PacketRecord {
            id: 4,
            sender: 2,
            rx_time: Instant::now(),
        };

        let mut test_set = HashSet::new();
        test_set.insert(record_1);

        assert!(!test_set.contains(&record_2));
    }

    #[test]
    fn packet_not_seen_returns_false() {
        let mut history = PacketHistoryStore {
            recent_packets: HashSet::new(),
        };

        let packet = protobufs::MeshPacket {
            id: 1,
            ..Default::default()
        };

        assert!(!history.query_was_seen_recently(&packet));
    }

    #[test]
    fn packet_seen_returns_true() {
        let mut history = PacketHistoryStore {
            recent_packets: HashSet::new(),
        };

        let packet = protobufs::MeshPacket {
            id: 1,
            ..Default::default()
        };

        assert!(!history.query_was_seen_recently(&packet)); // Not seen
        history.record_seen_recently(&packet);
        assert!(history.query_was_seen_recently(&packet)); // Seen
    }

    #[test]
    fn valid_record_inserted() {
        let mut history = PacketHistoryStore {
            recent_packets: HashSet::new(),
        };

        let packet = protobufs::MeshPacket {
            id: 1,
            ..Default::default()
        };

        assert!(!history.query_was_seen_recently(&packet)); // Not seen
        history.record_seen_recently(&packet);
        assert!(history.query_was_seen_recently(&packet)); // Seen
    }

    #[test]
    fn expired_packets_cleared() {
        let mut history = PacketHistoryStore {
            recent_packets: HashSet::new(),
        };

        let packet = protobufs::MeshPacket {
            id: 1,
            ..Default::default()
        };

        // Move time forward
        std::thread::sleep(FLOOD_PACKET_TIMEOUT);

        history.clear_expired_packets();

        // Should be cleared
        assert!(!history.query_was_seen_recently(&packet)); // Not seen
    }
}
