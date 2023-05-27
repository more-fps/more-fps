use crate::NonZeroDecimal;
use rust_decimal::Decimal;
use std::num::NonZeroUsize;

#[derive(Debug, PartialEq)]
pub struct TimeRange {
    pub start: Decimal,
    end: NonZeroDecimal,
}

impl TimeRange {
    pub fn duration(&self) -> NonZeroDecimal {
        // unwrapping because this struct can only be created via
        // TimeRange's next which has an if statement to check if start == *end

        // rounding to 3 decimal places to confirm we always extract at least 1 frame
        NonZeroDecimal::try_new((*self.end - self.start).round_dp(3)).unwrap()
    }
}

#[derive(Debug, PartialEq)]
pub struct TimeRanges {
    start: Decimal,
    max_step_size: NonZeroUsize,
    end: NonZeroDecimal,
}

impl TimeRanges {
    pub fn try_new<A, B, C>(start: A, max_step_size: B, end: C) -> Option<Self>
    where
        A: TryInto<Decimal>,
        B: TryInto<NonZeroUsize>,
        C: TryInto<Decimal>,
    {
        let start = start.try_into().ok()?;
        let end = NonZeroDecimal::try_new(end.try_into().ok()?)?;
        if start >= *end {
            return None;
        }
        let max_step_size = max_step_size.try_into().ok()?;
        Some(Self {
            start,
            max_step_size,
            end,
        })
    }
    pub fn end(&self) -> &Decimal {
        &self.end
    }
}

impl Iterator for TimeRanges {
    type Item = TimeRange;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.start;
        let end = self.end.get();
        if start == *end {
            return None;
        }
        let step_size: Decimal = self.max_step_size.get().try_into().ok()?;
        let next_start = vec![(start + step_size).round(), *end]
            .into_iter()
            .min()
            .unwrap();
        self.start = next_start;
        Some(TimeRange {
            start,
            end: NonZeroDecimal::try_new(next_start)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten() {
        // 10 seconds worth of frames
        let max_step_size = NonZeroUsize::new(10).unwrap();

        let time_ranges = vec![
            TimeRanges::try_new(0, max_step_size, "6.675000").unwrap(),
            TimeRanges::try_new("6.675000", max_step_size, "35").unwrap(),
        ];
        let actual = time_ranges
            .into_iter()
            .flatten()
            .collect::<Vec<TimeRange>>();
        let expected = vec![
            TimeRange {
                start: Decimal::ZERO,
                end: "6.675000".try_into().unwrap(),
            },
            TimeRange {
                start: "6.675000".try_into().unwrap(),
                end: "17".try_into().unwrap(),
            },
            TimeRange {
                start: "17".try_into().unwrap(),
                end: "27".try_into().unwrap(),
            },
            TimeRange {
                start: "27".try_into().unwrap(),
                end: "35".try_into().unwrap(),
            },
        ];
        assert_eq!(actual, expected);
    }
    #[test]
    fn start_equals_end() {
        let actual = TimeRanges::try_new(1, 2, 1);
        assert_eq!(actual, None);
    }

    #[test]
    fn start_gt_end() {
        let actual = TimeRanges::try_new(2, 2, 1);
        assert_eq!(actual, None);
    }
    #[test]
    fn next_doesnt_exceed_end() {
        let start = 0;

        // 10 seconds worth of frames
        let max_step_size = 10;

        let end = Decimal::from_str_exact("9.76").unwrap();

        let mut time_ranges = TimeRanges::try_new(start, max_step_size, end)
            .unwrap()
            .into_iter();
        assert_eq!(
            time_ranges.next().unwrap(),
            TimeRange {
                start: Decimal::ZERO,
                end: NonZeroDecimal::try_new(end).unwrap()
            }
        );
        assert!(time_ranges.next().is_none());
    }

    #[test]
    fn start_is_zero_multiple_next() {
        let start = 0;
        let max_step_size = 3;
        let end = Decimal::from_str_exact("9.76").unwrap();
        let mut time_ranges = TimeRanges::try_new(start, max_step_size, end)
            .unwrap()
            .into_iter();
        assert_eq!(
            time_ranges.next().unwrap(),
            TimeRange {
                start: Decimal::ZERO,
                end: NonZeroDecimal::try_new(max_step_size).unwrap()
            }
        );
        assert_eq!(
            time_ranges.next().unwrap(),
            TimeRange {
                start: max_step_size.try_into().unwrap(),
                end: NonZeroDecimal::try_new(max_step_size * 2).unwrap()
            }
        );
        assert_eq!(
            time_ranges.next().unwrap(),
            TimeRange {
                start: (max_step_size * 2).try_into().unwrap(),
                end: NonZeroDecimal::try_new(max_step_size * 3).unwrap()
            }
        );
        assert_eq!(
            time_ranges.next().unwrap(),
            TimeRange {
                start: (max_step_size * 3).try_into().unwrap(),
                end: NonZeroDecimal::try_new(end).unwrap()
            }
        );
        assert!(time_ranges.next().is_none());
    }

    #[test]
    fn start_is_decimal_multiple_next() {
        let start = Decimal::from_str_exact("522.981000").unwrap();
        let end = Decimal::from_str_exact("568.151000").unwrap();
        let max_step_size = 20;

        let mut time_ranges = TimeRanges::try_new(start, max_step_size, end)
            .unwrap()
            .into_iter();
        assert_eq!(
            time_ranges.next().unwrap(),
            TimeRange {
                start,
                end: NonZeroDecimal::try_new(543).unwrap()
            }
        );
        assert_eq!(
            time_ranges.next().unwrap(),
            TimeRange {
                start: Decimal::from_str_exact("543").unwrap(),
                end: NonZeroDecimal::try_new(563).unwrap()
            }
        );
        assert_eq!(
            time_ranges.next().unwrap(),
            TimeRange {
                start: Decimal::from_str_exact("563").unwrap(),
                end: NonZeroDecimal::try_new(end).unwrap()
            }
        );
    }
}
