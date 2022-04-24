#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class ArgType {
  /// Signed 32 bit integer.
  I32,
  /// Signed 64 bit integer.
  I64,
  /// Floating point 32 bit integer.
  F32,
  /// Floating point 64 bit integer.
  F64,
  /// A 128 bit number.
  V128,
  /// A reference to opaque data in the Wasm instance.
  ExternRef,
  /// A reference to a Wasm function.
  FuncRef,
};

enum class ModuleList {
  WasmDiv,
  WasmSum,
};

struct WasmArg {
  const char *value;
  ArgType arg_type;
};

struct WasmFunction {
  const char *name;
  WasmArg args[2];
};

extern "C" {

uint64_t initialize_runtime();

void register_module(uint64_t runtime_id, ModuleList module);

void execute_module(uint64_t runtime_id, ModuleList module, WasmFunction function);

} // extern "C"
