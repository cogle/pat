use anyhow::Result;

pub trait PollSelectable {
    type Output;

    fn value(self) -> Self::Output;
}

pub trait PollResult {
    type Output;
    fn value(self) -> Self::Output;
    fn value_as_ref(&self) -> &Self::Output;
}

pub trait Pollable {
    type Error;
    type Output: PollResult;
    type Selection: PollSelectable;

    fn poll(self: &mut Self, selection: Self::Selection) -> Result<Self::Output, Self::Error>;
}
