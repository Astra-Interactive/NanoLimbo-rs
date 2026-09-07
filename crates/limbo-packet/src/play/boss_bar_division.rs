/// How many notches split a boss bar, as the client numbers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarDivision {
    Solid,
    Dashes6,
    Dashes10,
    Dashes12,
    Dashes20,
}

impl BossBarDivision {
    pub const fn index(self) -> i32 {
        match self {
            Self::Solid => 0,
            Self::Dashes6 => 1,
            Self::Dashes10 => 2,
            Self::Dashes12 => 3,
            Self::Dashes20 => 4,
        }
    }
}
