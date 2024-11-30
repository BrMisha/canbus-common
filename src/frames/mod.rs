use crate::message::MessageId;
use crate::frames::firmware::{UploadPart, UploadPartChangePos};
use crate::frames::Type::{Data, Remote};

/*pub mod dyn_id;
pub mod firmware;
pub mod serial;
pub mod version;
pub mod battery;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type<T> {
    Data(T),
    Remote,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Frame {
    Serial(Type<serial::Serial>),
    DynId(dyn_id::Data),
    HardwareVersion(Type<version::Version>),
    FirmwareVersion(Type<version::Version>),
    PendingFirmwareVersion(Type<Option<version::Version>>),
    FirmwareUploadPartChangePos(firmware::UploadPartChangePos),
    FirmwareUploadPause(bool),
    FirmwareUploadPart(firmware::UploadPart),
    FirmwareUploadFinished,
    FirmwareStartUpdate,

    BatteryVoltage(Type<Option<u32>>),
    BatteryCurrent(Type<Option<i32>>),
    BatteryVoltageCurrent(Type<Option<(u32, i32)>>),
    BatteryCellCount(Type<u16>),
    BatteryCellsStates(Type<battery::CellsStates>),
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ParserType<'a> {
    Data(&'a [u8]),
    Remote(u8),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RawType {
    Data(arrayvec::ArrayVec<u8, 8>),
    Remote(u8),
}

impl RawType {
    pub fn new_data<T: IntoIterator<Item = u8>>(array: T) -> Self {
        let mut t = arrayvec::ArrayVec::<u8, 8>::new();
        t.extend(array);
        Self::Data(t)
    }
}

impl Frame {
    pub fn parse_frame(frame_id: MessageId, data: ParserType) -> Result<Frame, ParseError> {
        match frame_id {
            MessageId::Serial => match data {
                ParserType::Remote(len) => match len {
                    5 => Ok(Frame::Serial(Remote)),
                    _l => Err(ParseError::RemovedWrongDlc),
                },
                ParserType::Data(data) => match data.len() {
                    5 => Ok(Frame::Serial(Data(serial::Serial::from(
                        <[u8; 5]>::try_from(&data[0..5]).unwrap(),
                    )))),
                    _ => Err(ParseError::WrongDataSize),
                },
            },
            MessageId::DynId => match data {
                ParserType::Remote(_) => Err(ParseError::RemoteFrame),
                ParserType::Data(data) => match data.len() {
                    6 => Ok(Frame::DynId(dyn_id::Data::from(
                        <[u8; 6]>::try_from(&data[0..6]).unwrap(),
                    ))),
                    _ => Err(ParseError::WrongDataSize),
                },
            },
            n @ MessageId::HardwareVersion | n @ MessageId::FirmwareVersion => {
                fn put(n: MessageId, v: Type<version::Version>) -> Frame {
                    match n {
                        MessageId::HardwareVersion => Frame::HardwareVersion(v),
                        MessageId::FirmwareVersion => Frame::FirmwareVersion(v),
                        _ => unreachable!(),
                    }
                }
                match data {
                    ParserType::Remote(len) => match len {
                        8 => Ok(put(n, Remote)),
                        _ => Err(ParseError::RemovedWrongDlc),
                    },
                    ParserType::Data(data) => match data.len() {
                        8 => Ok(put(
                            n,
                            Data(version::Version::from(
                                <[u8; 8]>::try_from(&data[..8]).unwrap(),
                            )),
                        )),
                        _ => Err(ParseError::RemovedWrongDlc),
                    },
                }
            }
            MessageId::PendingFirmwareVersion => match data {
                ParserType::Remote(len) => match len {
                    8 => Ok(Frame::PendingFirmwareVersion(Remote)),
                    _ => Err(ParseError::RemovedWrongDlc),
                },
                ParserType::Data(data) => match data.len() {
                    0 => Ok(Frame::PendingFirmwareVersion(Data(None))),
                    8 => Ok(Frame::PendingFirmwareVersion(Data(Some(
                        version::Version::from(<[u8; 8]>::try_from(&data[..8]).unwrap()),
                    )))),
                    _ => Err(ParseError::RemovedWrongDlc),
                },
            },
            MessageId::FirmwareUploadPartChangePos => match data {
                ParserType::Remote(_) => Err(ParseError::RemoteFrame),
                ParserType::Data(data) => match data.len() {
                    3 => Ok(Frame::FirmwareUploadPartChangePos(
                        UploadPartChangePos::from(<[u8; 3]>::try_from(&data[..3]).unwrap()),
                    )),
                    _ => Err(ParseError::WrongDataSize),
                },
            },
            MessageId::FirmwareUploadPart => match data {
                ParserType::Remote(_) => Err(ParseError::RemoteFrame),
                ParserType::Data(data) => match data.len() {
                    8 => Ok(Frame::FirmwareUploadPart(UploadPart::from(
                        <[u8; 8]>::try_from(&data[..8]).unwrap(),
                    ))),
                    _ => Err(ParseError::WrongDataSize),
                },
            },
            MessageId::FirmwareStartUpdate => match data {
                ParserType::Remote(_) => Err(ParseError::RemoteFrame),
                ParserType::Data(_) => Ok(Frame::FirmwareStartUpdate),
            },
            MessageId::FirmwareUploadPause => match data {
                ParserType::Remote(_) => Err(ParseError::RemoteFrame),
                ParserType::Data(data) => match data.len() {
                    1 => Ok(Frame::FirmwareUploadPause(data[0] != 0)),
                    _ => Err(ParseError::WrongDataSize),
                },
            },
            MessageId::FirmwareUploadFinished => match data {
                ParserType::Remote(_) => Err(ParseError::RemoteFrame),
                ParserType::Data(_) => Ok(Frame::FirmwareUploadFinished),
            },

            MessageId::BatteryVoltage => match data {
                ParserType::Data(data) => match data.len() {
                    4 => Ok(Frame::BatteryVoltage(Data(Some(u32::from_be_bytes(<[u8; 4]>::try_from(&data[..4]).unwrap()))))),
                    0 => Ok(Frame::BatteryVoltage(Data(None))),
                    _ => Err(ParseError::WrongDataSize),
                }
                ParserType::Remote(len) => match len {
                    4 => Ok(Frame::BatteryVoltage(Remote)),
                    _ => Err(ParseError::RemovedWrongDlc),
                },
            }
            MessageId::BatteryCurrent => match data {
                ParserType::Data(data) => match data.len() {
                    4 => Ok(Frame::BatteryCurrent(Data(Some(i32::from_be_bytes(<[u8; 4]>::try_from(&data[..4]).unwrap()))))),
                    0 => Ok(Frame::BatteryCurrent(Data(None))),
                    _ => Err(ParseError::WrongDataSize),
                }
                ParserType::Remote(len) => match len {
                    4 => Ok(Frame::BatteryCurrent(Remote)),
                    _ => Err(ParseError::RemovedWrongDlc),
                },
            }
            MessageId::BatteryVoltageCurrent => match data {
                ParserType::Data(data) => match data.len() {
                    8 => Ok(Frame::BatteryVoltageCurrent(Data(
                        Some((
                            u32::from_be_bytes(<[u8; 4]>::try_from(&data[..4]).unwrap()),
                            i32::from_be_bytes(<[u8; 4]>::try_from(&data[4..]).unwrap())
                        ))
                    ))),
                    0 => Ok(Frame::BatteryVoltageCurrent(Data(None))),
                    _ => Err(ParseError::WrongDataSize),
                }
                ParserType::Remote(len) => match len {
                    8 => Ok(Frame::BatteryVoltageCurrent(Remote)),
                    _ => Err(ParseError::RemovedWrongDlc),
                },
            }
            MessageId::BatteryCellCount => match data {
                ParserType::Data(data) => match data.len() {
                    2 => Ok(Frame::BatteryCellCount(Data(u16::from_be_bytes(<[u8; 2]>::try_from(&data[..2]).unwrap())))),
                    _ => Err(ParseError::WrongDataSize),
                }
                ParserType::Remote(len) => match len {
                    2 => Ok(Frame::BatteryCellCount(Remote)),
                    _ => Err(ParseError::RemovedWrongDlc),
                },
            }
            MessageId::BatteryCellsStates => match data {
                ParserType::Data(data) => battery::CellsStates::try_from(data)
                    .map(|v| Frame::BatteryCellsStates(Data(v)))
                    .map_err(|_| ParseError::WrongData),
                ParserType::Remote(_len) => Ok(Frame::BatteryCellsStates(Remote)),
            }
        }
    }

    pub fn raw_frame(&self) -> (MessageId, RawType) {
        match self {
            Frame::Serial(v) => match v {
                Remote => (MessageId::Serial, RawType::Remote(5)),
                Data(v) => (MessageId::Serial, RawType::new_data(v.0)),
            },
            Frame::DynId(v) => (MessageId::DynId, RawType::new_data(<[u8; 6]>::from(*v))),
            n @ Frame::HardwareVersion(v) | n @ Frame::FirmwareVersion(v) => {
                let id = match n {
                    Frame::HardwareVersion(_) => MessageId::HardwareVersion,
                    Frame::FirmwareVersion(_) => MessageId::FirmwareVersion,
                    _ => unreachable!(),
                };
                match v {
                    Remote => (id, RawType::Remote(8)),
                    Data(v) => (id, RawType::new_data(<[u8; 8]>::from(*v))),
                }
            }
            Frame::PendingFirmwareVersion(v) => (
                MessageId::PendingFirmwareVersion,
                match v {
                    Remote => RawType::Remote(8),
                    Data(Some(v)) => RawType::new_data(<[u8; 8]>::from(*v)),
                    Data(None) => RawType::new_data([0_u8; 0]),
                },
            ),
            Frame::FirmwareUploadPartChangePos(v) => (
                MessageId::FirmwareUploadPartChangePos,
                RawType::new_data(<[u8; 3]>::from(*v)),
            ),
            Frame::FirmwareUploadPause(v) => (
                MessageId::FirmwareUploadPause,
                RawType::new_data([u8::from(*v)]),
            ),
            Frame::FirmwareUploadPart(v) => (
                MessageId::FirmwareUploadPart,
                RawType::new_data(<[u8; 8]>::from(*v)),
            ),
            Frame::FirmwareStartUpdate => (MessageId::FirmwareStartUpdate, RawType::new_data([])),
            Frame::FirmwareUploadFinished => {
                (MessageId::FirmwareUploadFinished, RawType::new_data([]))
            },

            Frame::BatteryVoltage(v) => {
                (MessageId::BatteryVoltage, match v {
                    Remote => RawType::Remote(4),
                    Data(Some(v)) => RawType::new_data(<[u8; 4]>::from(v.to_be_bytes())),
                    Data(None) => RawType::new_data([0_u8; 0]),
                })
            }
            Frame::BatteryVoltageCurrent(v) => {
                (MessageId::BatteryVoltage, match v {
                    Remote => RawType::Remote(8),
                    Data(Some(v)) => {
                        let mut r: [u8; 8] = Default::default();
                        r[..4].clone_from_slice(&<[u8; 4]>::from(v.0.to_be_bytes()));
                        r[4..].clone_from_slice(&<[u8; 4]>::from(v.1.to_be_bytes()));
                        RawType::new_data(r)
                    },
                    Data(None) => RawType::new_data([0_u8; 0]),
                })
            }
            Frame::BatteryCurrent(v) => {
                (MessageId::BatteryCurrent, match v {
                    Remote => RawType::Remote(4),
                    Data(Some(v)) => RawType::new_data(<[u8; 4]>::from(v.to_be_bytes())),
                    Data(None) => RawType::new_data([0_u8; 0]),
                })
            }
            Frame::BatteryCellCount(v) => {
                (MessageId::BatteryCellCount, match v {
                    Remote => RawType::Remote(2),
                    Data(v) => RawType::new_data(<[u8; 2]>::from(v.to_be_bytes())),
                })
            }
            Frame::BatteryCellsStates(v) => {
                (MessageId::BatteryCellsStates, match v {
                    Remote => RawType::Remote(0),
                    Data(v) => RawType::new_data(arrayvec::ArrayVec::<u8, 8>::from(v)),
                })
            }
        }
    }

    #[inline]
    pub fn id(&self) -> MessageId {
        match self {
            Frame::Serial(_) => MessageId::Serial,
            Frame::DynId(_) => MessageId::DynId,
            Frame::HardwareVersion(_) => MessageId::HardwareVersion,
            Frame::FirmwareVersion(_) => MessageId::FirmwareVersion,
            Frame::PendingFirmwareVersion(_) => MessageId::PendingFirmwareVersion,
            Frame::FirmwareUploadPartChangePos(_) => MessageId::FirmwareUploadPartChangePos,
            Frame::FirmwareUploadPause(_) => MessageId::FirmwareUploadPause,
            Frame::FirmwareUploadPart(_) => MessageId::FirmwareUploadPart,
            Frame::FirmwareStartUpdate => MessageId::FirmwareStartUpdate,
            Frame::FirmwareUploadFinished => MessageId::FirmwareUploadFinished,

            Frame::BatteryVoltage(_) => MessageId::BatteryVoltage,
            Frame::BatteryCurrent(_) => MessageId::BatteryCurrent,
            Frame::BatteryVoltageCurrent(_) => MessageId::BatteryVoltageCurrent,
            Frame::BatteryCellCount(_) => MessageId::BatteryCellCount,
            Frame::BatteryCellsStates(_) => MessageId::BatteryCellsStates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageId;
    use crate::frames::ParserType;

    #[test]
    fn serial() {
        assert_eq!(
            Frame::parse_frame(MessageId::Serial, ParserType::Remote(5),),
            Ok(Frame::Serial(Remote))
        );

        assert_eq!(
            Frame::Serial(Remote).raw_frame(),
            (MessageId::Serial, RawType::Remote(5))
        );

        assert_eq!(
            Frame::parse_frame(MessageId::Serial, ParserType::Data(&[1, 2, 3, 4, 5])),
            Ok(Frame::Serial(Data(serial::Serial::from([1, 2, 3, 4, 5]))))
        );

        assert_eq!(
            Frame::Serial(Data(serial::Serial::from([1, 2, 3, 4, 5]))).raw_frame(),
            (MessageId::Serial, RawType::new_data([1, 2, 3, 4, 5]))
        );
    }

    #[test]
    fn dyn_id() {
        assert_eq!(
            Frame::parse_frame(MessageId::DynId, ParserType::Remote(0),),
            Err(ParseError::RemoteFrame)
        );

        assert_eq!(
            Frame::parse_frame(MessageId::DynId, ParserType::Data(&[1, 2, 3, 4, 5, 80]),),
            Ok(Frame::DynId(dyn_id::Data::new(
                serial::Serial::from([1, 2, 3, 4, 5]),
                80
            )))
        );

        assert_eq!(
            Frame::DynId(dyn_id::Data::new(serial::Serial::from([1, 2, 3, 4, 5]), 55)).raw_frame(),
            (MessageId::DynId, RawType::new_data([1, 2, 3, 4, 5, 55]))
        );
    }

    #[test]
    fn version() {
        fn none(id: MessageId, res: Frame) {
            assert_eq!(Frame::parse_frame(id, ParserType::Remote(8)), Ok(res.clone()));
            assert_eq!(res.raw_frame(), (id, RawType::Remote(8)));
        }
        none(MessageId::FirmwareVersion, Frame::FirmwareVersion(Remote));
        none(MessageId::HardwareVersion, Frame::HardwareVersion(Remote));
        none(
            MessageId::PendingFirmwareVersion,
            Frame::PendingFirmwareVersion(Remote),
        );

        fn data(v: version::Version, id: MessageId, res: Frame) {
            assert_eq!(
                Frame::parse_frame(id, ParserType::Data(&<[u8; 8]>::from(v))),
                Ok(res.clone())
            );

            assert_eq!(res.raw_frame(), (id, RawType::new_data(<[u8; 8]>::from(v))));
        }
        let v = version::Version {
            major: 1,
            minor: 2,
            path: 3,
            build: 9864,
        };
        data(v, MessageId::FirmwareVersion, Frame::FirmwareVersion(Data(v)));
        data(v, MessageId::HardwareVersion, Frame::HardwareVersion(Data(v)));
        data(
            v,
            MessageId::PendingFirmwareVersion,
            Frame::PendingFirmwareVersion(Data(Some(v))),
        );

        assert_eq!(
            Frame::parse_frame(
                MessageId::PendingFirmwareVersion,
                ParserType::Data(&[0_u8; 0])
            ),
            Ok(Frame::PendingFirmwareVersion(Data(None)))
        );

        assert_eq!(
            Frame::PendingFirmwareVersion(Data(Some(v))).raw_frame(),
            (
                MessageId::PendingFirmwareVersion,
                RawType::new_data(<[u8; 8]>::from(v))
            )
        );
    }

    #[test]
    fn firmware_upload_part_change_pos() {
        assert_eq!(
            Frame::parse_frame(MessageId::DynId, ParserType::Data(&[1, 2, 3, 4]),),
            Err(ParseError::WrongDataSize)
        );

        assert_eq!(
            Frame::parse_frame(
                MessageId::FirmwareUploadPartChangePos,
                ParserType::Data(&[0x01, 0x02, 0x03]),
            ),
            Ok(Frame::FirmwareUploadPartChangePos(
                firmware::UploadPartChangePos::new(0x010203usize).unwrap()
            ))
        );

        assert_eq!(
            Frame::FirmwareUploadPartChangePos(
                firmware::UploadPartChangePos::new(0x010203usize).unwrap()
            )
            .raw_frame(),
            (
                MessageId::FirmwareUploadPartChangePos,
                RawType::new_data([0x01, 0x02, 0x03])
            )
        );
    }

    #[test]
    fn firmware_upload_part() {
        assert_eq!(
            Frame::parse_frame(
                MessageId::FirmwareUploadPart,
                ParserType::Data(&[0x01, 0x02, 0x03, 1, 2, 3, 4])
            ),
            Err(ParseError::WrongDataSize)
        );

        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareUploadPart, ParserType::Remote(1)),
            Err(ParseError::RemoteFrame)
        );

        assert_eq!(
            Frame::parse_frame(
                MessageId::FirmwareUploadPart,
                ParserType::Data(&[0x01, 0x02, 0x03, 1, 2, 3, 4, 5])
            ),
            Ok(Frame::FirmwareUploadPart(
                firmware::UploadPart::new(0x010203usize, [1, 2, 3, 4, 5]).unwrap()
            ))
        );

        assert_eq!(
            Frame::FirmwareUploadPart(
                firmware::UploadPart::new(0x010203usize, [1, 2, 3, 4, 5]).unwrap()
            )
            .raw_frame(),
            (
                MessageId::FirmwareUploadPart,
                RawType::new_data([0x01, 0x02, 0x03, 1, 2, 3, 4, 5])
            )
        );
    }

    #[test]
    fn firmware_start_update() {
        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareStartUpdate, ParserType::Remote(1)),
            Err(ParseError::RemoteFrame)
        );

        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareStartUpdate, ParserType::Data(&[])),
            Ok(Frame::FirmwareStartUpdate)
        );

        assert_eq!(
            Frame::FirmwareStartUpdate.raw_frame(),
            (MessageId::FirmwareStartUpdate, RawType::new_data([]))
        );
    }

    #[test]
    fn firmware_upload_pause() {
        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareUploadPause, ParserType::Remote(1)),
            Err(ParseError::RemoteFrame)
        );

        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareUploadPause, ParserType::Data(&[1])),
            Ok(Frame::FirmwareUploadPause(true))
        );

        assert_eq!(
            Frame::parse_frame(MessageId::FirmwareUploadPause, ParserType::Data(&[0])),
            Ok(Frame::FirmwareUploadPause(false))
        );

        assert_eq!(
            Frame::FirmwareUploadPause(false).raw_frame(),
            (MessageId::FirmwareUploadPause, RawType::new_data([0]))
        );

        assert_eq!(
            Frame::FirmwareUploadPause(true).raw_frame(),
            (MessageId::FirmwareUploadPause, RawType::new_data([1]))
        );
    }

    #[test]
    fn battery() {
        {
            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryVoltage,
                    ParserType::Remote(3)
                ),
                Err(ParseError::RemovedWrongDlc)
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryVoltage,
                    ParserType::Remote(4)
                ),
                Ok(Frame::BatteryVoltage(Remote))
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryVoltage,
                    ParserType::Data(&[0, 0, 0, 0x57, 0xA4])
                ),
                Err(ParseError::WrongDataSize)
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryVoltage,
                    ParserType::Data(&[])
                ),
                Ok(Frame::BatteryVoltage(Data(None)))
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryVoltage,
                    ParserType::Data(&[0, 0, 0x57, 0xA4])
                ),
                Ok(Frame::BatteryVoltage(Data(Some(0x000057A4))))
            );
        }

        {
            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCurrent,
                    ParserType::Remote(3)
                ),
                Err(ParseError::RemovedWrongDlc)
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCurrent,
                    ParserType::Remote(4)
                ),
                Ok(Frame::BatteryCurrent(Remote))
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCurrent,
                    ParserType::Data(&[0, 0, 0, 0x57, 0xA4])
                ),
                Err(ParseError::WrongDataSize)
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCurrent,
                    ParserType::Data(&[])
                ),
                Ok(Frame::BatteryCurrent(Data(None)))
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCurrent,
                    ParserType::Data(&[0, 0, 0x57, 0xA4])
                ),
                Ok(Frame::BatteryCurrent(Data(Some(0x000057A4))))
            );
        }

        {
            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryVoltageCurrent,
                    ParserType::Remote(8)
                ),
                Ok(Frame::BatteryVoltageCurrent(Remote))
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryVoltageCurrent,
                    ParserType::Data(&[0, 0, 0x57, 0xA4, 0, 0, 0xB0, 0x14])
                ),
                Ok(Frame::BatteryVoltageCurrent(Data(Some((0x000057A4, 0x0000B014)))))
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryVoltageCurrent,
                    ParserType::Data(&[0, 0, 0x57, 0xA4, 0, 0, 0, 0])
                ),
                Ok(Frame::BatteryVoltageCurrent(Data(Some((0x000057A4, 0i32)))))
            );
        }

        {
            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCellCount,
                    ParserType::Remote(3)
                ),
                Err(ParseError::RemovedWrongDlc)
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCellCount,
                    ParserType::Remote(2)
                ),
                Ok(Frame::BatteryCellCount(Remote))
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCellCount,
                    ParserType::Data(&[30])
                ),
                Err(ParseError::WrongDataSize)
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCellCount,
                    ParserType::Data(&[0, 30])
                ),
                Ok(Frame::BatteryCellCount(Data(30)))
            );
        }

        {
            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCellsStates,
                    ParserType::Remote(0)
                ),
                Ok(Frame::BatteryCellsStates(Remote))
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCellsStates,
                    ParserType::Data(&[0, 0, 0, 0x57, 0xA4])
                ),
                Err(ParseError::WrongData)
            );

            assert_eq!(
                Frame::parse_frame(
                    MessageId::BatteryCellsStates,
                    ParserType::Data(&[0, 10, 0x57, 0xA4, 0x0A, 0x44])
                ),
                Ok(Frame::BatteryCellsStates(Data(battery::CellsStates {
                    from: 10,
                    values: {
                        let mut arr: arrayvec::ArrayVec<u16, 3> = Default::default();
                        arr.push(0x57A4);
                        arr.push(0x0A44);
                        arr
                    },
                })))
            );
        }
    }
}
*/