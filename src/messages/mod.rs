use crate::message_id::MessageId;

pub mod battery;
pub mod firmware;
pub mod helpers;
pub mod serial;
pub mod version;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type<D, R> {
    Data(D),
    Request(R),
}

impl<D, R> Type<D, R> {
    pub fn from_slice<'a>(is_request: bool, data: &'a [u8]) -> Option<Self>
    where
        D: TryFrom<&'a [u8]>,
        R: TryFrom<&'a [u8]>,
    {
        match is_request {
            false => Some(Type::Data(D::try_from(data).ok()?)),
            true => Some(Type::Request(R::try_from(data).ok()?)),
        }
    }

    pub fn into_slice(&self, dst: &mut [u8]) -> Option<(usize, bool)>
    where
        D: helpers::CopyIntoSlice,
        R: helpers::CopyIntoSlice,
    {
        match self {
            Type::Data(data) => Some((data.copy_into_slice(dst)?, false)),
            Type::Request(data) => Some((data.copy_into_slice(dst)?, true)),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Empty;

impl TryFrom<&[u8]> for Empty {
    type Error = ();
    fn try_from(_value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Empty)
    }
}

impl helpers::CopyIntoSlice for Empty {
    fn copy_into_slice(&self, _dst: &mut [u8]) -> Option<usize> {
        Some(0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Message {
    Serial(Type<serial::Serial, Empty>),
    HardwareVersion(Type<version::Version, Empty>),
    FirmwareVersion(Type<version::Version, Empty>),
    PendingFirmwareVersion(Type<version::Version, Empty>),
    FirmwareUploadPartChangePos(Type<firmware::UploadPartChangePos, Empty>),
    FirmwareUploadPause(Type<bool, Empty>),
    FirmwareUploadPart(Type<firmware::UploadPart, Empty>),
    FirmwareStartUpdate,
    FirmwareUploadFinished,
    Battery(Type<battery::Battery, Empty>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ParseError {
    WrongDataSize,
    WrongData,
    UnknownId,
    RemoteFrame,
    RemovedWrongDlc,
    Other,
}

impl Message {
    pub fn parse_message(
        message_id: MessageId,
        data: &[u8],
        is_request: bool,
    ) -> Result<Message, ParseError> {
        match message_id {
            MessageId::Serial => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Message::Serial(t))
            }
            MessageId::HardwareVersion => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Message::HardwareVersion(t))
            }
            MessageId::FirmwareVersion => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Message::FirmwareVersion(t))
            }
            MessageId::PendingFirmwareVersion => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Message::PendingFirmwareVersion(t))
            }
            MessageId::FirmwareUploadPartChangePos => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Message::FirmwareUploadPartChangePos(t))
            }
            MessageId::FirmwareUploadPause => match is_request {
                false => {
                    let v = *(data.first().ok_or(ParseError::WrongDataSize)?) != 0;
                    Ok(Message::FirmwareUploadPause(Type::Data(v)))
                }
                true => Err(ParseError::RemoteFrame),
            },
            MessageId::FirmwareUploadPart => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Message::FirmwareUploadPart(t))
            }
            MessageId::FirmwareStartUpdate => match is_request {
                true => Err(ParseError::RemoteFrame),
                false => Ok(Message::FirmwareStartUpdate),
            },
            MessageId::FirmwareUploadFinished => match is_request {
                true => Err(ParseError::RemoteFrame),
                false => Ok(Message::FirmwareUploadFinished),
            },
            MessageId::Battery => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Message::Battery(t))
            }
        }
    }

    pub fn message_into_slise(&self, dst: &mut [u8]) -> Option<(usize, bool)> {
        match self {
            Message::Serial(v) => v.into_slice(dst),
            Message::HardwareVersion(v) => v.into_slice(dst),
            Message::FirmwareVersion(v) => v.into_slice(dst),
            Message::PendingFirmwareVersion(v) => v.into_slice(dst),
            Message::FirmwareUploadPartChangePos(v) => v.into_slice(dst),
            Message::FirmwareUploadPause(v) => v.into_slice(dst),
            Message::FirmwareUploadPart(v) => v.into_slice(dst),
            Message::FirmwareStartUpdate => Some((0, false)),
            Message::FirmwareUploadFinished => Some((0, false)),
            Message::Battery(v) => v.into_slice(dst),
        }
    }

    #[inline]
    pub fn id(&self) -> MessageId {
        match self {
            Message::Serial(_) => MessageId::Serial,
            Message::HardwareVersion(_) => MessageId::HardwareVersion,
            Message::FirmwareVersion(_) => MessageId::FirmwareVersion,
            Message::PendingFirmwareVersion(_) => MessageId::PendingFirmwareVersion,
            Message::FirmwareUploadPartChangePos(_) => MessageId::FirmwareUploadPartChangePos,
            Message::FirmwareUploadPause(_) => MessageId::FirmwareUploadPause,
            Message::FirmwareUploadPart(_) => MessageId::FirmwareUploadPart,
            Message::FirmwareStartUpdate => MessageId::FirmwareStartUpdate,
            Message::FirmwareUploadFinished => MessageId::FirmwareUploadFinished,
            Message::Battery(_) => MessageId::Battery,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_id::MessageId;

    #[test]
    fn serial() {
        assert_eq!(
            Message::parse_message(MessageId::Serial, &[], true),
            Ok(Message::Serial(Type::Request(Empty)))
        );

        assert_eq!(
            Message::Serial(Type::Request(Empty)).message_into_slise(&mut [5, 5]),
            Some((0, true))
        );

        assert_eq!(
            Message::parse_message(MessageId::Serial, &[1, 2, 3, 4, 5], false),
            Ok(Message::Serial(Type::Data(serial::Serial::from([
                1, 2, 3, 4, 5
            ]))))
        );

        let mut buf = [5; 50];
        let r = Message::Serial(Type::Data(serial::Serial::from([1, 2, 3, 4, 5])))
            .message_into_slise(&mut buf)
            .unwrap();
        assert_eq!(
            (5usize, false, [1u8, 2, 3, 4, 5].as_slice()),
            (r.0, r.1, &buf[..5])
        );
    }

    #[test]
    fn version() {
        fn none(id: MessageId, res: Message) {
            let mut buf = [5; 50];
            let (size, is_request) = res.message_into_slise(&mut buf).unwrap();
            assert_eq!(size, 0);
            assert!(is_request);

            assert_eq!(Message::parse_message(id, &buf, true), Ok(res));
        }
        none(
            MessageId::FirmwareVersion,
            Message::FirmwareVersion(Type::Request(Empty)),
        );
        none(
            MessageId::HardwareVersion,
            Message::HardwareVersion(Type::Request(Empty)),
        );
        none(
            MessageId::PendingFirmwareVersion,
            Message::PendingFirmwareVersion(Type::Request(Empty)),
        );

        fn data(v: version::Version, id: MessageId, res: Message) {
            assert_eq!(
                Message::parse_message(id, &<[u8; 8]>::from(v), false),
                Ok(res.clone())
            );

            let mut buf = [5; 50];
            let (size, is_request) = res.message_into_slise(&mut buf).unwrap();
            assert_eq!(size, 8);
            assert!(!is_request);
            assert_eq!(buf[0..size], <[u8; 8]>::from(v));
        }
        let v = version::Version {
            major: 1,
            minor: 2,
            path: 3,
            build: 9864,
        };
        data(
            v,
            MessageId::FirmwareVersion,
            Message::FirmwareVersion(Type::Data(v)),
        );
        data(
            v,
            MessageId::HardwareVersion,
            Message::HardwareVersion(Type::Data(v)),
        );
        data(
            v,
            MessageId::PendingFirmwareVersion,
            Message::PendingFirmwareVersion(Type::Data(v)),
        );
    }

    #[test]
    fn firmware_upload_part_change_pos() {
        assert_eq!(
            Message::parse_message(
                MessageId::FirmwareUploadPartChangePos,
                &[0x01, 0x02, 0x03],
                false
            ),
            Ok(Message::FirmwareUploadPartChangePos(Type::Data(
                firmware::UploadPartChangePos::new(0x010203usize).unwrap()
            )))
        );

        let mut buf = [5; 50];
        let (size, is_request) = Message::FirmwareUploadPartChangePos(Type::Data(
            firmware::UploadPartChangePos::new(0x010203usize).unwrap(),
        ))
        .message_into_slise(&mut buf)
        .unwrap();

        assert!(!is_request);
        assert_eq!(buf.get(..size).unwrap(), &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn firmware_upload_pause() {
        assert_eq!(
            Message::parse_message(MessageId::FirmwareUploadPause, [1u8].as_slice(), true),
            Err(ParseError::RemoteFrame)
        );

        assert_eq!(
            Message::parse_message(MessageId::FirmwareUploadPause, [1u8].as_slice(), false),
            Ok(Message::FirmwareUploadPause(Type::Data(true)))
        );

        assert_eq!(
            Message::parse_message(MessageId::FirmwareUploadPause, [0u8].as_slice(), false),
            Ok(Message::FirmwareUploadPause(Type::Data(false)))
        );

        let mut buf = [0; 10];
        let (size, is_request) = Message::FirmwareUploadPause(Type::Data(false))
            .message_into_slise(&mut buf)
            .unwrap();
        assert!(!is_request);
        assert_eq!(buf[0..size].as_ref(), [0u8,].as_ref());

        let (size, is_request) = Message::FirmwareUploadPause(Type::Data(true))
            .message_into_slise(&mut buf)
            .unwrap();
        assert!(!is_request);
        assert_eq!(buf[0..size].as_ref(), [1u8,].as_ref());
    }

    #[test]
    fn firmware_upload_part() {
        assert_eq!(
            Message::parse_message(
                MessageId::FirmwareUploadPart,
                &[0x01, 0x02, 0x03, 1, 2, 3, 4],
                false
            ),
            Err(ParseError::WrongData)
        );

        assert_eq!(
            Message::parse_message(
                MessageId::FirmwareUploadPart,
                &[0x01, 0x02, 0x03, 1, 2, 3, 4, 5],
                false
            ),
            Ok(Message::FirmwareUploadPart(Type::Data(
                firmware::UploadPart::new(0x010203usize, [1, 2, 3, 4, 5]).unwrap()
            )))
        );

        let mut buf = [0; 10];
        let (size, is_request) = Message::FirmwareUploadPart(Type::Data(
            firmware::UploadPart::new(0x010203usize, [1, 2, 3, 4, 5]).unwrap(),
        ))
        .message_into_slise(&mut buf)
        .unwrap();
        assert!(!is_request);
        assert_eq!(
            buf[..size].as_ref(),
            [0x01, 0x02, 0x03, 1, 2, 3, 4, 5].as_ref()
        );
    }

    #[test]
    fn firmware_start_update() {
        assert_eq!(
            Message::parse_message(MessageId::FirmwareStartUpdate, [1, 1].as_ref(), true),
            Err(ParseError::RemoteFrame)
        );

        assert_eq!(
            Message::parse_message(MessageId::FirmwareStartUpdate, &[], false),
            Ok(Message::FirmwareStartUpdate)
        );

        let mut buf = [0; 10];
        let (size, is_request) = Message::FirmwareStartUpdate
            .message_into_slise(&mut buf)
            .unwrap();
        assert!(!is_request);
        assert_eq!(buf[..size].as_ref(), &[]);
    }

    #[test]
    fn battery() {
        assert_eq!(
            Message::parse_message(MessageId::Battery, &[], true),
            Ok(Message::Battery(Type::Request(Empty)))
        );

        assert_eq!(
            Message::Battery(Type::Request(Empty)).message_into_slise(&mut [5, 5]),
            Some((0, true))
        );

        assert_eq!(
            Message::parse_message(MessageId::Battery, &[1, 2, 3, 4, 5], false),
            Ok(Message::Battery(Type::Data(battery::Battery::from([
                1, 2, 3, 4, 5
            ]))))
        );

        let mut buf = [5; 50];
        let r = Message::Battery(Type::Data(battery::Battery::from([1, 2, 3, 4, 5])))
            .message_into_slise(&mut buf)
            .unwrap();
        assert_eq!(
            (5usize, false, [1u8, 2, 3, 4, 5].as_slice()),
            (r.0, r.1, &buf[..5])
        );
    }
}
