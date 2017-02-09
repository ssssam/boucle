#ifndef __BOUCLE_H__
#define __BOUCLE_H__

#include <gst/gst.h>
#include <gst/base/gstbasetransform.h>

#define BOUCLE_TYPE_EFFECT boucle_effect_get_type ()
G_DECLARE_FINAL_TYPE (BoucleEffect, boucle_effect, BOUCLE, EFFECT, GstBaseTransform)

#endif
