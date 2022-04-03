#include <emscripten.h>

// EM_IMPORT("sum") <- broken for some reason, compiling with -s ERROR_ON_UNDEFINED_SYMBOLS=0 instead
int sum(int a, int b);

int div(int a, int b);

EMSCRIPTEN_KEEPALIVE
int div_sum(int a, int b)
{
    return div(sum(a, b), sum(a, b));
}