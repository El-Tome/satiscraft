//! Chances: decimals in the files for readability, exact fractions in the simulation.

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};

/// A probability between 0 and 1, as an exact fraction in lowest terms: the simulation draws
/// it, or averages it in aggregated mode, without floating point.
///
/// Content writes it as a decimal, `chance = 0.2`, with at most [`Chance::MAX_DECIMALS`]
/// decimal places, and it reads as the fraction that decimal denotes, `1/5`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chance {
    numerator: u32,
    /// Never 0.
    denominator: u32,
}

impl Chance {
    /// Decimal places a chance may have in content: one in a million at the finest.
    pub const MAX_DECIMALS: usize = 6;

    /// Never happens.
    pub const NEVER: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    /// Always happens.
    pub const ALWAYS: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    pub fn numerator(self) -> u32 {
        self.numerator
    }

    /// Never 0.
    pub fn denominator(self) -> u32 {
        self.denominator
    }

    /// The fraction denoted by the decimal written in the file, which TOML parsed to `value`.
    fn from_f64(value: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&value) {
            return Err(format!("a chance lies between 0 and 1, not {value}"));
        }
        // Rust prints the shortest decimal that parses back to `value`, never in exponent
        // form: the decimal written in the file, as long as it has few enough places.
        let written = value.abs().to_string();
        let (whole, decimals) = written.split_once('.').unwrap_or((&written, ""));
        if decimals.len() > Self::MAX_DECIMALS {
            return Err(format!(
                "a chance has at most {} decimal places, {value} has more",
                Self::MAX_DECIMALS
            ));
        }
        let numerator: u32 = format!("{whole}{decimals}")
            .parse()
            .map_err(|_| format!("{value} is not a decimal chance"))?;
        // At most MAX_DECIMALS places: the power fits.
        let denominator = 10_u32.pow(decimals.len() as u32);
        let divisor = gcd(numerator, denominator);
        Ok(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }
}

/// Greatest common divisor; `gcd(0, n) = n`.
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl<'de> Deserialize<'de> for Chance {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ChanceVisitor)
    }
}

/// Reads a decimal (`0.25`), or the integers `0` and `1`.
struct ChanceVisitor;

impl Visitor<'_> for ChanceVisitor {
    type Value = Chance;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a chance, a decimal between 0 and 1")
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Chance, E> {
        Chance::from_f64(value).map_err(E::custom)
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Chance, E> {
        match value {
            0 => Ok(Chance::NEVER),
            1 => Ok(Chance::ALWAYS),
            _ => Err(E::custom(format!(
                "a chance lies between 0 and 1, not {value}"
            ))),
        }
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Chance, E> {
        self.visit_i64(i64::try_from(value).unwrap_or(i64::MAX))
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct Loot {
        chance: Chance,
    }

    fn read(text: &str) -> Result<(u32, u32), String> {
        toml::from_str::<Loot>(text)
            .map(|loot| (loot.chance.numerator(), loot.chance.denominator()))
            .map_err(|err| err.message().to_owned())
    }

    #[test]
    fn decimals_become_fractions_in_lowest_terms() {
        for (text, fraction) in [
            ("0.2", (1, 5)),
            ("0.20", (1, 5)),
            ("0.25", (1, 4)),
            ("0.125", (1, 8)),
            ("0.333", (333, 1000)),
            ("0.000001", (1, 1_000_000)),
            ("0.0", (0, 1)),
            ("1.0", (1, 1)),
        ] {
            assert_eq!(read(&format!("chance = {text}")), Ok(fraction), "{text}");
        }
    }

    #[test]
    fn integers_zero_and_one_are_chances_too() {
        assert_eq!(read("chance = 0"), Ok((0, 1)));
        assert_eq!(read("chance = 1"), Ok((1, 1)));
        assert_eq!(Chance::NEVER.numerator(), 0);
        assert_eq!(Chance::ALWAYS.denominator(), 1);
    }

    #[test]
    fn chances_outside_zero_to_one_are_refused() {
        for (text, shown) in [("1.5", "1.5"), ("-0.1", "-0.1"), ("2", "2"), ("-1", "-1")] {
            assert_eq!(
                read(&format!("chance = {text}")),
                Err(format!("a chance lies between 0 and 1, not {shown}")),
            );
        }
        for text in ["nan", "inf"] {
            assert!(read(&format!("chance = {text}")).is_err(), "{text}");
        }
        assert!(read("chance = 'often'").is_err());
    }

    #[test]
    fn negative_zero_is_never() {
        assert_eq!(read("chance = -0.0"), Ok((0, 1)));
    }

    #[test]
    fn too_many_decimal_places_are_refused() {
        assert_eq!(
            read("chance = 0.0000001"),
            Err("a chance has at most 6 decimal places, 0.0000001 has more".into())
        );
    }

    #[test]
    fn a_refused_chance_points_at_its_value() {
        let err = toml::from_str::<Loot>("\nchance = 1.5\n").unwrap_err();
        assert_eq!(err.span(), Some(10..13));
    }

    #[test]
    fn gcd_reduces_fractions() {
        assert_eq!(gcd(25, 100), 25);
        assert_eq!(gcd(333, 1000), 1);
        assert_eq!(gcd(0, 10), 10);
        assert_eq!(gcd(7, 0), 7);
    }

    #[test]
    fn large_unsigned_integers_are_refused() {
        let err: Result<Chance, de::value::Error> = ChanceVisitor.visit_u64(u64::MAX);
        assert!(err.is_err());
    }
}
