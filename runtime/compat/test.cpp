#include "libwasmfaas.h"
#include "iostream"

int main()
{
    uint64_t runtime_id = initialize_runtime();
    uint64_t runtime2_id = initialize_runtime();

    const char *module_name = register_module(runtime_id, ModuleList::WasmSum);
    std::cout << "Registed " << module_name << " With " << runtime_id << '\n';

    const char *module_name2 = register_module(runtime2_id, ModuleList::WasmSum);
    std::cout << "Registed " << module_name2 << " With " << runtime2_id << '\n';

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
    std::cout << "Sum result1 " << result << '\n';

    func.args[1] = WasmArg{
        "100",
        ArgType::I32,
    };

    auto result2 = execute_module(runtime2_id, ModuleList::WasmSum, func);
    std::cout << "Sum result2 " << result2 << '\n';

    free_ffi_string((char *)module_name);
}