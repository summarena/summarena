use crate::baseline::BaselineDigestModel;
use crate::defs::DigestModel;
use crate::defs::DigestModelSpec;
use crate::defs::DigestModelMemory;
use crate::defs::DigestOutput;
use crate::defs::DigestPreferences;
use crate::defs::DigestSelectedItem;
use crate::defs::InputItem;
use crate::defs::InputItemReference;

pub mod baseline;
pub mod defs;
pub mod empty;
pub mod state;

fn main() {
    let spec = DigestModelSpec {
        uri: "tag:summarena.pages.dev,2025-08:digest_model/baseline".to_owned(),
    };
    let memory_in = DigestModelMemory {
        text: "".to_owned(),
    };
    let preferences = DigestPreferences {
        uri: "tag:summarena.pages.dev,2025-08:digest_preferences/empty".to_owned(),
        description: "".to_owned(),
    };
    let item0_uri = "tag:summarena.pages.dev,2025-08:input_item/dummy/sample_text";
    let item0_text = "Hello, world!";
    let input_items = vec![
        InputItem {
            uri: item0_uri.to_owned(),
            text: item0_text.to_owned(),
            vision: None,
        },
    ];
    let output = BaselineDigestModel::digest(&spec, &memory_in, &preferences, &input_items);
    println!("output: {:#?}", &output);
    let other_output = DigestOutput {
        selected_items: vec![
            DigestSelectedItem {
                input_item_uri: item0_uri.to_owned(),
                references: vec![
                    InputItemReference {
                        text_start_index: 0,
                        text_end_index: item0_text.len(),
                    },
                ],
            },
        ],
        text: "They said the usual hello world".to_owned(),
    };
    let memory_out = BaselineDigestModel::reflect(&spec, &memory_in, &preferences, &input_items, &output, &other_output, true);
    println!("memory_out: {:#?}", &memory_out);
}
