use crate::messages::helpers::CopyIntoSlice;
use core::fmt::Debug;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Battery {
    pub temperature: [i8; 5],
    pub fan_duty: u8,
}

impl TryFrom<&[u8]> for Battery {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.get(0..6) {
            Some(value) => Ok(Self {
                temperature: {
                    let mut array: [i8; 5] = Default::default();
                    for i in array.iter_mut().zip(value) {
                        *i.0 = *i.1 as i8;
                    }
                    array
                },
                fan_duty: value[5],
            }),
            None => Err(()),
        }
    }
}

impl CopyIntoSlice for Battery {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize> {
        match dst.get_mut(0..6) {
            Some(x) => {
                x[..5].copy_from_slice(self.temperature.map(|v| v as u8).as_ref());
                x[5] = self.fan_duty;
                Some(x.len())
            }
            None => None,
        }
    }
}

impl From<[u8; 6]> for Battery {
    fn from(v: [u8; 6]) -> Self {
        Self::try_from(v.as_ref()).unwrap()
    }
}

impl From<Battery> for [u8; 6] {
    fn from(v: Battery) -> Self {
        let mut data: [u8; 6] = [0; 6];
        v.copy_into_slice(&mut data);
        data
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct BatteryRequest {
    pub fan_duty: Option<u8>,
}

impl TryFrom<&[u8]> for BatteryRequest {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.get(0) {
            Some(value) => Ok(Self {
                fan_duty: match value & 0b1000_0000 {
                    0 => None,
                    _ => Some(value & 0b0111_1111),
                } ,
            }),
            None => Err(()),
        }
    }
}

impl CopyIntoSlice for BatteryRequest {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize> {
        match dst.get_mut(0..1) {
            Some(x) => {
                x[0] = match self.fan_duty {
                    None => 0,
                    Some(fan_duty) => fan_duty | 0b1000_0000,
                };
                Some(x.len())
            }
            None => None,
        }
    }
}

impl From<[u8; 1]> for BatteryRequest {
    fn from(v: [u8; 1]) -> Self {
        Self::try_from(v.as_ref()).unwrap()
    }
}

impl From<BatteryRequest> for [u8; 1] {
    fn from(v: BatteryRequest) -> Self {
        let mut data: [u8; 1] = [0; 1];
        v.copy_into_slice(&mut data);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let s = Battery::from([1, 255, 0, 254, 253, 45]);
        assert_eq!(s.temperature, [1, -1, 0, -2, -3]);
        assert_eq!(s.fan_duty, 45);

        let s = Battery::try_from([1, 255, 0, 254, 253, 0]).unwrap();
        assert_eq!(s.temperature, [1, -1, 0, -2, -3]);
        assert_eq!(s.fan_duty, 0);
    }

    #[test]
    fn test_request() {
        let s = BatteryRequest::from([1]);
        assert_eq!(s.fan_duty, None);
        assert_eq!(s.try_into(), Ok([0, ]));

        let s = BatteryRequest::from([0b1000_0000_u8]);
        assert_eq!(s.fan_duty, Some(0));
        assert_eq!(s.try_into(), Ok([0b1000_0000_u8, ]));

        let s = BatteryRequest::from([0b1000_1111_u8]);
        assert_eq!(s.fan_duty, Some(0b1111));
        assert_eq!(s.try_into(), Ok([0b1000_1111_u8, ]));
    }
}
