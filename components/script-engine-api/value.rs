// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A value-level scripting contract beside the JS-shaped [`ScriptEngine`].
//!
//! [`ScriptEngine`] is a JavaScript VM's contract: realms, `WindowProxy`
//! traps, reflectors carrying DOM nodes, host promises. A script that only
//! computes over values (a scene rule, a style expression) needs none of
//! that, and an engine without a DOM (rhai, a configuration language) cannot
//! sensibly implement it. [`ValueEngine`] is what such scripts share: neutral
//! values in and out, host functions over neutral values, and evaluation under
//! a [`Budget`]. Engine-native values stay inside the backend, as they do for
//! [`ScriptEngine`]. (mere's Scenograph editor plan, rulings SE39 and SE40.)
//!
//! A host that needs reproducible results registers its maths as host
//! functions on a deterministic library and loads no engine-built-in maths,
//! so a rule gives the same bits natively and in the browser.
//!
//! [`ScriptEngine`]: crate::ScriptEngine

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::Budget;

/// A value crossing the boundary, copied in each direction. Maps are ordered,
/// so a value's encoding and iteration do not depend on a hash seed.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptValue {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<ScriptValue>),
    Map(BTreeMap<String, ScriptValue>),
}

impl ScriptValue {
    /// The value as a float: a `Float`, or an `Int` widened.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Int(value) => Some(*value as f64),
            _ => None,
        }
    }

    /// A short name for the value's kind, for errors.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Bool(_) => "bool",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Str(_) => "string",
            Self::List(_) => "list",
            Self::Map(_) => "map",
        }
    }
}

/// A host function a script may call: neutral arguments in, a neutral value
/// or a message out. Shared, so one registration can serve many engines.
pub type HostFunction = Arc<dyn Fn(&[ScriptValue]) -> Result<ScriptValue, String> + Send + Sync>;

/// Why an evaluation did not return a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueError {
    /// The [`Budget`] ran out before the script finished.
    Budget,
    /// The script failed to parse or raised an error, in the engine's words.
    Script(String),
    /// A host function refused, in its own words.
    Host(String),
    /// The script returned something the boundary cannot carry.
    Value(String),
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Budget => f.write_str("the script ran past its budget"),
            Self::Script(message) => write!(f, "script error: {message}"),
            Self::Host(message) => write!(f, "host function refused: {message}"),
            Self::Value(message) => write!(f, "value not carried: {message}"),
        }
    }
}

impl std::error::Error for ValueError {}

/// An engine that evaluates scripts over neutral values.
///
/// The script sees only the globals and host functions given to it: what a
/// script may do is the set of functions registered, so one engine serves a
/// trusted author and a stranger alike with different sets.
pub trait ValueEngine {
    /// Bind `name` to `value` for every later evaluation.
    fn set_global(&mut self, name: &str, value: ScriptValue) -> Result<(), ValueError>;

    /// Expose `function` to scripts as `name`, taking `arity` arguments.
    fn register(
        &mut self,
        name: &str,
        arity: usize,
        function: HostFunction,
    ) -> Result<(), ValueError>;

    /// Evaluate `source` and return its value. [`Budget::Steps`] bounds the
    /// work in the backend's own unit (operations, instructions or ticks), and
    /// running past it is [`ValueError::Budget`], never a hang.
    fn eval(&mut self, source: &str, budget: Budget) -> Result<ScriptValue, ValueError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A toy backend: its "script" is a host function name, evaluated with the
    /// globals `a` and `b` as arguments; one step per call.
    #[derive(Default)]
    struct Toy {
        globals: BTreeMap<String, ScriptValue>,
        functions: BTreeMap<String, (usize, HostFunction)>,
    }

    impl ValueEngine for Toy {
        fn set_global(&mut self, name: &str, value: ScriptValue) -> Result<(), ValueError> {
            self.globals.insert(name.to_string(), value);
            Ok(())
        }

        fn register(
            &mut self,
            name: &str,
            arity: usize,
            function: HostFunction,
        ) -> Result<(), ValueError> {
            self.functions.insert(name.to_string(), (arity, function));
            Ok(())
        }

        fn eval(&mut self, source: &str, budget: Budget) -> Result<ScriptValue, ValueError> {
            if budget == Budget::Steps(0) {
                return Err(ValueError::Budget);
            }
            let (arity, function) = self
                .functions
                .get(source.trim())
                .ok_or_else(|| ValueError::Script(format!("no function `{source}`")))?;
            let args: Vec<ScriptValue> = ["a", "b"]
                .iter()
                .take(*arity)
                .map(|name| {
                    self.globals
                        .get(*name)
                        .cloned()
                        .unwrap_or(ScriptValue::Unit)
                })
                .collect();
            function(&args).map_err(ValueError::Host)
        }
    }

    fn pow() -> HostFunction {
        Arc::new(|args: &[ScriptValue]| {
            let (Some(x), Some(y)) = (args[0].as_float(), args[1].as_float()) else {
                return Err(format!(
                    "pow wants floats, got {} and {}",
                    args[0].kind(),
                    args[1].kind()
                ));
            };
            Ok(ScriptValue::Float(x.powf(y)))
        })
    }

    #[test]
    fn an_engine_is_usable_behind_a_trait_object() {
        let mut engine: Box<dyn ValueEngine> = Box::new(Toy::default());
        engine.set_global("a", ScriptValue::Int(2)).unwrap();
        engine.set_global("b", ScriptValue::Float(10.0)).unwrap();
        engine.register("pow", 2, pow()).unwrap();
        assert_eq!(
            engine.eval("pow", Budget::Steps(1)),
            Ok(ScriptValue::Float(1024.0))
        );
    }

    #[test]
    fn budget_script_and_host_failures_are_told_apart() {
        let mut engine = Toy::default();
        engine.register("pow", 2, pow()).unwrap();
        assert_eq!(
            engine.eval("pow", Budget::Steps(0)),
            Err(ValueError::Budget)
        );
        assert!(matches!(
            engine.eval("nothing", Budget::Unbounded),
            Err(ValueError::Script(_))
        ));
        engine
            .set_global("a", ScriptValue::Str("x".into()))
            .unwrap();
        engine.set_global("b", ScriptValue::Unit).unwrap();
        assert_eq!(
            engine.eval("pow", Budget::Unbounded),
            Err(ValueError::Host(
                "pow wants floats, got string and unit".into()
            ))
        );
    }

    #[test]
    fn maps_are_ordered() {
        let map = ScriptValue::Map(BTreeMap::from([
            ("b".to_string(), ScriptValue::Int(2)),
            ("a".to_string(), ScriptValue::Int(1)),
        ]));
        let ScriptValue::Map(entries) = map else {
            unreachable!()
        };
        assert_eq!(entries.keys().collect::<Vec<_>>(), ["a", "b"]);
    }
}
