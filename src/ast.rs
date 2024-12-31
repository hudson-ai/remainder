use std::iter;

// Enum to represent regex types
#[derive(Debug, Clone)]
pub enum Regex {
    Empty,              // Matches nothing
    Epsilon,            // Matches the empty string
    Literal(char),      // Matches a specific character
    Concat(Vec<Regex>), // Concatenation
    Or(Vec<Regex>),     // Union (alternation)
    And(Vec<Regex>),    // Intersection
    Not(Box<Regex>),    // Negation
    Star(Box<Regex>),   // Kleene star
    Remainder {
        // Value mod divisor is target_remainder
        divisor: u32,
        current_remainder: u32,
        target_remainder: u32,
        scale: u32,
        fractional_mode: bool,
    },
}

impl Regex {
    fn nullable(&self) -> bool {
        match self {
            Regex::Empty => false,
            Regex::Epsilon => true,
            Regex::Literal(_) => false,
            Regex::Concat(rxs) => rxs.iter().all(|rx| rx.nullable()),
            Regex::Or(rxs) => rxs.iter().any(|rx| rx.nullable()),
            Regex::And(rxs) => rxs.iter().all(|rx| rx.nullable()),
            Regex::Not(r) => !r.nullable(),
            Regex::Star(_) => true,
            Regex::Remainder {
                current_remainder,
                target_remainder,
                ..
            } => current_remainder == target_remainder,
        }
    }

    fn null(&self) -> Regex {
        if self.nullable() {
            Regex::Epsilon
        } else {
            Regex::Empty
        }
    }

    fn derivative(&self, c: &char) -> Regex {
        match self {
            Regex::Empty => Regex::Empty,
            Regex::Epsilon => Regex::Empty,
            Regex::Literal(ch) => {
                if ch == c {
                    Regex::Epsilon
                } else {
                    Regex::Empty
                }
            }
            // ∂_c (r · s) = ∂_c r · s + ν(r) · ∂_c s
            Regex::Concat(rxs) => {
                fn deriv_concat(dr: Regex, vr: Regex, s: &Regex, c: &char) -> Regex {
                    Regex::Or(vec![
                        Regex::Concat(vec![dr, s.clone()]),
                        Regex::Concat(vec![vr, s.derivative(c)]),
                    ])
                }
                let mut deriv_so_far = Regex::Empty;
                let mut null_so_far = Regex::Epsilon;
                for s in rxs.iter() {
                    deriv_so_far = deriv_concat(deriv_so_far, null_so_far.clone(), s, c);
                    null_so_far = Regex::Concat(vec![null_so_far, s.null()]);
                }
                deriv_so_far
            }
            Regex::Or(rxs) => Regex::Or(rxs.iter().map(|r| r.derivative(c)).collect()),
            Regex::And(rxs) => Regex::And(rxs.iter().map(|r| r.derivative(c)).collect()),
            Regex::Not(r) => Regex::Not(Box::new(r.derivative(c))),
            Regex::Star(r) => Regex::Concat(vec![r.derivative(c), Regex::Star(r.clone())]),
            Regex::Remainder {
                divisor,
                current_remainder,
                target_remainder,
                scale,
                fractional_mode,
            } => {
                if !fractional_mode && c == &'.' {
                    Regex::Remainder {
                        divisor: *divisor,
                        current_remainder: *current_remainder,
                        target_remainder: *target_remainder,
                        scale: *scale,
                        fractional_mode: true,
                    }
                } else if let Some(digit) = c.to_digit(10) {
                    if *fractional_mode && *scale == 0 {
                        return Regex::Empty;
                    }
                    let current_remainder = if !fractional_mode {
                        (current_remainder * 10 + digit * 10_u32.pow(*scale)) % divisor
                    } else {
                        (current_remainder + digit * 10_u32.pow(*scale - 1)) % divisor
                    };
                    let scale = if *fractional_mode { *scale - 1 } else { *scale };
                    Regex::Remainder {
                        divisor: *divisor,
                        current_remainder,
                        target_remainder: *target_remainder,
                        scale: scale,
                        fractional_mode: *fractional_mode,
                    }
                } else {
                    Regex::Empty
                }
            }
        }
    }

    pub fn matches(&self, s: &str) -> bool {
        let mut current = self.clone();
        for c in s.chars() {
            current = current.derivative(&c);
        }
        current.nullable()
    }

