#include <emscripten.h>

// EM_IMPORT("sum") <- broken for some reason, compiling with -s ERROR_ON_UNDEFINED_SYMBOLS=0 instead
EMSCRIPTEN_KEEPALIVE
__attribute__((import_module("sum"))) int sum(int a, int b);

EMSCRIPTEN_KEEPALIVE
int div_sum(int a, int b)
{
    int sum_res = sum(a, b);
    int div_res = a / b;
    return sum_res + div_res;
}