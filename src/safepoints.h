#ifndef __SAFEPOINTS_H
#define __SAFEPOINTS_H

#include <stdint.h>

struct Safepoint {
  void *location;
  uint64_t stack_size;
  uint64_t *obj_stack_offsets;
};

extern struct Safepoint safepoints[];

extern int safepoints_len;

#endif // __SAFEPOINTS_H
