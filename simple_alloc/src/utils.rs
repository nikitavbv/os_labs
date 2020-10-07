pub trait ToBit {
    fn to_bit(&self) -> usize;
}

impl ToBit for bool {

    fn to_bit(&self) -> usize {
        if *self {
            1
        } else {
            0
        }
    }
}

pub trait FromBit {
    fn from_bit(v: usize) -> Self;
}

impl FromBit for bool {

    fn from_bit(v: usize) -> Self {
        if v == 0 {
            false
        } else {
            true
        }
    }
}