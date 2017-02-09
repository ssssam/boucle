#include <boucle.h>

#include <gst/gst.h>
#include <gst/audio/gstaudiofilter.h>

struct _BoucleEffect {
	GstBaseTransform parent;
};

G_DEFINE_TYPE (BoucleEffect, boucle_effect, GST_TYPE_BASE_TRANSFORM)

#define ALLOWED_CAPS \
    "audio/x-raw"

static GstFlowReturn
boucle_effect_transform_ip (GstBaseTransform * base, GstBuffer *buf)
{
	GstMapInfo map;

	gst_buffer_map (buf, &map, GST_MAP_READWRITE);

	/* You can fuck wit the buffer now, like reverse it n stuff */

	gst_buffer_unmap (buf, &map);

	return GST_FLOW_OK;
}

static void
boucle_effect_class_init (BoucleEffectClass * klass)
{
	GstCaps *caps;

	gst_element_class_set_static_metadata (GST_ELEMENT_CLASS (klass), "Boucle",
	        "Filter/Effect/Audio",
	        "Boucle rhythmik audio destruktor", "Sam Thursfield <sam@afuera.me.uk>");

	caps = gst_caps_from_string (ALLOWED_CAPS);
	gst_audio_filter_class_add_pad_templates (GST_AUDIO_FILTER_CLASS (klass),
	        caps);
	gst_caps_unref (caps);

	GST_BASE_TRANSFORM_CLASS (klass)->transform_ip =
	        GST_DEBUG_FUNCPTR (boucle_effect_transform_ip);
	GST_BASE_TRANSFORM_CLASS (klass)->transform_ip_on_passthrough = FALSE;
}

static void
boucle_effect_init (BoucleEffect * self)
{
	gst_base_transform_set_in_place (GST_BASE_TRANSFORM (self), TRUE);
}
