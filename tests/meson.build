test_op_heap = executable('test_op_heap',
	'test_op_heap.c',
	'../lv2/op-heap.c',
	dependencies: [glib],
	include_directories: topinc)

test('test_op_heap', test_op_heap)

test_lv2_core = executable('test_lv2_core',
	'test_lv2_core.c',
	dependencies: [glib, lilv])

test('lv2_core', test_lv2_core,
	env: ['LV2_PATH=' + lv2_install_path])
