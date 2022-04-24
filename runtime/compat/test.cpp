#include "libwasmfaas.h"
#include "iostream"

int main()
{
    uint64_t runtime_id = initialize_runtime();
    uint64_t runtime2_id = initialize_runtime();

    char *sum_module_data = get_static_module_data(StaticModuleList::WasmSum);
    const char *module_name = register_module(runtime_id, "sum", sum_module_data);
    std::cout << "Registed " << module_name << " With " << runtime_id << '\n';

    char *div_module_data = get_static_module_data(StaticModuleList::WasmDiv);
    const char *module_name2 = register_module(runtime2_id, "div", div_module_data);
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

    auto result = execute_module(runtime_id, "sum", func);
    std::cout << runtime_id << " "
              << "Sum result " << result << '\n';

    func.name = "div";

    func.args[1] = WasmArg{
        "2",
        ArgType::I32,
    };

    auto result2 = execute_module(runtime2_id, "div", func);

    std::cout << runtime2_id << " "
              << "Div result " << result2 << '\n';

    free_ffi_string(sum_module_data);
    free_ffi_string(div_module_data);
}