#[macro_use]
extern crate vst;

use vst::plugin::{Info, Plugin, Category};

#[derive(Default)]
struct BoucleVst;

impl Plugin for BoucleVst {
    fn get_info(&self) -> Info {
        Info {
            name: "Boucle".to_string(),
            vendor: "Medium Length Life".to_string(),
            unique_id: 42,
            inputs: 2,
            outputs: 2,
            version: 1,
            category: Category::Effect,
            ..Default::default()
        }
    }
}

plugin_main!(BoucleVst);
