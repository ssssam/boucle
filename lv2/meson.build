lv2_boucle_install_path = join_paths(lv2_install_path, 'boucle.lv2')

shared_library('lv2boucle',
	'boucle.c',
	dependencies: lv2,
	install: true,
	install_dir: lv2_boucle_install_path)

install_data('manifest.ttl', 'boucle.ttl',
	install_dir: lv2_boucle_install_path)
