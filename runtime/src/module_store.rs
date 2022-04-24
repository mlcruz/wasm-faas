use std::collections::HashMap;

use wasmer::{imports, Instance};
use wasmer::{ImportObject, Module};
use wasmer_wasi::{WasiEnv, WasiStateBuilder};

#[derive(Debug, Clone)]
pub struct ModulePackage {
    pub module: Module,
    pub wasi: bool,
    pub imports: ImportObject,
}

impl ModulePackage {
    pub fn new(module: &Module, store: &ModuleStore, wasi: bool) -> anyhow::Result<Self> {
        let mut imports = module.imports().collect::<Vec<_>>();
        imports.sort_unstable_by_key(|i| i.module().to_string());
        imports.dedup();

        let mut import_object = if wasi {
            let wasi_state = WasiStateBuilder::default().build()?;
            let mut wasi_env = WasiEnv::new(wasi_state);
            let imports = wasi_env.import_object(&module)?;
            imports
        } else {
            let imports = imports! {};
            imports
        };

        for import in imports {
            if let Some(imported_module) = store.get(import.module()) {
                let imports = &imported_module.imports;
                let module = &imported_module.module;
                let instance = Instance::new(module, imports)?;
                let exports = instance.exports;
                import_object.register(import.name(), exports);
            }
        }

        Ok(ModulePackage {
            module: module.clone(),
            wasi,
            imports: import_object,
        })
    }
}

#[derive(Default)]
pub struct ModuleStore {
    store: HashMap<String, ModulePackage>,
}

impl ModuleStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: impl AsRef<str>, module: Module, wasi: bool) -> anyhow::Result<()> {
        let name = name.as_ref().to_string();
        let package = ModulePackage::new(&module, self, wasi)?;
        self.store.insert(name, package);

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&ModulePackage> {
        self.store.get(name)
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.store.contains_key(name)
    }
}
