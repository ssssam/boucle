#include <gst/gst.h>

#include <src/boucle.h>

static gboolean
plugin_init (GstPlugin * plugin)
{
  return (gst_element_register (plugin, "boucle", GST_RANK_NONE,
          BOUCLE_TYPE_EFFECT))
}

GST_PLUGIN_DEFINE (GST_VERSION_MAJOR,
    GST_VERSION_MINOR,
    boucle,
    "Boucle effect plugin",
    plugin_init, "0.0.1", GST_LICENSE, "boucle", "boucle")
