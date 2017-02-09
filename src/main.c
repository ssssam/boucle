/* Boucle */
/* GPL3 license */

#include <glib.h>

static const char *filename = NULL;

static GOptionEntry options[] =
{
	{ "input", 0, 0, G_OPTION_ARG_FILENAME, &filename, "Input audio file", NULL },
	{ NULL }
};

int main(int argc, char *argv[]) {
	GError *error = NULL;
	GOptionContext *context;

	context = g_option_context_new ("- audio effect");
	g_option_context_add_main_entries (context, options, NULL);

	if (! g_option_context_parse (context, &argc, &argv, &error)) {
		g_printerr ("Option parsing failed: %s\n", error->message);
		return 1;
	}

	return 0;
}
