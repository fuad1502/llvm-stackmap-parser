# LLVM Stack Map Parser

> [!NOTE]
> *TL;DR* This crate parses the LLVM Stack Map section of an object file into a
> data structure that's easy to consume. This data structure can be accessed by
> linking to the generated static library (`libsafepoints.a`) and using the
> provided header file (`src/safepoints.h`).

> [!TIP]
> See the garbage collection runtime of
> [oonta](https://github.com/fuad1502/oonta) for an example use case.

## Background

To connect LLVM IR code with a garbage collection (GC) runtime, LLVM provides
the [GC statepoint mechanism](https://llvm.org/docs/Statepoints.html) for
identifying garbage collection roots on the stack. After running the
`PlaceSafepoints` and `RewriteSafepointsForGC` passes on an LLVM IR (e.g. using
LLVM `opt`), a Stack Map section will be emitted in the resulting object code.
On Linux, this section name is `.llvm_stackmaps`:

```
# readelf -S benchmark/oonta.o
There are 13 section headers, starting at offset 0x2da0:
Section Headers:
  [Nr] Name              Type             Address           Offset
       Size              EntSize          Flags  Link  Info  Align
...
  [ 7] .llvm_stackmaps   PROGBITS         0000000000000000  000008b8
       00000000000013e8  0000000000000000   A       0     0     8
...
```
The Stack Map contains data that is written using the format provided
[here](https://llvm.org/docs/StackMaps.html#stackmap-format). The basic idea on
how to utilize the Stack Map for identifying GC roots on the stack is provided
here:

1. Code reaches a safepoint, either a memory allocation call or a call to a
   function within the `gc.safepoint_poll`, both of which should be provided by
   your garbage collection runtime.
2. Inside the function, determine the instruction pointer of the safepoint,
   e.g. using `libunwind`.
3. If you've determined the instruction pointer correctly, there should be a
   corresponding `StkMapRecord` for that instruction pointer inside the Stack
   Map. Once found, GC roots are identified from the `Location` entries within
   the `StkMapRecord`.
4. Unwind the stack. The instruction pointer should point to yet another
   safepoint. Perform step 3 again, repeat until you've reached the end of the
   stack.

As described above, the data inside the Stack Map section will be frequently
visited and therefore should ideally be parsed (at compile-time) into a data
structure that allows for easy traversing in a programming language of your
choice (at run-time). The purpose of this crate is exactly that.

This crate parses the LLVM Stack Map section and generates a static library
(`libsafepoints.a`). To access the data structure, simply link your program
with the library and use the provided header file (`src/safepoints.h`):

```c
enum LocationType { DIRECT, INDIRECT, CONSTANT };

struct Location {
  enum LocationType type;
  uint16_t reg;
  int32_t offset;
  size_t constant;
};

struct Safepoint {
  void *ip;
  uint64_t stack_size;
  uint32_t num_of_locations;
  struct Location *obj_locations;
};

extern struct Safepoint safepoints[];

extern int safepoints_len;

extern void *global_gcroots[];

extern int global_gcroots_len;
```
As seen above, it also provides pointers to global GC roots. To have it
populated, you must create a `.gcroots` section and place your global variables
in that section.

## Quick Start (Ubuntu)

```sh
cargo run -- <object file>
```
