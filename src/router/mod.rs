use crate::{protobufs, radio_interface::RadioInterface};

pub mod flooding_router;
pub mod packet_history;

pub trait PacketRouter<'a> {
    fn new(interface: &'a mut dyn RadioInterface) -> Self; // ? Why not just take ownership?
    fn send(&mut self, packet: protobufs::MeshPacket) -> Result<(), String>;
}

pub trait PacketHistory {
    fn query_was_seen_recently(&mut self, packet: &protobufs::MeshPacket) -> bool;
    fn record_seen_recently(&mut self, packet: &protobufs::MeshPacket);
}
