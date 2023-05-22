use std::collections::Bound::{Excluded, Included, Unbounded};
use std::collections::BTreeSet;
use std::fmt;

use num::One;

use crate::id_type::IdType;
use crate::interval::Interval;

pub struct Intervals<T: IdType> {
    intervals: BTreeSet<Interval<T>>,
}

impl<T: IdType> Intervals<T> {
    pub fn new() -> Self {
        Intervals::<T> {
            intervals: BTreeSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn insert_interval(&mut self, lower: T, upper: T) -> bool {
        let interval = Interval::new(lower, upper);

        self.insert(interval)
    }

    pub fn insert_value(&mut self, value: T) -> bool {
        let interval = Interval::new_single_value_interval(value);

        self.insert(interval)
    }

    pub fn dump(&self) -> String {
        format!("{}", self)
    }

    pub fn remove_first_interval(&mut self) -> Interval<T> {
        let first_it = self.intervals.iter().next();

        if let Some(first_interval) = first_it {
            let ret = first_interval.clone();

            self.intervals.remove(&first_interval.clone());

            return ret;
        }

        panic!("Empty!");
    }

    pub fn remove_first_value(&mut self) -> T {
        let first_interval = self.remove_first_interval();

        let first_value = first_interval.lower();

        if first_interval.lower() != first_interval.upper()
        {
            self.intervals
                .insert(Interval::new(first_value + One::one(), first_interval.upper()));
        }

        first_value
    }

    pub fn remove_value(&mut self, value: T) -> bool {
        if let Some(interval) = self.find(&Interval::new_single_value_interval(value)) {
            if interval.lower() < value {
                self.intervals
                    .insert(Interval::new(interval.lower(), value - One::one()));
            }

            if value < interval.upper() {
                self.intervals
                    .insert(Interval::new(value + One::one(), interval.upper()));
            }

            self.intervals.remove(&interval);

            return true;
        }

        false
    }

    pub fn remove_interval(&mut self, lower: T, upper: T) {
        let mut remove_these: BTreeSet<Interval<T>> = BTreeSet::new();

        let mut add_these: BTreeSet<Interval<T>> = BTreeSet::new();

        {
            let interval_to_remove = Interval::new(lower, upper);

            let intervals = self.intervals.range((Included(Interval::new(T::MIN, lower)), Unbounded));

            for interval in intervals {
                if interval_to_remove.overlaps(interval) {
                    remove_these.insert(interval.clone());
                }
                else if interval.contains_value(lower) || interval.contains_value(upper) {
                    remove_these.insert(interval.clone());
                }

                if interval.lower() < lower && interval.upper() >= lower {
                    add_these.insert(Interval::<T>::new(interval.lower(), lower - T::one()));
                }

                if interval.upper() > upper && interval.lower() <= upper {
                    add_these.insert(Interval::<T>::new(upper + T::one(), interval.upper()));
                }
            }
        }

        for interval in remove_these {
            self.intervals.remove(&interval);
        }

        for interval in add_these {
            self.intervals.insert(interval);
        }
    }

    fn find(&self, interval: &Interval<T>) -> Option<Interval<T>> {
        let before = self.intervals.range((Unbounded, Included(interval)));

        let prev = before.max();

        if let Some(prev) = prev {
            if prev.overlaps(interval) {
                return Some(prev.clone());
            }
        }

        let after = self.intervals.range((Included(interval), Unbounded));

        let next = after.min();

        if let Some(next) = next {
            if next.overlaps(interval) {
                return Some(next.clone());
            }
        }

        None
    }

    fn insert(&mut self, interval: Interval<T>) -> bool {
        let intervals_after = self.intervals.range((Included(&interval), Unbounded));

        let next_it = intervals_after.min();

        let next_is = next_it.is_some();

        if next_is {
            let next_ref = next_it.unwrap();

            if next_ref.overlaps(&interval)
            {
                return false;
            }
        }

        let intervals_before = self.intervals.range((Unbounded, Excluded(&interval)));

        let prev_it = intervals_before.max();

        let prev_is = prev_it.is_some();

        if prev_is {
            let prev_ref = prev_it.unwrap();

            if prev_ref.overlaps(&interval)
            {
                return false;
            }
        }

        if next_is && prev_is {
            self.insert_or_join_intervals(interval, next_it.unwrap().clone(), prev_it.unwrap().clone());
        } else if next_is {
            self.insert_or_merge_with_next(interval, next_it.unwrap().clone());
        } else if prev_is {
            self.insert_or_merge_with_prev(interval, prev_it.unwrap().clone());
        } else {
            self.intervals.insert(interval);
        }

        true
    }

    fn insert_or_join_intervals(&mut self, interval: Interval<T>, next: Interval<T>, prev: Interval<T>) {
        let next_extends = next.extends_lower(&interval);

        let prev_extends = prev.extends_upper(&interval);

        if next_extends && prev_extends
        {
            // merges the prev and next intervals

            let new_interval = Interval::new(prev.lower(), next.upper());

            self.intervals.remove(&prev);
            self.intervals.remove(&next);
            self.intervals.insert(new_interval);
        } else if next_extends
        {
            // extends the next interval

            let new_interval = Interval::new(interval.lower(), next.upper());

            self.intervals.remove(&next);
            self.intervals.insert(new_interval);
        } else if prev_extends
        {
            // extends the previous interval

            let new_interval = Interval::new(prev.lower(), interval.upper());

            self.intervals.remove(&prev);
            self.intervals.insert(new_interval);
        } else {
            self.intervals.insert(interval);
        }
    }

    fn insert_or_merge_with_next(&mut self, interval: Interval<T>, next: Interval<T>) {
        if next.extends_lower(&interval) {

            // extends the next interval

            let new_interval = Interval::new(interval.lower(), next.upper());

            self.intervals.remove(&next);
            self.intervals.insert(new_interval);
        } else {
            self.intervals.insert(interval);
        }
    }

    fn insert_or_merge_with_prev(&mut self, interval: Interval<T>, prev: Interval<T>) {
        if prev.extends_upper(&interval) {

            // extends the previous interval

            let new_interval = Interval::new(prev.lower(), interval.upper());

            self.intervals.remove(&prev);
            self.intervals.insert(new_interval);
        } else {
            self.intervals.insert(interval);
        }
    }
}

impl<T: IdType> fmt::Display for Intervals<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;

        for interval in self.intervals.iter() {
            if !first {
                write!(f, ", ")?;
            }

            write!(f, "{}", interval)?;

            if first {
                first = false;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let _intervals = Intervals::<u8>::new();
    }

    #[test]
    fn test_new_for_all_supported_types() {
        {
            let _intervals = Intervals::<u8>::new();
        }
        {
            let _intervals = Intervals::<u16>::new();
        }
        {
            let _intervals = Intervals::<u32>::new();
        }
        {
            let _intervals = Intervals::<u64>::new();
        }
        {
            let _intervals = Intervals::<u128>::new();
        }
        {
            let _intervals = Intervals::<usize>::new();
        }
    }

    #[test]
    fn test_is_empty_when_empty() {
        let intervals = Intervals::<u8>::new();

        assert_eq!(intervals.is_empty(), true);
    }

    #[test]
    fn test_insert_value() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(4), true);

        assert_eq!(intervals.dump(), "[4]");

        assert_eq!(intervals.insert_value(10), true);

        assert_eq!(intervals.dump(), "[4], [10]");
    }

