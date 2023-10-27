use lru::LruCache;
use rand::Rng;
use std::num::NonZeroUsize;

use crate::{helpers::types::NodeNum, protobufs};

#[derive(Debug)]
pub struct NodeDB {
    mesh_nodes: LruCache<NodeNum, protobufs::NodeInfoLite>,
    my_node_info: protobufs::MyNodeInfo,
}

impl Default for NodeDB {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeDB {
    pub fn new() -> Self {
        let random_node_id = rand::thread_rng().gen::<u32>();

        let my_node_info: protobufs::MyNodeInfo = protobufs::MyNodeInfo {
            my_node_num: random_node_id,
            ..Default::default()
        };

        NodeDB {
            mesh_nodes: LruCache::new(NonZeroUsize::new(100).expect("Could not create LRU cache")),
            my_node_info,
        }
    }

    pub fn get_node_info_by_num(&mut self, node_num: NodeNum) -> Option<&protobufs::NodeInfoLite> {
        self.mesh_nodes.get(&node_num)
    }

    pub fn update_node_from_mesh_packet(
        &mut self,
        packet: protobufs::MeshPacket,
    ) -> Result<(), String> {
        if packet.from == 0 {
            return Err("Received mesh packet with empty 'from' field, returning...".to_owned());
        }

        let payload_variant = match packet.payload_variant {
            Some(variant) => variant,
            None => {
                return Err(
                    "Received mesh packet with empty payload variant, returning...".to_owned(),
                );
            }
        };

        let data = match payload_variant {
            protobufs::mesh_packet::PayloadVariant::Decoded(dec) => dec,
            protobufs::mesh_packet::PayloadVariant::Encrypted(enc) => {
                return Err(format!(
                    "Encrypted payload variant not supported yet: {:?}",
                    enc
                ));
            }
        };

        let node_info_entry =
            self.mesh_nodes
                .get_or_insert_mut(packet.from, || protobufs::NodeInfoLite {
                    num: packet.from,
                    ..Default::default()
                });

        if packet.rx_time != 0 {
            node_info_entry.last_heard = packet.rx_time;
        }

        if packet.rx_snr != 0. {
            node_info_entry.snr = packet.rx_snr;
        }

        if data.portnum == protobufs::PortNum::NodeinfoApp.into() {
            node_info_entry.channel = packet.channel;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn updates_packet_info_correctly() {
        let mut node_db = NodeDB::new();

        let packet = protobufs::MeshPacket {
            from: 1,
            rx_time: 123,
            rx_snr: 456.,
            channel: 789,
            payload_variant: Some(protobufs::mesh_packet::PayloadVariant::Decoded(
                protobufs::Data {
                    portnum: protobufs::PortNum::NodeinfoApp.into(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };

        node_db.update_node_from_mesh_packet(packet).unwrap();

        let node_info = node_db.get_node_info_by_num(1).unwrap();

        assert_eq!(node_info.last_heard, 123);
        assert_eq!(node_info.snr, 456.);
        assert_eq!(node_info.channel, 789);
    }

    #[test]
    fn inserts_non_existant_node_num() {
        let mut node_db = NodeDB::new();

        let packet = protobufs::MeshPacket {
            from: 1,
            payload_variant: Some(protobufs::mesh_packet::PayloadVariant::Decoded(
                protobufs::Data {
                    portnum: protobufs::PortNum::NodeinfoApp.into(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };

        node_db.update_node_from_mesh_packet(packet).unwrap();

        let node_info = node_db.get_node_info_by_num(1).unwrap().to_owned();

        assert_eq!(node_db.mesh_nodes.len(), 1);
        assert_eq!(node_info.num, 1);
    }

    #[test]
    fn updates_existing_node_num() {
        let mut node_db = NodeDB::new();

        let packet = protobufs::MeshPacket {
            from: 1,
            payload_variant: Some(protobufs::mesh_packet::PayloadVariant::Decoded(
                protobufs::Data {
                    portnum: protobufs::PortNum::NodeinfoApp.into(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };

        node_db.update_node_from_mesh_packet(packet).unwrap();

        let packet = protobufs::MeshPacket {
            from: 1,
            payload_variant: Some(protobufs::mesh_packet::PayloadVariant::Decoded(
                protobufs::Data {
                    portnum: protobufs::PortNum::NodeinfoApp.into(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };

        node_db.update_node_from_mesh_packet(packet).unwrap();

        let node_info = node_db.get_node_info_by_num(1).unwrap().to_owned();

        assert_eq!(node_db.mesh_nodes.len(), 1);
        assert_eq!(node_info.num, 1);
    }

    #[test]
    fn ignores_empty_from_field() {
        let mut node_db = NodeDB::new();

        let packet = protobufs::MeshPacket {
            from: 0,
            payload_variant: Some(protobufs::mesh_packet::PayloadVariant::Decoded(
                protobufs::Data {
                    portnum: protobufs::PortNum::NodeinfoApp.into(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };

        let result = node_db.update_node_from_mesh_packet(packet);

        assert_eq!(node_db.mesh_nodes.len(), 0);
        assert!(result.is_err());
    }
}
