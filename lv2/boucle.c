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

#define BOUCLE_URI "http://afuera.me.uk/boucle"

/* Maximum memory usage: 11MB, based on 96KHz sample rate */
#define BUFFER_LENGTH  60 * 48000 /* Frames */

#define MIN(a,b) ({ __typeof__ (a) _a = (a); \
               __typeof__ (b) _b = (b); \
               _a < _b ? _a : _b; })
#define MAX(a,b) ({ __typeof__ (a) _a = (a); \
               __typeof__ (b) _b = (b); \
               _a > _b ? _a : _b; })
#define CLAMP(x,lo,hi) MIN(hi, MAX(lo, x))

typedef enum {
	PORT_BOUCLE_CONTROL = 0,
	PORT_INPUT  = 1,
	PORT_OUTPUT = 2,
	PORT_LOOP_LENGTH = 3,
} Port;

typedef enum {
	OP_TYPE_NONE = 0,
	OP_TYPE_REVERSE = 1,
	OP_TYPE_ABSOLUTE_JUMP = 2,
	OP_TYPE_RELATIVE_JUMP = 3,
	OP_TYPE_LOOP_IN_LOOP = 4,
	OP_TYPE_SPEED_RAMP = 5
} OpType;

typedef struct {
	OpType type;
	uint32_t start;  /* in samples */
	uint32_t duration;  /* in samples */
	/* More custom stuff may follow */
} Op;

typedef struct {
	Op op;
} ReverseOp;

typedef struct {
	Op op;
	uint32_t absolute_position;  /* in samples */
} AbsoluteJumpOp;

typedef struct {
	Op op;
	uint32_t relative_position;  /* in samples */
} RelativeJumpOp;

typedef struct {
	Op op;
	uint32_t loop_size;  /* in samples */
} LoopInLoopOp;

typedef struct {
	Op op;
	float start_speed;  /* coefficient */
	float end_speed;  /* coefficient */
} SpeedRampOp;

#define MAX_OPS  256

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
	const LV2_Atom_Sequence* midi_control;
	const float* input;
	float* output;
	const float* loop_length; /* in beats */

	/* Internals */
	double samplerate;

	float *buffer;
	bool buffer_full;

	size_t record_head; /* in samples */
	size_t play_head;

	int reverser_offset; /* For reverse operatios */

	int n_active_notes;

	/* This is a ring buffer; a linked list would be nice but we don't
	 * want to be allocating memory while processing audio. This must
	 * be ordered, and we should do some checking because no doubt I'll
	 * introduce lots of bugs where it isn't over the coming months.
	 */
	Op ops[MAX_OPS];
	int n_ops, op_record_head, op_play_head;
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
		fprintf(stderr, "Boucle: buggy host -- required features not provided. "
		                "Log feature: %p; map feature: %p", self->map, self->log);
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
			self->midi_control = (const LV2_Atom_Sequence*)data;
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

#define NO_OP -1

/* Create a new operation, if there is space in the op buffer. */
static int
record_op (Boucle *self,
           OpType type,
           uint32_t start /* in samples */)
{
	if (self->n_ops >= MAX_OPS)
		return NO_OP;

	self->n_ops ++;

	int op_index = self->op_record_head;

	if (++ self->op_record_head >= MAX_OPS)
		self->op_record_head = 0;

	self->ops[op_index].type = type;
	self->ops[op_index].start = start;

	return op_index;
}

static int
get_last_recorded_op (Boucle *self)
{
	int active_op_index = self->op_record_head - 1;
	if (active_op_index < 0)
		active_op_index = MAX_OPS - 1;
	return active_op_index;
}

static void
actioned_op (Boucle *self)
{
	self->ops[self->op_play_head].type = OP_TYPE_NONE;
	self->ops[self->op_play_head].start = 0;
	self->ops[self->op_play_head].duration = 0;

	if (++ self->op_play_head >= MAX_OPS)
		self->op_play_head = 0;

	self->n_ops --;

	lv2_log_trace (&self->logger, "Actioned op; n_ops now %i, op play ptr %i\n",
	        self->n_ops, self->op_play_head);
}

/* Process the MIDI inputs to produce a list of operations. */
static void
get_ops_from_midi (Boucle *self,
                   uint32_t max_op_length /* frames */)
{
	LV2_ATOM_SEQUENCE_FOREACH(self->midi_control, ev) {
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
		get_ops_from_midi (self, loop_length_samples);

		/* Play back n_samples of the delay buffer */
		for (uint32_t pos = 0; pos < n_samples; pos++) {
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
	Boucle* self = (Boucle*)instance;

	free (self->buffer);
	self->buffer = NULL;
}

static void
cleanup (LV2_Handle instance)
{
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