    #[test]
    fn test_insert_interval() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(4, 10), true);

        assert_eq!(intervals.dump(), "[4,10]");

        assert_eq!(intervals.insert_interval(12, 20), true);

        assert_eq!(intervals.dump(), "[4,10], [12,20]");
    }

    #[test]
    fn test_is_empty_when_not_empty() {
        let mut intervals = Intervals::<u8>::new();

        intervals.insert_interval(4, 10);

        assert_eq!(intervals.is_empty(), false);
    }

    #[test]
    fn test_dump() {
        let mut intervals = Intervals::<u8>::new();

        intervals.insert_interval(4, 10);

        assert_eq!(intervals.dump(), "[4,10]");
    }

    #[test]
    fn test_insert_duplicate_value() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(4), true);

        assert_eq!(intervals.dump(), "[4]");

        assert_eq!(intervals.insert_value(4), false);

        assert_eq!(intervals.dump(), "[4]");
    }

    #[test]
    fn test_insert_duplicate_value_requires_correct_sorting() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(10, 20), true);

        assert_eq!(intervals.dump(), "[10,20]");

        assert_eq!(intervals.insert_value(8), true);

        assert_eq!(intervals.dump(), "[8], [10,20]");

        assert_eq!(intervals.insert_value(11), false);

        assert_eq!(intervals.dump(), "[8], [10,20]");
    }

    #[test]
    fn test_insert_value_extends_lower() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(4), true);

        assert_eq!(intervals.dump(), "[4]");

        assert_eq!(intervals.insert_value(3), true);

        assert_eq!(intervals.dump(), "[3,4]");
    }

    #[test]
    fn test_insert_value_extends_upper() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(4), true);

        assert_eq!(intervals.dump(), "[4]");

        assert_eq!(intervals.insert_value(5), true);

        assert_eq!(intervals.dump(), "[4,5]");
    }

    #[test]
    fn test_insert_value_is_before_and_is_first() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(4), true);

        assert_eq!(intervals.dump(), "[4]");

        assert_eq!(intervals.insert_value(2), true);

        assert_eq!(intervals.dump(), "[2], [4]");
    }

    #[test]
    fn test_insert_value_is_before_and_is_not_first() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(10), true);

        assert_eq!(intervals.dump(), "[10]");

        assert_eq!(intervals.insert_value(2), true);

        assert_eq!(intervals.dump(), "[2], [10]");

        assert_eq!(intervals.insert_value(5), true);

        assert_eq!(intervals.dump(), "[2], [5], [10]");

        assert_eq!(intervals.insert_value(6), true);

        assert_eq!(intervals.dump(), "[2], [5,6], [10]");
    }

    #[test]
    fn test_insert_value_is_after_and_is_last() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(10), true);

        assert_eq!(intervals.dump(), "[10]");

        assert_eq!(intervals.insert_value(18), true);

        assert_eq!(intervals.dump(), "[10], [18]");
    }

    #[test]
    fn test_insert_value_is_after_and_is_not_last() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(10), true);

        assert_eq!(intervals.dump(), "[10]");

        assert_eq!(intervals.insert_value(18), true);

        assert_eq!(intervals.dump(), "[10], [18]");

        assert_eq!(intervals.insert_value(15), true);

        assert_eq!(intervals.dump(), "[10], [15], [18]");
    }

    #[test]
    fn test_insert_value_joins_intervals() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_value(4), true);

        assert_eq!(intervals.dump(), "[4]");

        assert_eq!(intervals.insert_value(6), true);

        assert_eq!(intervals.dump(), "[4], [6]");

        assert_eq!(intervals.insert_value(5), true);

        assert_eq!(intervals.dump(), "[4,6]");
    }

    #[test]
    fn test_insert_overlapping_interval() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(2, 6), true);

        assert_eq!(intervals.dump(), "[2,6]");

        assert_eq!(intervals.insert_interval(2, 6), false);

        assert_eq!(intervals.dump(), "[2,6]");

        assert_eq!(intervals.insert_interval(3, 4), false);

        assert_eq!(intervals.dump(), "[2,6]");

        assert_eq!(intervals.insert_interval(4, 5), false);

        assert_eq!(intervals.dump(), "[2,6]");

        assert_eq!(intervals.insert_interval(5, 6), false);

        assert_eq!(intervals.dump(), "[2,6]");

        assert_eq!(intervals.insert_interval(5, 7), false);

        assert_eq!(intervals.dump(), "[2,6]");
    }

    #[test]
    fn test_insert_interval_extends_lower() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(4, 6), true);

        assert_eq!(intervals.dump(), "[4,6]");

        assert_eq!(intervals.insert_interval(1, 3), true);

        assert_eq!(intervals.dump(), "[1,6]");
    }

    #[test]
    fn test_insert_interval_extends_upper() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(4, 6), true);

        assert_eq!(intervals.dump(), "[4,6]");

        assert_eq!(intervals.insert_interval(7, 9), true);

        assert_eq!(intervals.dump(), "[4,9]");
    }

    #[test]
    fn test_insert_interval_is_before_and_is_first() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(4, 6), true);

        assert_eq!(intervals.dump(), "[4,6]");

        assert_eq!(intervals.insert_interval(1, 2), true);

        assert_eq!(intervals.dump(), "[1,2], [4,6]");
    }

    #[test]
    fn test_insert_interval_is_before_and_is_not_first() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(10, 12), true);

        assert_eq!(intervals.dump(), "[10,12]");

        assert_eq!(intervals.insert_interval(2, 3), true);

        assert_eq!(intervals.dump(), "[2,3], [10,12]");

        assert_eq!(intervals.insert_interval(5, 6), true);

        assert_eq!(intervals.dump(), "[2,3], [5,6], [10,12]");
    }

    #[test]
    fn test_insert_interval_is_after_and_is_last() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(10, 12), true);

        assert_eq!(intervals.dump(), "[10,12]");

        assert_eq!(intervals.insert_interval(18, 20), true);

        assert_eq!(intervals.dump(), "[10,12], [18,20]");
    }

    #[test]
    fn test_insert_interval_is_after_and_is_not_last() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(10, 12), true);

        assert_eq!(intervals.dump(), "[10,12]");

        assert_eq!(intervals.insert_interval(18, 20), true);

        assert_eq!(intervals.dump(), "[10,12], [18,20]");

        assert_eq!(intervals.insert_interval(15, 16), true);

        assert_eq!(intervals.dump(), "[10,12], [15,16], [18,20]");
    }

    #[test]
    fn test_insert_interval_joins_intervals() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(10, 12), true);

        assert_eq!(intervals.dump(), "[10,12]");

        assert_eq!(intervals.insert_interval(18, 20), true);

        assert_eq!(intervals.dump(), "[10,12], [18,20]");

        assert_eq!(intervals.insert_interval(13, 17), true);

        assert_eq!(intervals.dump(), "[10,20]");
    }

    #[test]
    fn test_insert_interval_sorting() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(4, 10), true);

        assert_eq!(intervals.dump(), "[4,10]");

        assert_eq!(intervals.insert_interval(5, 10), false);

        assert_eq!(intervals.dump(), "[4,10]");

        assert_eq!(intervals.insert_interval(5, 12), false);

        assert_eq!(intervals.dump(), "[4,10]");

        assert_eq!(intervals.insert_interval(12, 20), true);

        assert_eq!(intervals.dump(), "[4,10], [12,20]");

        assert_eq!(intervals.insert_interval(10, 12), false);

        assert_eq!(intervals.dump(), "[4,10], [12,20]");

        assert_eq!(intervals.insert_interval(9, 9), false);

        assert_eq!(intervals.dump(), "[4,10], [12,20]");

        assert_eq!(intervals.insert_interval(4, 9), false);

        assert_eq!(intervals.dump(), "[4,10], [12,20]");

        assert_eq!(intervals.insert_interval(8, 11), false);

        assert_eq!(intervals.dump(), "[4,10], [12,20]");
    }

    #[test]
    #[should_panic(expected = "Empty!")]
    fn test_remove_first_interval_when_empty() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.is_empty(), true);

        intervals.remove_first_interval();
    }

    #[test]
    fn test_remove_first_interval() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(4, 10), true);
        assert_eq!(intervals.insert_interval(12, 20), true);

        assert_eq!(intervals.dump(), "[4,10], [12,20]");

        let first = intervals.remove_first_interval();

        assert_eq!(first.lower(), 4);
        assert_eq!(first.upper(), 10);

        assert_eq!(intervals.dump(), "[12,20]");
    }

    #[test]
    #[should_panic(expected = "Empty!")]
    fn test_remove_first_value_when_empty() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.is_empty(), true);

        intervals.remove_first_value();
    }

    #[test]
    fn test_remove_first_value() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(4, 10), true);
        assert_eq!(intervals.insert_interval(12, 20), true);

        assert_eq!(intervals.dump(), "[4,10], [12,20]");

        assert_eq!(intervals.remove_first_value(), 4);

        assert_eq!(intervals.dump(), "[5,10], [12,20]");
    }

    #[test]
    fn test_remove_first_value_consumes_first_interval() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.insert_interval(4, 5), true);
        assert_eq!(intervals.insert_interval(12, 20), true);

        assert_eq!(intervals.dump(), "[4,5], [12,20]");

        assert_eq!(intervals.remove_first_value(), 4);

        assert_eq!(intervals.dump(), "[5], [12,20]");

        assert_eq!(intervals.remove_first_value(), 5);

        assert_eq!(intervals.dump(), "[12,20]");
    }

    #[test]
    fn test_remove_first_value_to_remove_all_values() {
        let mut intervals = Intervals::new();

        assert_eq!(intervals.insert_interval(u8::MIN, u8::MAX), true);

        assert_eq!(intervals.dump(), "[0,255]");

        for i in u8::MIN..u8::MAX {
            assert_eq!(intervals.remove_first_value(), i);
        }
        assert_eq!(intervals.remove_first_value(), u8::MAX);

        assert_eq!(intervals.dump(), "");
    }

    #[test]
    fn test_remove_value_from_lowest_value_of_interval() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.remove_value(4), false);

        assert_eq!(intervals.insert_interval(4, 10), true);
        assert_eq!(intervals.dump(), "[4,10]");

        assert_eq!(intervals.remove_value(4), true);
        assert_eq!(intervals.dump(), "[5,10]");
    }

    #[test]
    fn test_remove_value_from_inside_interval() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.remove_value(6), false);

        assert_eq!(intervals.insert_interval(4, 10), true);
        assert_eq!(intervals.dump(), "[4,10]");

        assert_eq!(intervals.remove_value(6), true);
        assert_eq!(intervals.dump(), "[4,5], [7,10]");
    }

    #[test]
    fn test_remove_value_from_highest_value_of_interval() {
        let mut intervals = Intervals::<u8>::new();

        assert_eq!(intervals.remove_value(10), false);

        assert_eq!(intervals.insert_interval(4, 10), true);
        assert_eq!(intervals.dump(), "[4,10]");

        assert_eq!(intervals.remove_value(10), true);
        assert_eq!(intervals.dump(), "[4,9]");
    }

    #[test]
    fn test_remove_value_to_remove_all_values() {
        let mut intervals = Intervals::new();

        assert_eq!(intervals.insert_interval(u8::MIN, u8::MAX), true);

        assert_eq!(intervals.dump(), "[0,255]");

        for i in u8::MIN..u8::MAX {
            assert_eq!(intervals.remove_value(i), true);
        }
        assert_eq!(intervals.remove_value(u8::MAX), true);

        assert_eq!(intervals.dump(), "");
    }

    #[test]
    fn test_remove_interval()
    {
        let mut intervals = Intervals::new();

        assert_eq!(intervals.insert_interval(u8::MIN, u8::MAX), true);

        assert_eq!(intervals.dump(), "[0,255]");

        intervals.remove_interval(10,30);               // middle

        assert_eq!(intervals.dump(), "[0,9], [31,255]");

        intervals.remove_interval(10,30);               // duplicate

        assert_eq!(intervals.dump(), "[0,9], [31,255]");

        intervals.remove_interval(50,60);

        assert_eq!(intervals.dump(), "[0,9], [31,49], [61,255]");

        intervals.remove_interval(70,90);

        assert_eq!(intervals.dump(), "[0,9], [31,49], [61,69], [91,255]");

        intervals.remove_interval(68,91);           // first/last

        assert_eq!(intervals.dump(), "[0,9], [31,49], [61,67], [92,255]");

        intervals.remove_interval(65,93);

        assert_eq!(intervals.dump(), "[0,9], [31,49], [61,64], [94,255]");

        intervals.remove_interval(50,93);           // spans

        assert_eq!(intervals.dump(), "[0,9], [31,49], [94,255]");

        intervals.remove_interval(50,93);

        assert_eq!(intervals.dump(), "[0,9], [31,49], [94,255]");

        intervals.remove_interval(200,255);         // end

        assert_eq!(intervals.dump(), "[0,9], [31,49], [94,199]");

        intervals.remove_interval(0,5);             // start

        assert_eq!(intervals.dump(), "[6,9], [31,49], [94,199]");

        intervals.remove_interval(7,198);           // most

        assert_eq!(intervals.dump(), "[6], [199]");

        intervals.remove_interval(0,255);           // all

        assert_eq!(intervals.dump(), "");
    }
}
