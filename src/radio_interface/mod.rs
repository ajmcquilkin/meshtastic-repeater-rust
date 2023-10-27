pub mod mock_radio_interface;

use crate::protobufs;

pub trait RadioInterface {
    fn init(&mut self) -> Result<(), String>;
    fn send(&mut self, packet: protobufs::MeshPacket) -> Result<(), String>;
}
