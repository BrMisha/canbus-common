use crate::messages::helpers::CopyIntoSlice;
use core::fmt::Debug;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Battery {
    pub temperature: [i8; 5],
}

impl TryFrom<&[u8]> for Battery {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.get(0..5) {
            Some(value) => Ok(Self {
                temperature: {
                    let mut array: [i8; 5] = Default::default();
                    for i in array.iter_mut().zip(value) {
                        *i.0 = *i.1 as i8;
                    }
                    array
                },
            }),
            None => Err(()),
        }
    }
}

impl CopyIntoSlice for Battery {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize> {
        match dst.get_mut(0..5) {
            Some(x) => {
                x.copy_from_slice(self.temperature.map(|v| v as u8).as_ref());
                Some(x.len())
            }
            None => None,
        }
    }
}

impl From<[u8; 5]> for Battery {
    fn from(v: [u8; 5]) -> Self {
        Self::try_from(v.as_ref()).unwrap()
    }
}

impl From<Battery> for [u8; 5] {
    fn from(v: Battery) -> Self {
        let mut data: [u8; 5] = [0; 5];
        v.copy_into_slice(&mut data);
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let s = Battery::from([1, 255, 0, 254, 253]);
        assert_eq!(s.temperature, [1, -1, 0, -2, -3]);

        let s = Battery::try_from([1, 255, 0, 254, 253]).unwrap();
        assert_eq!(s.temperature, [1, -1, 0, -2, -3]);
    }
}