    // Highly suboptimal implementation of the repeat operator
    pub fn repeat(r: Regex, low: u32, high: Option<u32>) -> Regex {
        let mut result = vec![];
        for _ in 0..low {
            result.push(r.clone());
        }
        if let Some(high) = high {
            for _ in low..high {
                result.push(Regex::Or(vec![Regex::Epsilon, r.clone()]));
            }
        } else {
            result.push(Regex::Star(Box::new(r)));
        }
        Regex::Concat(result)
    }

    pub fn literal(s: &str) -> Regex {
        let mut result = vec![];
        for c in s.chars() {
            result.push(Regex::Literal(c));
        }
        Regex::Concat(result)
    }

    pub fn fractional_remainder(divisor: f32, remainder: u32) -> Result<Regex, String> {
        let (divisor, scale) = scale_divisor(divisor)?;

        Ok(Regex::Remainder {
            divisor,
            current_remainder: 0,
            target_remainder: remainder,
            scale: scale,
            fractional_mode: false,
        })
    }

    pub fn remainder(divisor: u32, remainder: u32) -> Regex {
        Regex::Remainder {
            divisor,
            current_remainder: 0,
            target_remainder: remainder,
            scale: 0,
            fractional_mode: false,
        }
    }
}

fn scale_divisor(divisor: f32) -> Result<(u32, u32), String> {
    if divisor.fract() == 0.0 {
        Ok((divisor.abs() as u32, 0))
    } else {
        let divisor_str = divisor.to_string();
        let decimal_part = divisor_str
            .split('.')
            .nth(1)
            .ok_or("No decimal part found")?;
        let scale = decimal_part.len();
        let scaled_divisor = divisor * 10_f32.powi(scale as i32);
        if scaled_divisor > u32::MAX as f32 {
            return Err("Scaled divisor exceeds u32::MAX".to_string());
        }

        Ok((scaled_divisor.abs() as u32, scale as u32))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_regex() {
        let regex = Regex::Concat(vec![
            Regex::Literal('a'),
            Regex::Star(Box::new(Regex::Literal('b'))),
        ]);

        assert_eq!(regex.matches("a"), true);
        assert_eq!(regex.matches("ab"), true);
        assert_eq!(regex.matches("abb"), true);
        assert_eq!(regex.matches("aba"), false);
        assert_eq!(regex.matches("b"), false);
    }

    #[test]
    fn test_repeat() {
        let regex = Regex::repeat(Regex::Literal('a'), 2, Some(4));
        assert_eq!(regex.matches("a"), false);
        assert_eq!(regex.matches("aa"), true);
        assert_eq!(regex.matches("aaa"), true);
        assert_eq!(regex.matches("aaaa"), true);
        assert_eq!(regex.matches("aaaaa"), false);

        let regex = Regex::repeat(Regex::Literal('a'), 2, None);
        assert_eq!(regex.matches("a"), false);
        assert_eq!(regex.matches("aa"), true);
        assert_eq!(regex.matches("aaa"), true);
        assert_eq!(regex.matches("aaaa"), true);
        assert_eq!(regex.matches("aaaaa"), true);
    }

    #[test]
    fn test_remainder() {
        for divisor in 1..=27 {
            for remainder in 0..divisor {
                let regex = Regex::remainder(divisor, remainder);
                for i in 0..1000 {
                    let s = i.to_string();
                    assert_eq!(
                        regex.matches(&s),
                        i % divisor == remainder,
                        "{:?} ({} % {} == {})",
                        regex,
                        s,
                        divisor,
                        remainder
                    );
                }
            }
        }
    }

    #[test]
    fn test_remainder_fractional() {
        let step = 0.0125;
        let (_, scale) = scale_divisor(step).unwrap();
        let factor = 10_f32.powi(scale as i32);
        for divisor in [2.5, 2.25, 2.125, 1.5, 1.25, 1.125, 0.5, 0.25, 0.125, 0.05].iter() {
            let regex = Regex::fractional_remainder(*divisor, 0).unwrap();
            for i in 0..100 {
                // Round to avoid floating point errors
                let i = ((i as f32 * step) * factor).round() / factor;
                // Check if i is a multiple of divisor, with some tolerance smaller than our factor
                let is_multiple = ((i / divisor) - (i / divisor).round()).abs() < 0.1 / factor;
                let s = i.to_string();
                assert_eq!(
                    regex.matches(&s),
                    is_multiple,
                    "{:?} ({} % {} == 0.0) ({}) {}",
                    regex,
                    s,
                    divisor,
                    is_multiple,
                    ((i / divisor) - (i / divisor).round()).abs()
                );
            }
        }
    }
}
