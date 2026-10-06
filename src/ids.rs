#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModelId(u64);

impl ModelId {
    pub(crate) const fn of(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequestId(u64);

impl RequestId {
    pub(crate) const fn of(raw: u64) -> Self {
        Self(raw)
    }
}
