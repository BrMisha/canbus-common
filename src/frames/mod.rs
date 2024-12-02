use crate::message::MessageId;

pub mod helpers;
pub mod firmware;
pub mod serial;
pub mod version;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type<D, R>

{
    Data(D),
    Request(R),
}

impl<D, R> Type<D, R> {
    pub fn from_slice<'a>(is_request: bool, data: &'a[u8]) -> Option<Self>
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
            Type::Request(data) =>  Some((data.copy_into_slice(dst)?, true)),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Empty;

impl TryFrom<&[u8]> for Empty {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Empty)
    }
}

impl helpers::CopyIntoSlice for Empty {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize> {
        Some(0)
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Frame {
    Serial(Type<serial::Serial, Empty>),
    HardwareVersion(Type<version::Version, Empty>),
    FirmwareVersion(Type<version::Version, Empty>),
    PendingFirmwareVersion(Type<version::Version, Empty>),
    FirmwareUploadPartChangePos(Type<firmware::UploadPartChangePos, Empty>),
    FirmwareUploadPause(Type<bool, Empty>),
    FirmwareUploadPart(Type<firmware::UploadPart, Empty>),
    FirmwareStartUpdate,
    FirmwareUploadFinished,
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

impl Frame {
    pub fn parse_frame(frame_id: MessageId, data: &[u8], is_request: bool) -> Result<Frame, ParseError> {
        match frame_id {
            MessageId::Serial => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Frame::Serial(t))
            },
            MessageId::HardwareVersion => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Frame::HardwareVersion(t))
            },
            MessageId::FirmwareVersion => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Frame::FirmwareVersion(t))
            }
            MessageId::PendingFirmwareVersion => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Frame::PendingFirmwareVersion(t))
            },
            MessageId::FirmwareUploadPartChangePos => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Frame::FirmwareUploadPartChangePos(t))
            },
            MessageId::FirmwareUploadPause => {
                match is_request {
                    false => {
                        let v = *(data.get(0).ok_or(ParseError::WrongDataSize)?) != 0;
                        Ok(Frame::FirmwareUploadPause(Type::Data(v)))
                    }
                    true => Err(ParseError::RemoteFrame),
                }
            },
            MessageId::FirmwareUploadPart => {
                let t = Type::from_slice(is_request, data).ok_or(ParseError::WrongData)?;
                Ok(Frame::FirmwareUploadPart(t))
            },
            MessageId::FirmwareStartUpdate => match is_request {
                true => Err(ParseError::RemoteFrame),
                false => Ok(Frame::FirmwareStartUpdate),
            },
            MessageId::FirmwareUploadFinished => match is_request {
                true => Err(ParseError::RemoteFrame),
                false => Ok(Frame::FirmwareUploadFinished),
            },
        }
    }


    pub fn frame_into_slise(&self, dst: &mut [u8]) -> Option<(usize, bool)> {
        match self {
            Frame::Serial(v) =>  v.into_slice(dst),
            Frame::HardwareVersion(v) => v.into_slice(dst),
            Frame::FirmwareVersion(v) =>  v.into_slice(dst),
            Frame::PendingFirmwareVersion(v) => v.into_slice(dst),
            Frame::FirmwareUploadPartChangePos(v) => v.into_slice(dst),
            Frame::FirmwareUploadPause(v) => v.into_slice(dst),
            Frame::FirmwareUploadPart(v) => v.into_slice(dst),
            Frame::FirmwareStartUpdate => Some((0, false)),
            Frame::FirmwareUploadFinished => Some((0, false)),
        }
    }

    #[inline]
    pub fn id(&self) -> MessageId {
        match self {
            Frame::Serial(_) => MessageId::Serial,
            Frame::HardwareVersion(_) => MessageId::HardwareVersion,
            Frame::FirmwareVersion(_) => MessageId::FirmwareVersion,
            Frame::PendingFirmwareVersion(_) => MessageId::PendingFirmwareVersion,
            Frame::FirmwareUploadPartChangePos(_) => MessageId::FirmwareUploadPartChangePos,
            Frame::FirmwareUploadPause(_) => MessageId::FirmwareUploadPause,
            Frame::FirmwareUploadPart(_) => MessageId::FirmwareUploadPart,
            Frame::FirmwareStartUpdate => MessageId::FirmwareStartUpdate,
            Frame::FirmwareUploadFinished => MessageId::FirmwareUploadFinished,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageId;

    #[test]
    fn serial() {
        assert_eq!(
            Frame::parse_frame(MessageId::Serial, &[], true),
            Ok(Frame::Serial(Type::Request(Empty)))
        );

        assert_eq!(
            Frame::Serial(Type::Request(Empty)).frame_into_slise(&mut [5,5]),
            Some((0, true))
        );

        assert_eq!(
            Frame::parse_frame(MessageId::Serial, &[1, 2, 3, 4, 5], false),
            Ok(Frame::Serial(Type::Data(serial::Serial::from([1, 2, 3, 4, 5]))))
        );

        let mut buf = [5; 50];
        let r = Frame::Serial(Type::Data(serial::Serial::from([1, 2, 3, 4, 5]))).frame_into_slise(&mut buf).unwrap();
        assert_eq!(
            (5usize, false, [1u8, 2, 3, 4, 5].as_slice()),
            (r.0, r.1, &buf[..5])
        );
    }

    #[test]
    fn version() {
        fn none(id: MessageId, res: Frame) {
            let mut buf = [5; 50];
            let (size, is_request) = res.frame_into_slise(&mut buf).unwrap();
            assert_eq!(size, 0);
            assert_eq!(is_request, true);

            assert_eq!(Frame::parse_frame(id, &buf, true), Ok(res));
        }
        none(MessageId::FirmwareVersion, Frame::FirmwareVersion(Type::Request(Empty)));
        none(MessageId::HardwareVersion, Frame::HardwareVersion(Type::Request(Empty)));
        none(MessageId::PendingFirmwareVersion, Frame::PendingFirmwareVersion(Type::Request(Empty)));

        fn data(v: version::Version, id: MessageId, res: Frame) {
            assert_eq!(
                Frame::parse_frame(id, &<[u8; 8]>::from(v), false),
                Ok(res.clone())
            );

            let mut buf = [5; 50];
            let (size, is_request) = res.frame_into_slise(&mut buf).unwrap();
            assert_eq!(size, 8);
            assert_eq!(is_request, false);
            assert_eq!(buf[0..size], <[u8; 8]>::from(v));
        }
        let v = version::Version {
            major: 1,
            minor: 2,
            path: 3,
            build: 9864,
        };
        data(v, MessageId::FirmwareVersion, Frame::FirmwareVersion(Type::Data(v)));
        data(v, MessageId::HardwareVersion, Frame::HardwareVersion(Type::Data(v)));
        data(v, MessageId::PendingFirmwareVersion, Frame::PendingFirmwareVersion(Type::Data(v)));
    }

    #[test]
    fn firmware_upload_part_change_pos() {
        assert_eq!(
            Frame::parse_frame(
                MessageId::FirmwareUploadPartChangePos,
                &[0x01, 0x02, 0x03],
                false
            ),
            Ok(Frame::FirmwareUploadPartChangePos(
                Type::Data(firmware::UploadPartChangePos::new(0x010203usize).unwrap())
            ))
        );

        let mut buf = [5; 50];
        let (size, is_request) = Frame::FirmwareUploadPartChangePos(
            Type::Data(firmware::UploadPartChangePos::new(0x010203usize).unwrap())
        ).frame_into_slise(&mut buf).unwrap();

        assert_eq!(is_request, false);
        assert_eq!(buf.get(..size).unwrap(), &[0x01, 0x02, 0x03]);

    }

    #[test]
    fn firmware_upload_pause() {
        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareUploadPause, [1u8].as_slice(), true),
            Err(ParseError::RemoteFrame)
        );

        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareUploadPause, [1u8].as_slice(), false),
            Ok(Frame::FirmwareUploadPause(Type::Data(true)))
        );

        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareUploadPause, [0u8].as_slice(), false),
            Ok(Frame::FirmwareUploadPause(Type::Data(false)))
        );


        let mut buf = [0; 10];
        let (size, is_request) = Frame::FirmwareUploadPause(Type::Data(false)).frame_into_slise(&mut buf).unwrap();
        assert_eq!(is_request, false);
        assert_eq!(buf[0..size].as_ref(), [0u8,].as_ref());

        let (size, is_request) = Frame::FirmwareUploadPause(Type::Data(true)).frame_into_slise(&mut buf).unwrap();
        assert_eq!(is_request, false);
        assert_eq!(buf[0..size].as_ref(), [1u8,].as_ref());
    }

    #[test]
    fn firmware_upload_part() {
        assert_eq!(
            Frame::parse_frame(
                MessageId::FirmwareUploadPart,
                &[0x01, 0x02, 0x03, 1, 2, 3, 4],
                false
            ),
            Err(ParseError::WrongData)
        );

        assert_eq!(
            Frame::parse_frame(
                MessageId::FirmwareUploadPart,
                &[0x01, 0x02, 0x03, 1, 2, 3, 4, 5],
                false
            ),
            Ok(Frame::FirmwareUploadPart(
                Type::Data(firmware::UploadPart::new(0x010203usize, [1, 2, 3, 4, 5]).unwrap())
            ))
        );

        let mut buf = [0; 10];
        let (size, is_request) = Frame::FirmwareUploadPart(
            Type::Data(firmware::UploadPart::new(0x010203usize, [1, 2, 3, 4, 5]).unwrap())
        ).frame_into_slise(&mut buf).unwrap();
        assert_eq!(is_request, false);
        assert_eq!(buf[..size].as_ref(), [0x01, 0x02, 0x03, 1, 2, 3, 4, 5].as_ref());
    }

    #[test]
    fn firmware_start_update() {
        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareStartUpdate, [1, 1].as_ref(), true),
            Err(ParseError::RemoteFrame)
        );

        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareStartUpdate, &[], false),
            Ok(Frame::FirmwareStartUpdate)
        );

        let mut buf = [0; 10];
        let (size, is_request) = Frame::FirmwareStartUpdate.frame_into_slise(&mut buf).unwrap();
        assert_eq!(is_request, false);
        assert_eq!(buf[..size].as_ref(), &[]);
    }
}