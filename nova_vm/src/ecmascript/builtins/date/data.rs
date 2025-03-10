// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::time::SystemTime;

use crate::{
    SmallInteger,
    ecmascript::types::{IntoValue, Number, OrdinaryObject, Value},
    heap::{CompactionLists, HeapMarkAndSweep, WorkQueues},
};

/// ### [21.4.1.1 Time Values and Time Range](https://tc39.es/ecma262/#sec-time-values-and-time-range)
///
/// A Number can exactly represent all integers from -9,007,199,254,740,992
/// to 9,007,199,254,740,992 (21.1.2.8 and 21.1.2.6). A time value supports
/// a slightly smaller range of -8,640,000,000,000,000 to 8,640,000,000,000,000 milliseconds.
/// This yields a supported time value range of exactly -100,000,000 days
/// to 100,000,000 days relative to midnight at the beginning of 1 January 1970 UTC.
///
/// In that case, the time value can be either:
///
/// - Invalid, which is presented as `i64::MAX`
/// - An integer in the range of -8,640,000,000,000,000 to 8,640,000,000,000,000,
///   which is represented as a non-max `i64`, and can also fit in `SmallInteger`
#[derive(Debug, Clone, Copy)]
pub struct DateValue(i64);

impl DateValue {
    pub const NAN: Self = Self(i64::MAX);

    pub fn get_i64(self) -> Option<i64> {
        if self.0 == i64::MAX {
            None
        } else {
            Some(self.0 as i64)
        }
    }

    pub fn get_f64(self) -> Option<f64> {
        self.get_i64().map(|v| v as f64)
    }

    pub fn now() -> Self {
        let now = SystemTime::now();
        let now = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        Self(now as i64)
    }
}

/// ### [21.4.1.31 TimeClip ( time )](https://tc39.es/ecma262/#sec-timeclip)
///
/// The abstract operation TimeClip takes argument time (a Number) and returns
/// a Number. It calculates a number of milliseconds.
pub(crate) fn time_clip(time: f64) -> DateValue {
    // 1. If time is not finite, return NaN.
    if !time.is_finite() {
        return DateValue::NAN;
    }

    // 2. If abs(ℝ(time)) > 8.64 × 10**15, return NaN.
    if time.abs() > 8.64e15 {
        return DateValue::NAN;
    }

    // 3. Return 𝔽(! ToIntegerOrInfinity(time)).
    DateValue(time.trunc() as i64)
}

impl<'a> Into<Number<'a>> for DateValue {
    fn into(self) -> Number<'a> {
        if let Some(value) = self.get_f64() {
            // SAFETY: `value` is guaranteed to be in the range of `SmallInteger`.
            Number::Integer(SmallInteger::try_from(value).unwrap())
        } else {
            Number::nan()
        }
    }
}

impl<'a> IntoValue<'a> for DateValue {
    fn into_value(self) -> Value<'a> {
        let num: Number<'a> = self.into();
        num.into_value()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DateHeapData {
    pub(crate) object_index: Option<OrdinaryObject<'static>>,
    pub(crate) date: DateValue,
}

impl DateHeapData {
    pub(crate) fn new_invalid() -> Self {
        Self {
            object_index: None,
            date: DateValue::NAN,
        }
    }
}

impl HeapMarkAndSweep for DateHeapData {
    fn mark_values(&self, queues: &mut WorkQueues) {
        let Self {
            object_index,
            date: _,
        } = self;
        object_index.mark_values(queues);
    }

    fn sweep_values(&mut self, compactions: &CompactionLists) {
        let Self {
            object_index,
            date: _,
        } = self;
        object_index.sweep_values(compactions);
    }
}
