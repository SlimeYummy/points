#[repr(u8)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum TmplSlotType {
    Attack = 1,
    Defense = 2,
    Special = 3,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TmplSlotValue {
    pub special: u8,
    pub attack: u8,
    pub defense: u8,
}

impl TmplSlotValue {
    pub fn new(special: u8, attack: u8, defense: u8) -> TmplSlotValue {
        return TmplSlotValue {
            special,
            attack,
            defense,
        };
    }

    pub fn merge(&self, other: &TmplSlotValue) -> TmplSlotValue {
        return TmplSlotValue {
            special: self.special + other.special,
            attack: self.attack + other.attack,
            defense: self.defense + other.defense,
        };
    }

    pub fn append(&mut self, other: &TmplSlotValue) {
        *self = self.merge(other);
    }
}

impl From<(u8, u8, u8)> for TmplSlotValue {
    fn from((special, attack, defense): (u8, u8, u8)) -> Self {
        return TmplSlotValue {
            special,
            attack,
            defense,
        };
    }
}

impl Into<(u8, u8, u8)> for TmplSlotValue {
    fn into(self) -> (u8, u8, u8) {
        return (self.special, self.attack, self.defense);
    }
}

impl From<[u8; 3]> for TmplSlotValue {
    fn from([special, attack, defense]: [u8; 3]) -> Self {
        return TmplSlotValue {
            special,
            attack,
            defense,
        };
    }
}

impl Into<[u8; 3]> for TmplSlotValue {
    fn into(self) -> [u8; 3] {
        return [self.special, self.attack, self.defense];
    }
}

const _: () = {
    use serde::de::Deserializer;
    use serde::Deserialize;

    impl<'de> Deserialize<'de> for TmplSlotValue {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<TmplSlotValue, D::Error> {
            let arr: [u8; 3] = Deserialize::deserialize(deserializer)?;
            return Ok(TmplSlotValue::from(arr));
        }
    }
};
