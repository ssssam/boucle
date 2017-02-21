#include <lv2/lv2plug.in/ns/lv2core/lv2.h>
#include <lv2/lv2plug.in/ns/ext/atom/atom.h>
#include <lv2/lv2plug.in/ns/ext/atom/util.h>
#include <lv2/lv2plug.in/ns/ext/log/log.h>
#include <lv2/lv2plug.in/ns/ext/log/logger.h>
#include <lv2/lv2plug.in/ns/ext/midi/midi.h>
#include <lv2/lv2plug.in/ns/ext/urid/urid.h>

#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#include "ops.h"
#include "op-heap.h"

#define BOUCLE_URI "http://afuera.me.uk/boucle"

/* Maximum memory usage: 11MB, based on 96KHz sample rate */
#define BUFFER_LENGTH  60 * 48000 /* Frames */

#define MAX_QUEUED_OPS 256
#define MAX_ACTIVE_OPS 256

#define MIN(a,b) ({ __typeof__ (a) _a = (a); \
               __typeof__ (b) _b = (b); \
               _a < _b ? _a : _b; })
#define MAX(a,b) ({ __typeof__ (a) _a = (a); \
               __typeof__ (b) _b = (b); \
               _a > _b ? _a : _b; })
#define CLAMP(x,lo,hi) MIN(hi, MAX(lo, x))

static void cleanup (LV2_Handle instance);

typedef enum {
	PORT_BOUCLE_CONTROL = 0,
	PORT_INPUT  = 1,
	PORT_OUTPUT = 2,
	PORT_LOOP_LENGTH = 3,
	PORT_MIDI_BRIDGE = 4,
	PORT_TEMPO = 5
} Port;


typedef struct {
	/* LV2 extensions */
	LV2_Log_Log* log;
	LV2_Log_Logger logger;

	LV2_URID_Map* map;

	/* URI mappings, from the urid module. Used so we can receive integer codes
	 * instead of long URI strings for each MIDI event.
	 */
	struct {
		LV2_URID midi_MidiEvent;
	} uris;

	/* LV2 connections */
	const LV2_Atom_Sequence* control;
	const float* input;
	float* output;
	const float* loop_length; /* in frames (we're in mono, so 1 frame = 1 sample) */

	const LV2_Atom_Sequence* midi;
	const float* tempo;  /* bpm */

	/* Internals */
	double samplerate;

	float *buffer;
	bool buffer_full;

	size_t record_head; /* in samples */
	size_t play_head;

	int reverser_offset; /* For reverse operatios */

	int n_active_notes;

	/* These heaps store ops that are waiting to be played, or are currently
	 * active.
	 *
	 * We need two heaps of each type because our time wraps around back to
	 * zero when we hit the end of the loop buffer. So it's not always true
	 * that the earliest op we have to execute has the lowest timestamp.
	 * By switching heaps whenever time wraps around, we can keep the heaps
	 * ordered, and we just need to track which heap is older currently. This
	 * does mean that the maximum duration for an operation is 2 x the loop
	 * length.
	 */
	OpHeap queued_ops[2];
	OpHeap active_ops[2];
	int primary_queue_heap, primary_active_heap;
} Boucle;

static bool get_host_features (Boucle *self,
                               const LV2_Feature* const *features)
{
	const int STRCMP_MATCH = 0;
	for (int i = 0; features[i]; ++i) {
		if (strcmp (features[i]->URI, LV2_LOG__log) == STRCMP_MATCH) {
			self->log = (LV2_Log_Log*)features[i]->data;
		}

		if (strcmp (features[i]->URI, LV2_URID__map) == STRCMP_MATCH) {
			self->map = (LV2_URID_Map*)features[i]->data;
		}
	}

	if (!self->map || !self->log) {
		fprintf(stderr, "Boucle: required features (map, log) not provided.\n");
		return false;
	} else {
		return true;
	}
}

