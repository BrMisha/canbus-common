use num_traits::FromPrimitive;
use num_traits::ToPrimitive;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SubId(pub u16);

impl SubId {
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }

    pub fn split(&self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

impl From<SubId> for [u8; 2] {
    fn from(v: SubId) -> Self {
        v.split()
    }
}

impl From<[u8; 2]> for SubId {
    fn from(v: [u8; 2]) -> Self {
        Self(u16::from_be_bytes(v))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, enum_primitive_derive::Primitive)]
pub enum MessageId {
    Serial = 0,
    /*Type = 1,
    HardwareVersion = 2,
    FirmwareVersion = 3,

    FirmwareUpgrade = 4,

    ModuleBatteryManagerUnit = 20,*/

    /*
    PendingFirmwareVersion = 10,
    FirmwareUploadPartChangePos = 11, // to host
    FirmwareUploadPause = 8026,         // to host
    FirmwareUploadPart = 8028,          // from host
    FirmwareUploadFinished = 8029,         // from host
    FirmwareStartUpdate = 8030,         // from host*/
}

#[cfg(test)]
mod tests {
    //use crate::frames::Frame;
    use super::*;

    #[test]
    fn message_id() {
        assert_eq!(MessageId::from_u8(0), Some(MessageId::Serial));
        assert_eq!(MessageId::from_u8(200), None);
    }
}
