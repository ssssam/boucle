/* Operation heap: holds control events that are active or queued.
 *
 * Inserts to the heap are sorted so that element with the lowest 'key' is *
 * always at the front of the heap. For the record heap we want the op with the
 * earliest start time at the front, and for the play heap we want the op with
 * the earliest end time at the front.
 *
 * Based on: <https://gist.github.com/martinkunev/1365481>
 */

#ifndef __OP_HEAP_H__
#define __OP_HEAP_H__

#include "ops.h"

#include <stdbool.h>

typedef struct {
	int key;
	Op op;
} OpHeapEntry;

typedef struct {
	unsigned int size;
	unsigned int count;
	OpHeapEntry *data;
} OpHeap;

bool op_heap_init (OpHeap *heap, int size);
void op_heap_free (OpHeap *heap);

bool op_heap_push (OpHeap *heap, Op op, int key);
void op_heap_pop (OpHeap *heap);

#define op_heap_front(h)  ((h)->data[0].op)

#endif
