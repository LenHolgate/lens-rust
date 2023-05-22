use std::cmp::Ordering;
use std::fmt;

use num::One;

use crate::id_type::IdType;

#[derive(PartialOrd, Eq, PartialEq, Clone)]
pub struct Interval<T: IdType> {
    lower: T,
    upper: T,
}

impl<T: IdType> Interval<T> {
    pub fn new(lower: T, upper: T) -> Self {
        if upper < lower {
            panic!("upper must be >= lower");
        }

        Interval { lower, upper }
    }

    pub fn new_single_value_interval(value: T) -> Self {
        Interval {
            lower: value,
            upper: value,
        }
    }

    pub fn lower(&self) -> T {
        self.lower
    }

    pub fn upper(&self) -> T {
        self.upper
    }

    fn dump(&self) -> String {
        format!("{}", self)
    }

    pub fn contains_value(&self, value: T) -> bool {
        value >= self.lower && value <= self.upper
    }

    pub fn overlaps(&self, value: &Self) -> bool {
        value.upper >= self.lower && value.lower <= self.upper
    }

    pub fn extends_lower(&self, value: &Self) -> bool {
        if value.upper == T::MAX {
            return false;
        }

        let next_value = value.upper + One::one();

        next_value == self.lower
    }

    pub fn extends_upper(&self, value: &Self) -> bool {
        if value.lower == T::MIN {
            return false;
        }

        let next_value = value.lower - One::one();

        next_value == self.upper
    }
}

impl<T: IdType> fmt::Display for Interval<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.lower == self.upper {
            write!(f, "[{}]", self.lower)
        } else {
            write!(f, "[{},{}]", self.lower, self.upper)
        }
    }
}

