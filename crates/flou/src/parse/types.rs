use nom_supreme::error::ErrorTree;

pub(crate) type Input<'i> = &'i str;
pub(crate) type Error<'i> = ErrorTree<Input<'i>>;

pub(crate) type Result<'i, O> = nom::IResult<Input<'i>, O, Error<'i>>;

pub(crate) trait Parser<'i, O>: nom::Parser<Input<'i>, O, Error<'i>> {}
impl<'i, O, N> Parser<'i, O> for N where N: nom::Parser<Input<'i>, O, Error<'i>> {}
