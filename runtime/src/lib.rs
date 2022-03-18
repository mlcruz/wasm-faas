use serde::{Deserialize, Serialize};

pub mod kernel_store;

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedFunction {
    pub name: String,
    pub data_base64: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmFunction {
    pub name: String,
    pub args: Vec<wasmer_runtime::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteFunction {
    pub name: String,
    pub function: WasmFunction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    RegisterFunction(NamedFunction),
    ExecuteFunction(ExecuteFunction),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
