/* Auto-generated from compiler/runtime/runtime.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/runtime/runtime.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_RUNTIME_H
#define T27_RUNTIME_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define T27_STACK_SIZE    4096
#define T27_HEAP_SIZE     65536
#define T27_MAX_THREADS   8
#define T27_MAX_CHANNELS  16

typedef enum { THREAD_IDLE = 0, THREAD_RUNNING = 1, THREAD_BLOCKED = 2 } T27ThreadState;

typedef struct {
    size_t stack_base;
    size_t stack_ptr;
    size_t heap_base;
    size_t heap_ptr;
    size_t heap_end;
    T27ThreadState thread_states[T27_MAX_THREADS];
    size_t thread_sp[T27_MAX_THREADS];
    size_t channel_sizes[T27_MAX_CHANNELS];
    size_t exception_handler;
    uint8_t exception_code;
    size_t cycle_counter;
    size_t instruction_counter;
} T27Runtime;

void t27_runtime_init(T27Runtime *rt);
int  t27_runtime_execute(T27Runtime *rt, size_t entry);
void t27_runtime_shutdown(T27Runtime *rt);

#ifdef __cplusplus
}
#endif

#endif /* T27_RUNTIME_H */
