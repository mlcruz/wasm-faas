use std::collections::HashMap;

use wasmer::Module;

#[derive(Default)]
pub struct KernelStore {
    store: HashMap<String, Module>,
}

impl KernelStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: impl AsRef<str>, module: Module) {
        let name = name.as_ref().to_string();

        self.store.insert(name, module);
    }

    pub fn get(&self, name: &str) -> Option<&Module> {
        self.store.get(name)
    }
}