static LV2_Handle
instantiate (const LV2_Descriptor* descriptor,
             double rate,
             const char* bundle_path,
             const LV2_Feature* const *features)
{
	Boucle* self = malloc (sizeof (Boucle));
	bool ok = true;

	if (!self)
		return NULL;

	memset (self, 0, sizeof(Boucle));

	if (! get_host_features (self, features)) {
		free (self);
		return NULL;
	}

	lv2_log_logger_init (&self->logger, NULL, self->log);

	self->uris.midi_MidiEvent = self->map->map(self->map->handle, LV2_MIDI__MidiEvent);

	self->samplerate = rate;

	self->buffer = calloc (BUFFER_LENGTH, sizeof(float));

	if (!self->buffer) {
		lv2_log_error (&self->logger, "Unable to allocate buffer of %zu samples\n",
		               BUFFER_LENGTH);
		free (self);
		return NULL;
	}

	ok &= op_heap_init (&self->queued_ops[0], MAX_QUEUED_OPS);
	ok &= op_heap_init (&self->queued_ops[1], MAX_QUEUED_OPS);
	ok &= op_heap_init (&self->active_ops[0], MAX_ACTIVE_OPS);
	ok &= op_heap_init (&self->active_ops[1], MAX_ACTIVE_OPS);

	if (! ok) {
		lv2_log_error (&self->logger, "Unable to allocate operation heaps.\n");
		cleanup (self);
	}

	lv2_log_trace (&self->logger, "Successfully initialized Boucle plugin\n");

	return (LV2_Handle) self;
};

static void
connect_port (LV2_Handle instance,
              uint32_t port,
              void* data)
{
	Boucle* self = (Boucle*)instance;

	switch ((Port)port) {
		case PORT_BOUCLE_CONTROL:
			self->control = (const LV2_Atom_Sequence*)data;
			break;
		case PORT_INPUT:
			self->input = (const float*)data;
			break;
		case PORT_OUTPUT:
			self->output = (float*)data;
			break;
		case PORT_LOOP_LENGTH:
			self->loop_length = (const float*)data;
			break;
		case PORT_MIDI_BRIDGE:
			self->midi = (const LV2_Atom_Sequence*)data;
			break;
		case PORT_TEMPO:
			self->tempo = (const float*)data;
			break;
		default:
			lv2_log_warning (&self->logger, "Host tried to connect invalid port %i", port);
			break;
	}
}

static void
activate (LV2_Handle instance)
{
	Boucle* self = (Boucle*)instance;

	memset (self->buffer, 0, BUFFER_LENGTH * sizeof(float));
}

static void
queue_op (Boucle *self, Op op, uint32_t start)
{
	OpHeap *queue = &self->queued_ops[self->primary_queue_heap];
	op_heap_push (queue, op, start);
}

static void
pop_queued_op (Boucle *self)
{
	/* The secondary heap is older, so we should use that up first */
	OpHeap *primary_queue = &self->queued_ops[self->primary_queue_heap];
	OpHeap *secondary_queue = &self->queued_ops[1 - self->primary_queue_heap];
	if (secondary_queue->count > 0) {
		op_heap_pop (secondary_queue);
	} else {
		op_heap_pop (primary_queue);
	}
}

/* Process the MIDI inputs to produce a list of operations. */
#if 0
static void
get_ops_from_midi (Boucle *self,
                   uint32_t max_op_length /* frames */)
{
	LV2_ATOM_SEQUENCE_FOREACH(self->control, ev) {
		if (ev->body.type == self->uris.midi_MidiEvent) {
			const uint8_t* const msg = (const uint8_t*)(ev+1);
			switch (lv2_midi_message_type (msg)) {
				case LV2_MIDI_MSG_NOTE_ON:
					if (self->n_active_notes == 0) {
						/* Simple simple. Reverse whenever a key is pressed until all keys
						 * are released again.
						 */
						int op_index = record_op (self, OP_TYPE_REVERSE,
						         self->play_head + ev->time.frames);
						if (op_index == NO_OP) {
							lv2_log_warning (&self->logger, "No more space for operations\n");
						} else {
							self->ops[op_index].duration = max_op_length;

							lv2_log_note (&self->logger, "Got initial note on, "
							        "time: %llu. Recorded op@%u dur %u\n", ev->time.frames,
							        self->ops[op_index].start, self->ops[op_index].duration);
						}
					}
					++self->n_active_notes;
					break;

				case LV2_MIDI_MSG_NOTE_OFF:
					--self->n_active_notes;
					if (self->n_active_notes == 0) {
						int op_index = get_last_recorded_op (self);
						if (op_index == NO_OP) {
							lv2_log_warning (&self->logger, "No active op to stop\n");
						} else {
							uint32_t current_time = self->play_head + ev->time.frames;
							self->ops[op_index].duration = (current_time - self->ops[op_index].start);

							lv2_log_note (&self->logger, "Got last note off, time: "
							        "%llu. Op now %u->%u\n", ev->time.frames,
							        self->ops[op_index].start, self->ops[op_index].duration);
						}
					}
					break;
				default:; /* silence -Wswitch warning */
			}
		}
	}
}
#endif

