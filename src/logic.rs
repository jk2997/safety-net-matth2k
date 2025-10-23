/*!

  Four-state logic

*/

use std::{fmt, str::FromStr};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
/// An enum to represent four-state logic
pub enum Logic {
    /// Logical zero
    False,
    /// Logical one
    True,
    /// Don't care
    X,
    /// High-impedance state
    Z,
}

impl Logic {
    /// Get logic as bool
    pub fn unwrap(self) -> bool {
        match self {
            Logic::True => true,
            Logic::False => false,
            _ => panic!("Logic is not truthy"),
        }
    }

    /// Get logic as bool with failure `msg`
    pub fn expect(self, msg: &str) -> bool {
        match self {
            Logic::True => true,
            Logic::False => false,
            _ => panic!("{}", msg),
        }
    }

    /// Returns [prim@true] if the logic is a don't care
    pub fn is_dont_care(&self) -> bool {
        matches!(self, Logic::X)
    }

    /// Returns the logic as a string
    pub fn as_str(&self) -> &str {
        match self {
            Logic::True => "1'b1",
            Logic::False => "1'b0",
            Logic::X => "1'bx",
            Logic::Z => "1'bz",
        }
    }

    /// Create a four-state logic element from a boolean
    pub fn from_bool(b: bool) -> Logic {
        if b { Logic::True } else { Logic::False }
    }
}

impl std::ops::BitAnd for Logic {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Logic::False, _) | (_, Logic::False) => Logic::False,
            (Logic::True, Logic::True) => Logic::True,
            (Logic::True, Logic::Z) | (Logic::Z, Logic::True) => Logic::X,
            (Logic::X, _) | (_, Logic::X) | (Logic::Z, Logic::Z) => Logic::X,
        }
    }
}

impl std::ops::BitOr for Logic {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Logic::True, _) | (_, Logic::True) => Logic::True,
            (Logic::False, Logic::False) => Logic::False,
            _ => Logic::X,
        }
    }
}

impl std::ops::Not for Logic {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Logic::False => Logic::True,
            Logic::True => Logic::False,
            _ => Logic::X,
        }
    }
}

impl fmt::Display for Logic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Logic::True => write!(f, "1'b1"),
            Logic::False => write!(f, "1'b0"),
            Logic::X => write!(f, "1'bx"),
            Logic::Z => write!(f, "1'bz"),
        }
    }
}

impl From<bool> for Logic {
    fn from(value: bool) -> Self {
        if value { Logic::True } else { Logic::False }
    }
}

impl FromStr for Logic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1'b1" | "1'h1" => Ok(Logic::True),
            "1'b0" | "1'h0" => Ok(Logic::False),
            "1'bx" | "1'hx" => Ok(Logic::X),
            "1'bz" | "1'hz" => Ok(Logic::Z),
            _ => Err(Error::ParseError(s.to_string())),
        }
    }
}

/// Create a [Logic::True] instance
pub fn r#true() -> Logic {
    Logic::True
}

/// Create a [Logic::False] instance
pub fn r#false() -> Logic {
    Logic::False
}

/// Create a don't care instance
pub fn dont_care() -> Logic {
    Logic::X
}

/// Create a high-impedance instance
pub fn high_z() -> Logic {
    Logic::Z
}

#[test]
fn test_logic_unwrap() {
    assert_eq!(Logic::True.unwrap(), true);
    assert_eq!(Logic::False.unwrap(), false);
    assert_eq!(Logic::True.expect("Should be true"), true);
    assert_eq!(Logic::False.expect("Should be false"), false);
}

#[test]
#[should_panic(expected = "Custom error message")]
fn test_expect_x_panics_with_message() {
    Logic::X.expect("Custom error message");
}
#[test]
#[should_panic(expected = "Another message")]
fn test_expect_z_panics_with_message() {
    Logic::Z.expect("Another message");
}

#[test]
fn test_is_dont_care() {
    assert!(Logic::X.is_dont_care());
}

#[test]
fn test_as_str() {
    assert_eq!(Logic::True.as_str(), "1'b1");
    assert_eq!(Logic::False.as_str(), "1'b0");
    assert_eq!(Logic::X.as_str(), "1'bx");
    assert_eq!(Logic::Z.as_str(), "1'bz");
}

#[test]
fn test_from_bool() {
    assert_eq!(Logic::from_bool(true), Logic::True);
    assert_eq!(Logic::from_bool(false), Logic::False);
}

#[test]
fn test_bitand() {
    assert_eq!(Logic::True & Logic::True, Logic::True);
    assert_eq!(Logic::False & Logic::False, Logic::False);
    assert_eq!(Logic::False & Logic::True, Logic::False);
    assert_eq!(Logic::False & Logic::X, Logic::False);
    assert_eq!(Logic::False & Logic::Z, Logic::False);
    assert_eq!(Logic::True & Logic::False, Logic::False);
    assert_eq!(Logic::X & Logic::True, Logic::X);
    assert_eq!(Logic::X & Logic::X, Logic::X);
    assert_eq!(Logic::X & Logic::Z, Logic::X);
    assert_eq!(Logic::True & Logic::X, Logic::X);
    assert_eq!(Logic::True & Logic::Z, Logic::X);
    assert_eq!(Logic::Z & Logic::True, Logic::X);
}

#[test]
fn test_bitor() {
    assert_eq!(Logic::True | Logic::True, Logic::True);
    assert_eq!(Logic::True | Logic::False, Logic::True);
    assert_eq!(Logic::True | Logic::X, Logic::True);
    assert_eq!(Logic::True | Logic::Z, Logic::True);
    assert_eq!(Logic::False | Logic::True, Logic::True);
    assert_eq!(Logic::False | Logic::False, Logic::False);
    assert_eq!(Logic::False | Logic::X, Logic::X);
    assert_eq!(Logic::False | Logic::Z, Logic::X);
    assert_eq!(Logic::X | Logic::False, Logic::X);
    assert_eq!(Logic::X | Logic::X, Logic::X);
    assert_eq!(Logic::X | Logic::Z, Logic::X);
    assert_eq!(Logic::Z | Logic::False, Logic::X);
    assert_eq!(Logic::Z | Logic::X, Logic::X);
    assert_eq!(Logic::Z | Logic::Z, Logic::X);
}

#[test]
fn test_not() {
    assert_eq!(!Logic::True, Logic::False);
    assert_eq!(!Logic::False, Logic::True);
    assert_eq!(!Logic::X, Logic::X);
    assert_eq!(!Logic::Z, Logic::X);
}

#[test]
fn test_helper_functions() {
    assert_eq!(r#true(), Logic::True);
    assert_eq!(r#false(), Logic::False);
    assert_eq!(dont_care(), Logic::X);
    assert_eq!(high_z(), Logic::Z);
}
