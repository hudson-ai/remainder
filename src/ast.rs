// Enum to represent regex types
#[derive(Debug, Clone)]
pub enum Regex {
    Empty,                          // Matches nothing
    Epsilon,                        // Matches the empty string
    Literal(char),                  // Matches a specific character
    Concat(Box<Regex>, Box<Regex>), // Concatenation
    Or(Box<Regex>, Box<Regex>),     // Union (alternation)
    And(Box<Regex>, Box<Regex>),    // Intersection
    Not(Box<Regex>),                // Negation
    Star(Box<Regex>),               // Kleene star
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
            Regex::Concat(r, s) => r.nullable() && s.nullable(),
            Regex::Or(r, s) => r.nullable() || s.nullable(),
            Regex::And(r, s) => r.nullable() && s.nullable(),
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
            // ∂a (r · s) = ∂a r · s + ν(r) · ∂a s
            Regex::Concat(r, s) => Regex::Or(
                Box::new(Regex::Concat(Box::new(r.derivative(c)), s.clone())),
                Box::new(Regex::Concat(Box::new(r.null()), Box::new(s.derivative(c)))),
            ),
            Regex::Or(r, s) => Regex::Or(Box::new(r.derivative(c)), Box::new(s.derivative(c))),
            Regex::And(r, s) => Regex::And(Box::new(r.derivative(c)), Box::new(s.derivative(c))),
            Regex::Not(r) => Regex::Not(Box::new(r.derivative(c))),
            Regex::Star(r) => {
                Regex::Concat(Box::new(r.derivative(c)), Box::new(Regex::Star(r.clone())))
            }
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
            Regex::Concat(r, _) => r.empty(),
            Regex::Or(r, s) => r.empty() && s.empty(),
            Regex::And(r, s) => r.empty() || s.empty(),
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
        let mut result = Regex::Epsilon;
        for _ in 0..low {
            result = Regex::Concat(Box::new(result), Box::new(r.clone()));
        }
        if let Some(high) = high {
            for _ in low..high {
                result = Regex::Concat(
                    Box::new(result),
                    Box::new(Regex::Or(Box::new(r.clone()), Box::new(Regex::Epsilon))),
                );
            }
        } else {
            result = Regex::Concat(Box::new(result), Box::new(Regex::Star(Box::new(r.clone()))));
        }
        result
    }

    pub fn literal(s: &str) -> Regex {
        let mut result = Regex::Epsilon;
        for c in s.chars() {
            result = Regex::Concat(Box::new(result), Box::new(Regex::Literal(c)));
        }
        result
    }

    pub fn remainder(divisor: u32, remainder: u32) -> Regex {
        Regex::Remainder {
            divisor,
            current_remainder: 0,
            target_remainder: remainder,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_regex() {
        let regex = Regex::Concat(
            Box::new(Regex::Literal('a')),
            Box::new(Regex::Star(Box::new(Regex::Literal('b')))),
        );

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
