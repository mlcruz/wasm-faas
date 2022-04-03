#include <emscripten.h>

EMSCRIPTEN_KEEPALIVE
int div(int a, int b)
{
    return a / b;
}