impl<T: IdType> Ord for Interval<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let lower_is = self.lower.cmp(&other.lower);

        let upper_is = self.upper.cmp(&other.upper);

        if lower_is == Ordering::Less {
            return Ordering::Less;
        }

        if upper_is == Ordering::Greater {
            return Ordering::Greater;
        }

        if upper_is == Ordering::Less {
            return Ordering::Less;
        }

        if lower_is == Ordering::Greater {
            return Ordering::Greater;
        }

        Ordering::Equal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "upper must be >= lower")]
    fn test_create_upper_less_than_lower() {
        let _interval = Interval::<u8>::new(12, 11);
    }

    #[test]
    fn test_create_for_all_supported_types() {
        {
            let _interval = Interval::<u8>::new(u8::MIN, u8::MAX);
        }
        {
            let _interval = Interval::<u16>::new(u16::MIN, u16::MAX);
        }
        {
            let _interval = Interval::<u32>::new(u32::MIN, u32::MAX);
        }
        {
            let _interval = Interval::<u64>::new(u64::MIN, u64::MAX);
        }
        {
            let _interval = Interval::<u128>::new(u128::MIN, u128::MAX);
        }
        {
            let _interval = Interval::<usize>::new(usize::MIN, usize::MAX);
        }
    }

    #[test]
    fn test_lower() {
        let interval = Interval::<u8> {
            lower: 10,
            upper: 11,
        };

        assert_eq!(interval.lower(), 10);
    }

    #[test]
    fn test_upper() {
        let interval = Interval::<u8> {
            lower: 10,
            upper: 11,
        };

        assert_eq!(interval.upper(), 11);
    }

    #[test]
    fn test_equal() {
        let interval1 = Interval::<u8> {
            lower: 10,
            upper: 12,
        };

        let interval2 = Interval::<u8> {
            lower: 10,
            upper: 12,
        };

        let interval3 = Interval::<u8> {
            lower: 11,
            upper: 11,
        };

        let interval4 = Interval::<u8> {
            lower: 10,
            upper: 10,
        };

        let interval5 = Interval::<u8> {
            lower: 9,
            upper: 15,
        };

        let interval6 = Interval::<u8> {
            lower: 11,
            upper: 11,
        };

        assert_eq!(interval1.cmp(&interval2), Ordering::Equal);
        assert_eq!(interval2.cmp(&interval1), Ordering::Equal);
        assert_eq!(interval1.cmp(&interval3), Ordering::Less);
        assert_eq!(interval1.cmp(&interval4), Ordering::Greater);
        assert_eq!(interval1.cmp(&interval5), Ordering::Less);
        assert_eq!(interval1.cmp(&interval6), Ordering::Less);
    }

    #[test]
    fn test_dump() {
        {
            let interval = Interval::<u8> {
                lower: 10,
                upper: 11,
            };

            assert_eq!(interval.dump(), "[10,11]");
        }
        {
            let interval = Interval::<u8>::new(22, 33);

            assert_eq!(interval.dump(), "[22,33]");
        }

        {
            let interval = Interval::<u8> {
                lower: 10,
                upper: 10,
            };

            assert_eq!(interval.dump(), "[10]");
        }
        {
            let interval = Interval::<u8>::new_single_value_interval(255);

            assert_eq!(interval.dump(), "[255]");
        }
    }

    #[test]
    fn test_contains_value() {
        let interval = Interval::<u8> {
            lower: 10,
            upper: 12,
        };

        assert_eq!(interval.contains_value(9), false);
        assert_eq!(interval.contains_value(10), true);
        assert_eq!(interval.contains_value(11), true);
        assert_eq!(interval.contains_value(12), true);
        assert_eq!(interval.contains_value(13), false);
    }

    #[test]
    fn test_contains() {
        let interval1 = Interval::<u8> {
            lower: 10,
            upper: 13,
        };

        let interval2 = Interval::<u8> {
            lower: 11,
            upper: 12,
        };

        assert_eq!(interval1.overlaps(&interval1), true);
        assert_eq!(interval1.overlaps(&interval2), true);
        assert_eq!(interval2.overlaps(&interval1), true);

        let interval3 = Interval::<u8> {
            lower: 11,
            upper: 14,
        };

        assert_eq!(interval1.overlaps(&interval3), true);
        assert_eq!(interval2.overlaps(&interval3), true);
        assert_eq!(interval3.overlaps(&interval1), true);
    }

    #[test]
    fn test_extends_lower() {
        let interval1 = Interval::<u8> {
            lower: 10,
            upper: 13,
        };

        let interval2 = Interval::<u8> { lower: 7, upper: 8 };

        assert_eq!(interval1.extends_lower(&interval2), false);

        let interval3 = Interval::<u8> { lower: 7, upper: 9 };

        assert_eq!(interval1.extends_lower(&interval3), true);
    }

    #[test]
    fn test_extends_lower_new_interval_is_max() {
        let interval1 = Interval::<u8> {
            lower: 10,
            upper: 13,
        };

        let new_interval = Interval::<u8> {
            lower: 14,
            upper: u8::MAX,
        };

        assert_eq!(interval1.extends_lower(&new_interval), false);
    }

    #[test]
    fn test_extends_lower_new_interval_is_min() {
        let interval1 = Interval::<u8> {
            lower: 10,
            upper: 13,
        };

        let new_interval = Interval::<u8> {
            lower: u8::MIN,
            upper: 17,
        };

        assert_eq!(interval1.extends_lower(&new_interval), false);
    }

    #[test]
    fn test_extends_upper() {
        let interval1 = Interval::<u8> {
            lower: 10,
            upper: 13,
        };

        let interval2 = Interval::<u8> {
            lower: 15,
            upper: 17,
        };

        assert_eq!(interval1.extends_upper(&interval2), false);

        let interval3 = Interval::<u8> {
            lower: 14,
            upper: 17,
        };

        assert_eq!(interval1.extends_upper(&interval3), true);
    }

    #[test]
    fn test_extends_upper_new_interval_is_max() {
        let interval1 = Interval::<u8> {
            lower: 10,
            upper: 13,
        };

        let new_interval = Interval::<u8> {
            lower: 14,
            upper: u8::MAX,
        };

        assert_eq!(interval1.extends_upper(&new_interval), true);
    }

    #[test]
    fn test_extends_upper_new_interval_is_min() {
        let interval1 = Interval::<u8> {
            lower: 10,
            upper: 13,
        };

        let new_interval = Interval::<u8> {
            lower: u8::MIN,
            upper: 17,
        };

        assert_eq!(interval1.extends_upper(&new_interval), false);
    }
}
