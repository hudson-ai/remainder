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
            } => match c.to_digit(10) {
                Some(digit) => {
                    let current_remainder = (current_remainder * 10 + digit as u32) % divisor;
                    Regex::Remainder {
                        divisor: *divisor,
                        current_remainder,
                        target_remainder: *target_remainder,
                    }
                }
                None => Regex::Empty,
            },
        }
    }

    fn empty(&self) -> bool {
        if self.nullable() {
            return false;
        }
        match self {
            Regex::Empty => true,
            Regex::Epsilon => false,
            Regex::Literal(_) => false,
            Regex::Concat(rxs) => rxs.iter().any(|rx| rx.empty()),
            Regex::Or(rxs) => rxs.iter().all(|rx| rx.empty()),
            Regex::And(rxs) => rxs.iter().any(|rx| rx.empty()),
            Regex::Not(r) => !r.empty(),
            Regex::Star(_) => false,
            Regex::Remainder { .. } => false,
        }
    }

    pub fn matches(&self, s: &str) -> bool {
        let mut current = self.clone();
        for c in s.chars() {
            // Short-circuit if the current regex matches nothing
            if current.empty() {
                return false;
            }
            current = current.derivative(&c);
        }
        current.nullable()
    }

    // Highly suboptimal implementation of the repeat operator
    pub fn repeat(r: Regex, low: usize, high: Option<usize>) -> Regex {
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
        let scale_power = decimal_part.len();

        if scale_power > 9 {
            return Err("Too many decimal places, potential u32 overflow".to_string());
        }

        let scale = 10_u32.pow(scale_power as u32);
        let scaled_divisor = (divisor * scale as f32).round();

        if scaled_divisor > u32::MAX as f32 {
            return Err("Scaled divisor exceeds u32::MAX".to_string());
        }

        Ok((scaled_divisor.abs() as u32, scale))
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
}
