use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use starlark::environment::{Globals, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};

use crate::driver::DriverManifest;
use crate::paths;
use crate::{GameError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriverCall {
    pub function: String,
    #[serde(default)]
    pub args: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriverCallResult {
    pub result: Value,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeclaredScriptFunction {
    pub script: PathBuf,
    pub function: String,
    pub mutates: bool,
}

pub fn declared_function(
    driver: &DriverManifest,
    function: &str,
) -> Result<DeclaredScriptFunction> {
    let Some(declared) = driver.functions.get(function) else {
        return Err(GameError::Script(format!(
            "driver function is not declared: {function}"
        )));
    };
    if declared.mutates {
        return Err(GameError::Script(format!(
            "driver function may not mutate save state: {function}"
        )));
    }
    Ok(DeclaredScriptFunction {
        script: declared.script.clone(),
        function: declared.function.clone(),
        mutates: declared.mutates,
    })
}

pub fn run_driver_function(
    driver_root: impl AsRef<Path>,
    driver: &DriverManifest,
    call: DriverCall,
) -> Result<DriverCallResult> {
    let driver_root = paths::canonicalize_root(driver_root, "driver root")?;
    let declared = declared_function(driver, &call.function)?;
    let script_path =
        paths::resolve_existing_under(&driver_root, &declared.script, "driver script")?;
    let source = std::fs::read_to_string(&script_path).map_err(|source| GameError::Read {
        path: script_path.clone(),
        source,
    })?;
    let ast = AstModule::parse(&script_path.to_string_lossy(), source, &Dialect::Standard)
        .map_err(|err| GameError::Script(err.to_string()))?;
    let globals = Globals::standard();
    let module = Module::new();
    let mut eval = Evaluator::new(&module);
    eval.eval_module(ast, &globals)
        .map_err(|err| GameError::Script(err.to_string()))?;
    let function = module.get(&declared.function).ok_or_else(|| {
        GameError::Script(format!(
            "starlark function not found: {}",
            declared.function
        ))
    })?;

    let heap = module.heap();
    let named_values = call
        .args
        .iter()
        .map(|(name, value)| (name.as_str(), heap.alloc(value)))
        .collect::<Vec<_>>();
    let result = eval
        .eval_function(function, &[], &named_values)
        .map_err(|err| GameError::Script(err.to_string()))?
        .to_json_value()
        .map_err(|err| GameError::Script(err.to_string()))?;

    Ok(DriverCallResult {
        result,
        warnings: Vec::new(),
    })
}
