#include <lv2/lv2plug.in/ns/lv2core/lv2.h>

#include <stdlib.h>

#define BOUCLE_URI "http://afuera.me.uk/boucle"

typedef enum {
	PORT_INPUT  = 0,
	PORT_OUTPUT = 1,
	PORT_TEMPO = 2,
} Port;

typedef struct {
	const float* input;
	float* output;
	const float* tempo;
} Boucle;

static LV2_Handle
instantiate (const LV2_Descriptor* descriptor,
             double rate,
             const char* bundle_path,
             const LV2_Feature* const *features)
{
	Boucle* self = malloc (sizeof (Boucle));

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
	}
}

static void
activate (LV2_Handle instance)
{
}

static void
run (LV2_Handle instance,
     uint32_t n_samples)
{
	const Boucle* self = (const Boucle*)instance;

	const float* const input  = self->input;
	float* const output = self->output;

	/* FIXME: just passthru at this point */
	for (uint32_t pos = 0; pos < n_samples; pos++) {
		output[pos] = input[pos];
	}
}

static void
deactivate (LV2_Handle instance)
{
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
