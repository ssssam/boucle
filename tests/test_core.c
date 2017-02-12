/* This test is written in C, which makes me sad!
 *
 * However, the Python bindings included in version 0.20.0 of Lilv (the
 * version available in Fedora 25) have a pretty major bug where the first
 * item of a collection is missed out by the iterator.
 *
 * Upstream, the Python bindings are totally rewritten (and probably not
 * API compatible). The new bindings are probably good but they aren't
 * available in distros yet. So for the time being the tests are written
 * in C.
 *
 * Hopefully BuildStream will soon make these distro packaging problems
 * irrelevant and we can develop against upstream versions without
 * friction :-)
 *
 * Also, there seems to be no library that provides a test framework for
 * LV2 plugins right now. This is an obvious case for adding a liblilv-test
 * library to the Lilv project, it just requires someone to do the work...
 */

#include <lv2/lv2plug.in/ns/lv2core/lv2.h>
#include <lv2/lv2plug.in/ns/ext/log/log.h>
#include <lv2/lv2plug.in/ns/ext/urid/urid.h>
#include <lilv/lilv.h>

#include <glib.h>

#include <assert.h>
#include <stdio.h>
#include <string.h>

#define BOUCLE_URI "http://aferua.me.uk/boucle"

LV2_Feature lv2_log_feature = { LV2_LOG__log, NULL };
LV2_Feature lv2_map_feature = { LV2_URID__map, NULL };
LV2_Feature lv2_unmap_feature = { LV2_URID__unmap, NULL };

const LV2_Feature* lv2_features[4] = {
	&lv2_log_feature, &lv2_map_feature, &lv2_unmap_feature, NULL,
};

int test_lv2_log_vprintf(LV2_Log_Handle handle,
                         LV2_URID       type,
                         const char*    fmt,
                         va_list        ap)
{
	fprintf (stderr, "log: ");
	return vfprintf(stderr, fmt, ap);
}

int test_lv2_log_printf(LV2_Log_Handle handle,
                        LV2_URID       type,
                        const char*    fmt, ...)
{
	va_list args;
	va_start(args, fmt);
	const int ret = test_lv2_log_vprintf(handle, type, fmt, args);
	va_end(args);
	return ret;
}

static LV2_URID test_lv2_map_uri(LV2_URID_Map_Handle handle,
                                 const char*         uri)
{
	return g_quark_from_string (uri);
}

static const char* test_lv2_unmap_uri(LV2_URID_Unmap_Handle handle,
                                      LV2_URID             urid)
{
	return g_quark_to_string (urid);
}


const LilvPlugin* load_boucle_core_plugin(LilvWorld* lilv_world) {
	const LilvPlugins* plugin_list;
	const LilvPlugin* plugin = NULL;

	lilv_world_load_all (lilv_world);

	plugin_list = lilv_world_get_all_plugins (lilv_world);
	LILV_FOREACH(plugins, p, plugin_list) {
		plugin = lilv_plugins_get(plugin_list, p);

		const LilvNode *node = lilv_plugin_get_uri (plugin);
		if (strcmp (lilv_node_as_uri (node), BOUCLE_URI) == 0)
			break;
	}

	return plugin;
}

void test_basic(const LilvPlugin* plugin) {
	LilvInstance* instance;

	instance = lilv_plugin_instantiate (plugin, 48000, lv2_features);

	assert (instance != NULL);

	lilv_instance_free (instance);
}

int main() {
	LV2_Log_Log lv2_log_impl = { NULL, test_lv2_log_printf, test_lv2_log_vprintf };
	lv2_log_feature.data = &lv2_log_impl;

	LV2_URID_Map lv2_map_impl = { NULL, test_lv2_map_uri };
	lv2_map_feature.data = &lv2_map_impl;

	LV2_URID_Unmap lv2_unmap_impl = { NULL, test_lv2_unmap_uri };
	lv2_unmap_feature.data = &lv2_unmap_impl;

	LilvWorld* world = lilv_world_new ();

	const LilvPlugin* plugin = load_boucle_core_plugin (world);

	if (plugin == NULL) {
		fprintf (stderr, "Did not find plugin %s. Is LV2_PATH set correctly?",
		         BOUCLE_URI);
		return 1;
	}

	test_basic (plugin);

	return 0;
}
