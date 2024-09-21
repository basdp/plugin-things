use std::{any::Any, sync::{atomic::AtomicI64, Arc}};

use portable_atomic::{AtomicF64, Ordering};

use crate::ParameterId;

use super::{error::Error, formatter::ParameterFormatter, info::ParameterInfo, parameter::{Parameter, ParameterPlain}, range::ParameterRange, ParameterValue};

pub type ValueChangedCallback = Arc<dyn Fn(i64) + Send + Sync>;

pub struct IntParameter {
    info: ParameterInfo,
    value: AtomicI64,
    normalized_modulation: AtomicF64,
    range: Arc<dyn ParameterRange<i64>>,
    formatter: Arc<dyn ParameterFormatter<i64>>,
    value_changed: Option<ValueChangedCallback>,
}

impl IntParameter {
    pub fn new(id: impl Into<ParameterId>, name: impl Into<String>, range: Arc<dyn ParameterRange<i64>>) -> Self {
        let info = ParameterInfo::new(id.into(), name.into())
            .with_steps(range.steps());
        let value = range.normalized_to_plain(0.0);

        Self {
            info,
            value: value.into(),
            normalized_modulation: 0.0.into(),
            range,
            formatter: Arc::new(IntFormatter::new("")),
            value_changed: None,
        }
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.info = self.info.with_path(path);
        self
    }

    pub fn with_default_value(mut self, value: i64) -> Self {
        let default_normalized_value = self.range.plain_to_normalized(value).unwrap();
        self.info = self.info.with_default_normalized_value(default_normalized_value);
        self.value.store(value, Ordering::Release);
        self
    }

    pub fn with_formatter(mut self, formatter: Arc<dyn ParameterFormatter<i64>>) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn on_value_changed(mut self, value_changed: ValueChangedCallback) -> Self {
        self.value_changed = Some(value_changed);
        self
    }

    pub fn set_value(&self, value: i64) {
        let value = self.range.clamp(value);
        self.value.store(value, Ordering::Release);
        self.changed();
    }

    pub fn default_value(&self) -> i64 {
        self.range.normalized_to_plain(self.info.default_normalized_value())
    }

    pub fn to_string(&self) -> String {
        self.formatter.value_to_string(self.plain())
    }

    fn changed(&self) {
        if let Some(on_value_changed) = self.value_changed.as_ref() {
            on_value_changed(self.plain());
        }
    }
}

impl Clone for IntParameter {
    fn clone(&self) -> Self {
        Self {
            info: self.info.clone(),
            value: self.value.load(Ordering::Acquire).into(),
            normalized_modulation: self.normalized_modulation.load(Ordering::Acquire).into(),
            range: self.range.clone(),
            formatter: self.formatter.clone(),
            value_changed: self.value_changed.clone(),
        }
    }
}

impl Parameter for IntParameter {
    fn info(&self) -> &ParameterInfo {
        &self.info
    }

    fn normalized_value(&self) -> ParameterValue {
        self.range.plain_to_normalized(self.value.load(Ordering::Acquire)).unwrap()
    }

    fn set_normalized_value(&self, normalized: ParameterValue) -> Result<(), Error> {
        let normalized = f64::clamp(normalized, 0.0, 1.0);
        self.set_value(self.range.normalized_to_plain(normalized));
        Ok(())
    }

    fn normalized_modulation(&self) -> ParameterValue {
        self.normalized_modulation.load(Ordering::Acquire)
    }

    fn set_normalized_modulation(&self, amount: ParameterValue) {
        self.normalized_modulation.store(amount, Ordering::Release);
        self.changed();
    }

    fn normalized_to_string(&self, value: ParameterValue) -> String {
        let plain = self.range.normalized_to_plain(value);
        self.formatter.value_to_string(plain)
    }

    fn string_to_normalized(&self, string: &str) -> Option<ParameterValue> {
        let Some(plain) = self.formatter.string_to_value(string) else {
            return None;
        };
        
        self.range.plain_to_normalized(plain)
    }

    fn serialize_value(&self) -> ParameterValue {
        self.value.load(Ordering::Acquire) as ParameterValue
    }

    fn deserialize_value(&self, value: ParameterValue) -> Result<(), Error> {
        self.set_value(value as _);
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self as _
    }
}

impl ParameterPlain for IntParameter {
    type Plain = i64;
    
    fn normalized_to_plain(&self, normalized: ParameterValue) -> i64 {
        let normalized = normalized.clamp(0.0, 1.0);
        self.range.normalized_to_plain(normalized)
    }

    fn plain_to_normalized(&self, plain: i64) -> ParameterValue {
        self.range.plain_to_normalized(plain).unwrap()
    }
}

#[derive(Clone)]
pub struct IntRange {
    min: i64,
    max: i64,
}

impl IntRange {
    pub const fn new(min: i64, max: i64) -> Self {
        Self {
            min,
            max,
        }
    }
}

impl ParameterRange<i64> for IntRange {
    fn clamp(&self, value: i64) -> i64 {
        i64::clamp(value, self.min, self.max)
    }

    fn steps(&self) -> usize {
        i64::abs(self.max - self.min) as usize
    }

    fn plain_to_normalized(&self, plain: i64) -> Option<ParameterValue> {
        if plain < self.min || plain > self.max {
            return None;
        }

        Some((plain - self.min) as f64 / self.steps() as f64)
    }

    fn normalized_to_plain(&self, normalized: ParameterValue) -> i64 {
        let steps = self.steps();
        self.min + i64::min(steps as i64, (normalized * (steps + 1) as f64) as i64)
    }
}

pub struct IntFormatter {
    unit: &'static str,
}

impl IntFormatter {
    pub fn new(unit: &'static str) -> Self {
        Self {
            unit,
        }
    }
}

impl ParameterFormatter<i64> for IntFormatter {
    fn value_to_string(&self, value: i64) -> String {
        format!("{value}{}", self.unit)
    }

    fn string_to_value(&self, string: &str) -> Option<i64> {
        let string = string.strip_suffix(self.unit).unwrap_or(string);
        string.parse().ok()
    }
}