#![no_std]

use crate::message_id::MessageId;
use num_traits::{FromPrimitive, ToPrimitive};

pub mod message_id;
pub mod messages;

pub fn from_slice(data: &[u8]) -> Option<messages::Message> {
    let id = *data.first().ok_or(()).ok()?;
    let m_id = MessageId::from_u8(id & 0b01111111u8).ok_or(()).ok()?;
    let is_request = id & 0b10000000u8 != 0;
    messages::Message::parse_message(m_id, &data[1..], is_request).ok()
}

pub fn to_slice(message: &messages::Message, dst: &mut [u8]) -> Option<usize> {
    let (size, is_request) = message.message_into_slise(dst.get_mut(1..)?)?;

    let id = dst.first_mut()?;
    *id = message.id().to_u8()?;
    if is_request {
        *id |= 0b10000000;
    };

    Some(size + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::firmware;

    #[test]
    fn convert_data() {
        let mess = messages::Message::FirmwareUploadPartChangePos(messages::Type::Data(
            firmware::UploadPartChangePos::new(1000).unwrap(),
        ));
        // not enough space
        assert_eq!(to_slice(&mess, &mut [1, 2, 3]), None);
        assert_eq!(to_slice(&mess, &mut [1,]), None);
        assert_eq!(to_slice(&mess, &mut []), None);

        let mut buf = [0; 50];
        let size = to_slice(&mess, &mut buf).unwrap();
        assert_eq!(size, 4);

        assert_eq!(from_slice(&[]), None);
        assert_eq!(from_slice(&buf), Some(mess));
    }

    #[test]
    fn convert_request() {
        let mess = messages::Message::FirmwareVersion(messages::Type::Request(messages::Empty));
        // not enough space
        assert_eq!(to_slice(&mess, &mut []), None);

        let mut buf = [0; 50];
        let size = to_slice(&mess, &mut buf).unwrap();
        assert_eq!(size, 1);

        assert_eq!(from_slice(&buf), Some(mess));
    }
}
