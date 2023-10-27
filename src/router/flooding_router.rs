use crate::radio_interface::RadioInterface;

use super::PacketRouter;

pub struct FloodingRouter<'a> {
    radio_interface: &'a mut dyn RadioInterface,
}

impl<'a> PacketRouter<'a> for FloodingRouter<'a> {
    fn new(radio_interface: &'a mut dyn RadioInterface) -> Self {
        FloodingRouter { radio_interface }
    }

    fn send(&mut self, packet: crate::protobufs::MeshPacket) -> Result<(), String> {
        // Currently we will just relay the packet, but in the future
        // we will need to handle things like encryption, duty cycle, etc...

        self.radio_interface.send(packet)?;

        Ok(())
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use crate::{protobufs, radio_interface::mock_radio_interface::MockRadioInterface};

    use super::*;

    #[test]
    fn calls_send_on_interface() {
        let mut mock_interface = MockRadioInterface::default();
        let mut router = FloodingRouter::new(&mut mock_interface);

        let packet = protobufs::MeshPacket::default();
        router.send(packet).unwrap();

        assert!(mock_interface.send_called);
    }
}
