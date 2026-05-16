//! `ChartContext` — the typed boundary wrapper that replaces the bare
//! `(serde_json::Value, fn)` tuple flowing from `dispatch_*` builders to
//! the renderers and the template engine.
//!
//! The chart data is still a `serde_json::Value` internally (the 21
//! per-tradition builders compose it that way, and the renderers read
//! it positionally), but it now travels as a *named type*:
//!
//! - `Deref<Target = Value>` so renderers keep reading `ctx["field"]`
//!   unchanged, and `&ChartContext` deref-coerces to `&Value` helpers.
//! - `Serialize` (transparent) so JSON serialization happens only at
//!   the template / `--print-context` boundary, never re-derived.

use serde::{Serialize, Serializer};
use serde_json::Value;

/// Typed chart context handed to a [`crate::cmd::render::ChartRenderer`].
pub(crate) struct ChartContext(Value);

impl ChartContext {
    /// Borrow the underlying JSON value.
    pub(crate) fn as_value(&self) -> &Value {
        &self.0
    }

    /// Consume into the underlying JSON value.
    pub(crate) fn into_value(self) -> Value {
        self.0
    }
}

impl From<Value> for ChartContext {
    fn from(v: Value) -> Self {
        ChartContext(v)
    }
}

impl std::ops::Deref for ChartContext {
    type Target = Value;
    fn deref(&self) -> &Value {
        &self.0
    }
}

impl Serialize for ChartContext {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}
