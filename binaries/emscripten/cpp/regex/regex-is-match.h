#ifndef __BINDINGS_REGEX_IS_MATCH_H
#define __BINDINGS_REGEX_IS_MATCH_H
#ifdef __cplusplus
extern "C"
{
#endif

#include <stdint.h>
#include <stdbool.h>

  typedef struct
  {
    char *ptr;
    size_t len;
  } regex_is_match_string_t;

  void regex_is_match_string_set(regex_is_match_string_t *ret, const char *s);
  void regex_is_match_string_dup(regex_is_match_string_t *ret, const char *s);
  void regex_is_match_string_free(regex_is_match_string_t *ret);
  bool regex_is_match(regex_is_match_string_t *text, regex_is_match_string_t *pattern);
#ifdef __cplusplus
}
#endif
#endif
