use router::PacketRouter;

pub mod helpers;
pub mod node_db;
pub mod radio_interface;
pub mod router;

pub mod protobufs {
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/meshtastic.rs"));
}

fn main() {
    let node_db = node_db::NodeDB::new();
    let mut radio_interface = radio_interface::mock_radio_interface::MockRadioInterface::default();
    let router = router::flooding_router::FloodingRouter::new(&mut radio_interface);

    // TODO do something with this
}
