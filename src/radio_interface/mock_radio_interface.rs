use super::RadioInterface;

#[derive(Debug, Default)]
pub struct MockRadioInterface {
    pub init_called: bool,
    pub send_called: bool,
}

impl RadioInterface for MockRadioInterface {
    fn init(&mut self) -> Result<(), String> {
        self.init_called = true;
        Ok(())
    }

    fn send(&mut self, _packet: crate::protobufs::MeshPacket) -> Result<(), String> {
        self.send_called = true;
        Ok(())
    }
}
