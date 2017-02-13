#include <stdbool.h>
#include <stdlib.h>

#include "op-heap.h"

#define CMP(a, b) ((a) <= (b))

bool op_heap_init (OpHeap *heap, int size)
{
	heap->size = size;
	heap->count = 0;
	heap->data = calloc (size, sizeof(OpHeapEntry));

	if (heap->data == NULL)
		return false;

	return true;
}

void op_heap_free (OpHeap *heap)
{
	free (heap->data);
}

// Inserts element to the heap
bool op_heap_push (OpHeap *heap, Op op, int key)
{
	unsigned int index, parent;

	if (heap->count == heap->size) {
		/* No room at the inn */
		return false;
	}

	/* We start at the far end of the heap, and iterate upwards through the
	 * branches of its tree moving one item down a level at each step. When
	 * the first item on the parent level has a key lower than the item we're
	 * inserting, we stop because it's proved that our new item does not have
	 * the lowest key.
	 */
	for(index = heap->count++; index; index = parent) {
		parent = (index - 1) >> 1;
		if (CMP (heap->data[parent].key, key))
			break;
		heap->data[index] = heap->data[parent];
	}
	heap->data[index].op = op;
	heap->data[index].key = key;

	return true;
}

// Removes the smallest element from the heap
void op_heap_pop(OpHeap *heap) {
	unsigned int index, swap, other;

	// Remove the last element
	OpHeapEntry temp = heap->data[--heap->count];

	// Reorder the elements
	for(index = 0; 1; index = swap)
	{
		// Find the child to swap with
		swap = (index << 1) + 1;
		if (swap >= heap->count)
			break; // If there are no children, the heap is reordered
		other = swap + 1;
		if ((other < heap->count) && CMP (heap->data[other].key, heap->data[swap].key))
			swap = other;

		if (CMP(temp.key, heap->data[swap].key))
			break; // If the smaller child is greater than or equal to its parent, the heap is reordered

		heap->data[index] = heap->data[swap];
	}
	heap->data[index] = temp;
}
