#include "libwasmfaas.h"
#include "iostream"

int main()
{
    uint64_t runtime_id = initialize_runtime();
    const char *module_name = register_module(runtime_id, ModuleList::WasmSum);
    std::cout << "Registed " << module_name << " With " << runtime_id << '\n';

    auto func = WasmFunction{};
    func.name = "sum";

    func.args[0] = WasmArg{
        "10",
        ArgType::I32,
    };

    func.args[1] = WasmArg{
        "10",
        ArgType::I32,
    };

    auto result = execute_module(runtime_id, ModuleList::WasmSum, func);

    std::cout << "Sum result " << result << '\n';

    free_ffi_string((char *)module_name);
}