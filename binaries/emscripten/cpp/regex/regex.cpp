#include <emscripten.h>
#include <string>
#include <regex>
#include "regex-is-match.h"

using namespace std;

EMSCRIPTEN_KEEPALIVE
bool regex_is_match(regex_is_match_string_t *text, regex_is_match_string_t *pattern)
{
    return regex_match(text->ptr, regex(pattern->ptr));
}
