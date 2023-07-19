pub trait PollSelectable {
    type Output;
    fn value(self) -> Self::Output;
}

pub trait Pollable {
    type Selection: PollSelectable;
    fn poll(self: &mut Self, selection: Self::Selection);
}
