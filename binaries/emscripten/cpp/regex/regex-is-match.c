#include <stdlib.h>
#include <regex-is-match.h>

__attribute__((weak, export_name("canonical_abi_realloc"))) void *canonical_abi_realloc(
    void *ptr,
    size_t orig_size,
    size_t org_align,
    size_t new_size)
{
  void *ret = realloc(ptr, new_size);
  if (!ret)
    abort();
  return ret;
}

__attribute__((weak, export_name("canonical_abi_free"))) void canonical_abi_free(
    void *ptr,
    size_t size,
    size_t align)
{
  free(ptr);
}
#include <string.h>

void regex_is_match_string_set(regex_is_match_string_t *ret, const char *s)
{
  ret->ptr = (char *)s;
  ret->len = strlen(s);
}

void regex_is_match_string_dup(regex_is_match_string_t *ret, const char *s)
{
  ret->len = strlen(s);
  ret->ptr = canonical_abi_realloc(NULL, 0, 1, ret->len);
  memcpy(ret->ptr, s, ret->len);
}

void regex_is_match_string_free(regex_is_match_string_t *ret)
{
  canonical_abi_free(ret->ptr, ret->len, 1);
  ret->ptr = NULL;
  ret->len = 0;
}
__attribute__((export_name("regex-is-match")))
int32_t
__wasm_export_regex_is_match_regex_is_match(int32_t arg, int32_t arg0, int32_t arg1, int32_t arg2)
{
  regex_is_match_string_t arg3 = (regex_is_match_string_t){(char *)(arg), (size_t)(arg0)};
  regex_is_match_string_t arg4 = (regex_is_match_string_t){(char *)(arg1), (size_t)(arg2)};
  bool ret = regex_is_match(&arg3, &arg4);
  int32_t variant;
  switch ((int32_t)ret)
  {
  case 0:
  {
    variant = 0;
    break;
  }
  case 1:
  {
    variant = 1;
    break;
  }
  }
  return variant;
}
