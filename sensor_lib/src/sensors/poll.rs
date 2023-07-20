use anyhow::Result;

pub trait PollSelectable {
    type Output;

    fn value(self) -> Self::Output;
}

pub trait PollResult {
    type Output;
    fn value(self) -> Self::Output;
}

pub trait Pollable {
    type Output: PollResult;
    type Selection: PollSelectable;

    fn poll(self: &mut Self, selection: Self::Selection) -> Result<Self::Output>;
}
