#include <lv2/lv2plug.in/ns/lv2core/lv2.h>
#include <lv2/lv2plug.in/ns/ext/log/log.h>
#include <lv2/lv2plug.in/ns/ext/log/logger.h>

#include <math.h>
#include <stdbool.h>
#include <stdlib.h>

#define BOUCLE_URI "http://afuera.me.uk/boucle"

/* Maximum memory usage: 23MB, based on 96KHz sample rate */
#define BUFFER_LIMIT  60 /* Seconds */

#define MIN(a,b) ({ __typeof__ (a) _a = (a); \
               __typeof__ (b) _b = (b); \
               _a < _b ? _a : _b; })

typedef enum {
	PORT_INPUT  = 0,
	PORT_OUTPUT = 1,
	PORT_TEMPO = 2,
	PORT_LOOP_LENGTH = 3,
} Port;

typedef struct {
	/* LV2 extensions */
	LV2_Log_Log* log;
	LV2_Log_Logger logger;

	/* LV2 connections */
	const float* input;
	float* output;
	const float* tempo; /* bpm */
	const float* loop_length; /* in beats */

	/* Internals */
	double samplerate;

	float *buffer;
	size_t buffer_length; /* in samples */
	bool buffer_full;

	size_t record_head; /* in samples */
	size_t play_head;
} Boucle;

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

	// Get host features
	for (int i = 0; features[i]; ++i) {
		if (!strcmp(features[i]->URI, LV2_LOG__log)) {
			self->log = (LV2_Log_Log*)features[i]->data;
		}
	}

	lv2_log_logger_init (&self->logger, NULL, self->log);

	self->samplerate = rate;

	self->buffer_length = ceil (rate * BUFFER_LIMIT);
	self->buffer = calloc (self->buffer_length, sizeof(float));

	if (!self->buffer) {
		lv2_log_error (&self->logger, "Unable to allocate buffer of %zu samples\n",
		               self->buffer_length);
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
		case PORT_INPUT:
			self->input = (const float*)data;
			break;
		case PORT_OUTPUT:
			self->output = (float*)data;
			break;
		case PORT_TEMPO:
			self->tempo = (const float*)data;
			break;
		case PORT_LOOP_LENGTH:
			self->loop_length = (const float*)data;
			break;
	}
}

static void
activate (LV2_Handle instance)
{
	Boucle* self = (Boucle*)instance;

	memset (self->buffer, 0, self->buffer_length * sizeof(float));

	self->buffer_full = false;

	self->record_head = 0;
	self->play_head = 0;
}

static void
run (LV2_Handle instance,
     uint32_t n_samples)
{
	Boucle* self = (Boucle*)instance;

	const float* const input  = self->input;
	float* const output = self->output;

	const float tempo = *(self->tempo);
	const float loop_length_beats = *(self->loop_length);

	const float seconds_per_beat = 60.0f / tempo;
	const size_t loop_length_samples = MIN (
	    ceil (seconds_per_beat * loop_length_beats * self->samplerate),
	    self->buffer_length);

	lv2_log_trace (&self->logger, "Boucle loop started; loop length is %zu samples\n",
	    loop_length_samples);

	lv2_log_trace (&self->logger, "seconds per beat: %f, length in beats: %f, samplerate %lf, buffer length %zu\n",
	    seconds_per_beat, loop_length_beats, self->samplerate, self->buffer_length);

	/* Store the input into the buffer */
	for (uint32_t pos = 0; pos < n_samples; pos++) {
		self->buffer[self->record_head] = input[pos];

		self->record_head ++;
		if (self->record_head >= loop_length_samples) {
			self->buffer_full = true;
			self->record_head = 0;
		}
	}

	if (self->buffer_full) {
		/* Play back n_samples of the delay buffer */
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

	lv2_log_trace (&self->logger, "Processed %i samples; buffer %s; record %zu, play %zu\n", n_samples,
	               self->buffer_full ? "full" : "not full", self->record_head, self->play_head);
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
