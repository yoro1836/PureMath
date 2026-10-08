use crate::diagnostics::Diagnostic;
use crate::value::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Default)]
pub struct Environment {
    pub(crate) values: HashMap<String, Value>,
    pub(crate) loading: Vec<PathBuf>,
    pub(crate) loaded_modules: HashSet<PathBuf>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn define_value(
        &mut self,
        name: impl Into<String>,
        value: Value,
    ) -> Result<(), Diagnostic> {
        let name = name.into();
        if self.values.contains_key(&name) {
            return Err(Diagnostic::new(format!("name already defined: {}", name)));
        }
        self.values.insert(name, value);
        Ok(())
    }

    pub fn define_function(
        &mut self,
        name: impl Into<String>,
        function: crate::value::FunctionValue,
    ) -> Result<(), Diagnostic> {
        let name = name.into();
        if self.values.contains_key(&name) {
            return Err(Diagnostic::new(format!("name already defined: {}", name)));
        }
        self.values
            .insert(name, Value::Function(std::rc::Rc::new(function)));
        Ok(())
    }
}
