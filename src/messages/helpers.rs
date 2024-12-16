pub trait CopyIntoSlice {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize>;
}

impl CopyIntoSlice for bool {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize> {
        *(dst.get_mut(0)?) = *self as u8;
        Some(1)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct OptionWrapped<T>(pub Option<T>);

impl<T: CopyIntoSlice> CopyIntoSlice for OptionWrapped<T> {
    fn copy_into_slice(&self, dst: &mut [u8]) -> Option<usize> {
        match &self.0 {
            None => Some(0),
            Some(value) => {
                let size = value.copy_into_slice(dst)?;
                // If 0 or 1 byte, add to the end flag
                if size < 2 {
                    let flag = dst.get_mut(size)?;
                    *flag = 1;
                }
                Some(size)
            }
        }
    }
}

impl<'a, T> TryFrom<&'a [u8]> for OptionWrapped<T>
where
    T: TryFrom<&'a [u8]>,

{
    type Error = ();

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value.len() {
            0 => Ok(OptionWrapped(None)),
            1..=2 => {
                // check the flag
                (*value.last().ok_or(())? == 1).then_some(()).ok_or(())?;
                let value = value.get(..value.len()-1).ok_or(())?;
                let r = T::try_from(value).map_err(|_|())?;
                Ok(Self(Some(r)))
            },
            3.. => {
                let r = T::try_from(value).map_err(|_|())?;
                Ok(Self(Some(r)))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::serial;
    use super::*;
    extern crate std;

    #[test]
    fn option_wrapped() {
        let mut buff = [0u8; 200];
        let size = OptionWrapped::<bool>(None).copy_into_slice(&mut buff);
        assert_eq!(size, Some(0));

        let size = OptionWrapped(Some(true)).copy_into_slice(&mut buff);
        assert_eq!(size, Some(1));
        assert_ne!(buff[0], 0);

        let size = OptionWrapped(Some(false)).copy_into_slice(&mut buff);
        assert_eq!(size, Some(1));
        assert_eq!(buff[0], 0);

        let size = OptionWrapped(Some(serial::Serial::from([1,2,3,4,5]))).copy_into_slice(&mut buff);
        assert_eq!(size, Some(5));
        assert_eq!(buff[0..5], [1,2,3,4,5]);
    }
}