static void
run (LV2_Handle instance,
     uint32_t n_samples)
{
	Boucle* self = (Boucle*)instance;

	const float* const input  = self->input;
	float* const output = self->output;

	const uint32_t loop_length_samples = CLAMP (
	    *(self->loop_length), 512, BUFFER_LENGTH);

	if (self->buffer_full) {
/*		get_ops_from_midi (self, loop_length_samples);*/

		/* FIXME: now you need to ....
		 *  process the control stream and queue the events
		 *  call the MIDI bridge and get events from there too
		 */

#if 0
		for (uint32_t pos = 0; pos < n_samples; pos++) {
			/* To do playback, we need to ...
			 *
			 *   set up an identity transform for the playhead
			 *   check the queued_ops heaps for any new ops
			 *      if any are found
			 *        update the transform function
			 *        push it to the active_ops queue
			 *        pop it from the queued_ops queue
			 *   check the active_ops heaps for any ops finishing
			 *      if any are found
			 *        update the transform function
			 *        pop it from the active_ops queue
			 *
			 *   if the playhead wraps
			 *      swap both heaps
			 *
			 * Since checking each heap is just a pointer lookup, we
			 * can probably get away with doing it every time &
			 * being sample-accurate. Doing it every 22us is overkill
			 * though, probably at 48KHz doing it every 20 samples
			 * would still be super accurate (accurate to 0.44ms...)
			 */

			/* Play back n_samples of the delay buffer */
			if (self->reverser_offset > 0) {
				/* Backwards playback */
				int reversed_play_head = (self->play_head + self->reverser_offset) % loop_length_samples;
				output[pos] = self->buffer[reversed_play_head];
				self->reverser_offset -= 2;
			} else {
				/* Normal playback */
				output[pos] = self->buffer[self->play_head];
			}

/*			lv2_log_trace (&self->logger, "next op: %i, type %i, start %u  ; head @ %u\n", self->op_play_head,
					self->ops[self->op_play_head].type, self->ops[self->op_play_head].start,
					self->play_head);*/
			if (self->ops[self->op_play_head].type != OP_TYPE_NONE &&
				self->ops[self->op_play_head].start == self->play_head) {
				lv2_log_trace (&self->logger, "Actioning op %i\n", self->op_play_head);
				Op op = self->ops[self->op_play_head];
				if (op.type == OP_TYPE_REVERSE) {
					lv2_log_trace (&self->logger, "Got reverse op at %u, duration %u\n",
					        op.start, op.duration);
					if (self->reverser_offset > 0) {
						lv2_log_warning (&self->logger, "Already in a reverse, at "
						         "reverser offset %u\n", self->reverser_offset);
					} else {
						self->reverser_offset = op.duration;
					}
				}
				actioned_op (self);
			}

			self->play_head ++;
			if (self->play_head >= loop_length_samples) {
				self->play_head = 0;
			}
		}
#endif
		/* TEMPORARY: just play back the buffer */
		for (uint32_t pos = 0; pos < n_samples; pos++) {
			output[pos] = self->buffer[self->play_head];

			self->play_head ++;
			if (self->play_head >= loop_length_samples) {
				self->play_head = 0;
			}
		}
	} else {
		/* During the initial record, do passthrough. */
		for (uint32_t pos = 0; pos < n_samples; pos++) {
			output[pos] = input[pos];

			self->play_head ++;
			if (self->play_head >= loop_length_samples) {
				self->play_head = 0;
			}
		}
	}

	/* Store the input into the buffer */
	for (uint32_t pos = 0; pos < n_samples; pos++) {
		self->buffer[self->record_head] = input[pos];

		self->record_head ++;
		if (self->record_head >= loop_length_samples) {
			if (! self->buffer_full) {
				lv2_log_trace (&self->logger, "Buffer is now full\n");
			}
			self->buffer_full = true;
			self->record_head = 0;
		}
	}
}

static void
deactivate (LV2_Handle instance) {
}

static void
cleanup (LV2_Handle instance)
{
	Boucle* self = (Boucle*)instance;

	op_heap_free (&self->queued_ops[0]);
	op_heap_free (&self->queued_ops[1]);
	op_heap_free (&self->active_ops[0]);
	op_heap_free (&self->active_ops[1]);
	free (self->buffer);

	free (instance);
}

static const void*
extension_data (const char* uri)
{
	return NULL;
}

static const LV2_Descriptor descriptor = {
	BOUCLE_URI,
	instantiate, connect_port, activate, run, deactivate, cleanup,
	extension_data
};

LV2_SYMBOL_EXPORT
const LV2_Descriptor*
lv2_descriptor (uint32_t index)
{
	switch (index) {
		case 0: return &descriptor;
		default: return NULL;
	}
}
