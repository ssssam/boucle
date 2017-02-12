test_core = executable('test_core',
	'test_core.c',
	dependencies: [glib, lilv])

test('core', test_core,
	env: ['LV2_PATH=' + lv2_install_path])
