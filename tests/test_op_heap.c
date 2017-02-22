/* Tests for the operations heap data structure. */

#include "lv2/ops.h"
#include "lv2/op-heap.h"

#include <glib.h>

#include <stdlib.h>
#include <string.h>

#define END G_MAXINT32

int* duplicate_and_sort_int_sequence (int* sequence_in, int n_elements) {
	int* copy = calloc(n_elements, sizeof(int));
	memcpy (copy, sequence_in, n_elements * sizeof(int));

	int compare_ints (const void* a, const void* b) {
		return ( *(const int*) a - *(const int*) b);
	}

	qsort (copy, n_elements, sizeof (int), compare_ints);

	return copy;
}

/* Tests pushing a sequence of values onto the heap, then removing them all again. */
void test_op_heap (int size, int* sequence) {
	OpHeap heap;
	Op op;

	int sequence_length = 0;
	int lowest_stored = G_MAXINT32;

	int* sequence_sorted;

	op_heap_init (&heap, size);

	/* Insert values in the sequence provided, and check lowest one is always
	 * at the front.
	 */
	for (int* i = sequence; *i != END; i++) {
		int value = *i;
		op.start = value;
		op_heap_push (&heap, op, value);

		sequence_length ++;

		if (sequence_length <= size) {
			lowest_stored = MIN (lowest_stored, value);
		}

		g_assert_cmpint (op_heap_front (&heap).start, ==, lowest_stored);
	}

	sequence_sorted = duplicate_and_sort_int_sequence (sequence, MIN (sequence_length, size));

	/* Pop each value stored. They should come out lowest first. */
	for (int i = 0; i < MIN (sequence_length, size); i++) {
		g_assert_cmpint (op_heap_front (&heap).start, ==, sequence_sorted[i]);
		op_heap_pop (&heap);
	}

	free (sequence_sorted);

	op_heap_free (&heap);

	return;
}

int main() {
	int seq1[] = {5, 1, 9, END};
	test_op_heap (3, seq1);
	test_op_heap (5, seq1);
	test_op_heap (2, seq1);

	int seq2[] = {5, 9, 3, 3, 7, 4, 10, 33, 1, END};
	test_op_heap (10, seq2);

	return 0;
}
