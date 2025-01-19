#[derive(Debug, Copy, Clone, Eq, PartialEq, enum_primitive_derive::Primitive)]
pub enum MessageId {
    Serial = 0,

    HardwareVersion = 1,
    FirmwareVersion = 2,
    Reboot = 3,

    PendingFirmwareVersion = 10,
    FirmwareUploadPartChangePos = 11, // to host
    FirmwareUploadPause = 12,         // to host
    FirmwareUploadPart = 13,          // from host
    FirmwareStartUpdate = 14,         // from host
    FirmwareUploadFinished = 15,      // from host

    Battery = 50,
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::FromPrimitive;

    #[test]
    fn message_id() {
        assert_eq!(MessageId::from_u8(0), Some(MessageId::Serial));
        assert_eq!(MessageId::from_u8(200), None);
    }
}
