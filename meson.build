project('boucle', 'c')

glib = dependency('glib-2.0')
gstreamer = dependency('gstreamer-1.0')
gstreamer_audio = dependency('gstreamer-audio-1.0')
gstreamer_base = dependency('gstreamer-base-1.0')

math = meson.get_compiler('c').find_library('m')

lilv = dependency('lilv-0')
lv2 = dependency('lv2')

lv2_install_path = join_paths(get_option('prefix'), get_option('libdir'), 'lv2')

topinc = include_directories('.')

subdir('src')
subdir('gst')
subdir('lv2')
subdir('tests')
