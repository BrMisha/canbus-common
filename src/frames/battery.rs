#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct CellsStates {
    pub from: u16,
    pub values: arrayvec::ArrayVec<u16, 3>,
}

impl TryFrom<&[u8]> for CellsStates {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 4 || value.len()%2 != 0 {
            return Err(());
        }

        let mut value = value.chunks(2).map(|v| {
            u16::from_be_bytes(<[u8; 2]>::try_from(&v[..2]).unwrap())
        });

        let from = value.next().unwrap();
        let values= value
            .collect::<arrayvec::ArrayVec<u16, 3>>();

        Ok(Self{
            from,
            values,
        })
    }
}

impl From<&CellsStates> for arrayvec::ArrayVec<u8, 8> {
    fn from(v: &CellsStates) -> Self {
        let mut array: arrayvec::ArrayVec<u8, 8> = Default::default();

        let mut ar: [u8; 2] = v.from.to_be_bytes();
        array.push(ar[0]);
        array.push(ar[1]);

        for i in &v.values {
            ar = i.to_be_bytes();
            array.push(ar[0]);
            array.push(ar[1]);
        }

        array
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_voltage() {
        let mut array = arrayvec::ArrayVec::<u16, 3>::default();
        {
            array.push(4200);

            let v = CellsStates {
                from: 2,
                values: array,
            };

            let raw = arrayvec::ArrayVec::<u8, 8>::from(&v);
            assert_eq!(CellsStates::try_from(raw.as_slice()).unwrap(), v)
        }

        {
            array = Default::default();

            array.push(4200);
            array.push(15100);
            array.push(1000);

            let v = CellsStates {
                from: 2,
                values: array,
            };

            let raw = arrayvec::ArrayVec::<u8, 8>::from(&v);
            assert_eq!(CellsStates::try_from(raw.as_slice()).unwrap(), v);
        }

        {
            assert_eq!(CellsStates::try_from(&[0u8; 3][..]), Err(()));
            assert_eq!(CellsStates::try_from(&[0u8; 5][..]), Err(()));
        }
    }
}