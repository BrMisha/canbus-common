use crate::messages::helpers::CopyIntoSlice;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub path: u16,
    pub build: u32,
}

impl From<[u8; 8]> for Version {
    fn from(v: [u8; 8]) -> Self {
        Self::try_from(v.as_ref()).unwrap()
    }
}

impl TryFrom<&[u8]> for Version {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.get(0..8) {
            Some(value) => Ok(Self {
                major: value[0],
                minor: value[1],
                path: u16::from_be_bytes(<[u8; 2]>::try_from(&value[2..4]).unwrap()),
                build: u32::from_be_bytes(<[u8; 4]>::try_from(&value[4..8]).unwrap()),
            }),
            None => Err(())
        }
    }
}

impl From<Version> for [u8; 8] {
    fn from(v: Version) -> Self {
        let mut data: [u8; 8] = [0; 8];
        v.copy_into_slice(&mut data);
        data
    }
}

impl CopyIntoSlice for Version {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize> {
        match dst.get_mut(0..8) {
            Some(x) => {
                x[0] = self.major;
                x[1] = self.minor;
                x[2..4].clone_from_slice(&self.path.to_be_bytes());
                x[4..8].clone_from_slice(&self.build.to_be_bytes());
                Some(x.len())
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        let v = Version {
            major: 1,
            minor: 2,
            path: 3,
            build: 9864,
        };
        let arr: [u8; 8] = v.into();
        assert_eq!(Version::from(arr), v)
    }
}
