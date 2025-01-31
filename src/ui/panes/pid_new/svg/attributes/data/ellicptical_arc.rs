use super::DToken;
use nom::{
    branch::alt,
    character::complete::anychar,
    character::complete::char,
    combinator::map,
    number::complete::float,
    sequence::{preceded, tuple},
    IResult,
};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub struct EllipticalArc {
    abs: bool,
    rx: f32,
    ry: f32,
    angle: f32,
    large_arc: bool,
    sweep: bool,
    x: f32,
    y: f32,
}

impl EllipticalArc {
    pub fn abs(
        rx: f32,
        ry: f32,
        angle: f32,
        large_arc: bool,
        sweep: bool,
        x: f32,
        y: f32,
    ) -> DToken {
        DToken::EllipticalArc(Self {
            abs: true,
            rx,
            ry,
            angle,
            large_arc,
            sweep,
            x,
            y,
        })
    }

    pub fn rel(
        rx: f32,
        ry: f32,
        angle: f32,
        large_arc: bool,
        sweep: bool,
        x: f32,
        y: f32,
    ) -> DToken {
        DToken::EllipticalArc(Self {
            abs: false,
            rx,
            ry,
            angle,
            large_arc,
            sweep,
            x,
            y,
        })
    }

    pub(super) fn parse(input: &str) -> IResult<&str, DToken> {
        map(
            tuple((
                alt((map(char('A'), |_| true), map(char('a'), |_| false))),
                preceded(char(' '), float),
                preceded(char(' '), float),
                preceded(char(' '), float),
                map(preceded(char(' '), anychar), |c| c == '1'),
                map(preceded(char(' '), anychar), |c| c == '1'),
                preceded(char(' '), float),
                preceded(char(' '), float),
            )),
            |(abs, rx, ry, angle, large_arc, sweep, x, y)| {
                DToken::EllipticalArc(Self {
                    abs,
                    rx,
                    ry,
                    angle,
                    large_arc,
                    sweep,
                    x,
                    y,
                })
            },
        )(input)
    }
}

impl Display for EllipticalArc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = if self.abs { 'A' } else { 'a' };
        write!(
            f,
            "{} {} {} {} {} {} {} {}",
            prefix,
            self.rx,
            self.ry,
            self.angle,
            self.large_arc as i32,
            self.sweep as i32,
            self.x,
            self.y
        )
    }
}
