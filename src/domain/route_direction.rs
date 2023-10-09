use std::fmt::Display;

use super::Terminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteDirection {
    North,
    South,
}

impl Display for RouteDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::North => f.write_str("North"),
            Self::South => f.write_str("South"),
        }
    }
}

impl From<(Terminal, Terminal)> for RouteDirection {
    fn from(ride: (Terminal, Terminal)) -> Self {
        match ride {
            (Terminal::Airport, _) | (_, Terminal::Rawai) => Self::South,
            (Terminal::Rawai, _) | (_, Terminal::Airport) => Self::North,
            _ => unreachable!("Unknown direction, {} => {}", ride.0, ride.1),
        }
    }
}

impl From<Terminal> for RouteDirection {
    fn from(terminal: Terminal) -> Self {
        match terminal {
            Terminal::Airport => Self::North,
            Terminal::Rawai => Self::South,
            _ => unreachable!("Can't choose direction for {}", terminal),
        }
    }
}
