use crate::baseline::BaselineDigestModel;
use crate::defs::DigestModel;
use crate::defs::DigestModelSpec;
use crate::defs::DigestModelMemory;
use crate::defs::DigestOutput;
use crate::defs::DigestPreferences;
use crate::defs::DigestSelectedItem;
use crate::defs::InputItem;
use crate::defs::InputItemReference;
use crate::defs::LiveSourceSpec;

pub mod baseline;
pub mod defs;
pub mod empty;
pub mod state;

#[tokio::main]
async fn main() {
    state::migrate().await;

    let live_source_spec = LiveSourceSpec {
        uri: "tag:summarena.pages.dev,2025-08:live_source/dummy".to_owned(),
    };
    state::create_live_source_spec(&live_source_spec).await;

    let item0_uri = "tag:summarena.pages.dev,2025-08:input_item/dummy/sample_text";
    let item0_text = "Hello, world!";
    let input_items = vec![
        InputItem {
            uri: item0_uri.to_owned(),
            live_source_uri: live_source_spec.uri,
            text: item0_text.to_owned(),
            vision: None,
        },
    ];
    for input_item in &input_items {
        state::ingest(input_item).await;
    }

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
    let output = BaselineDigestModel::digest(&spec, &memory_in, &preferences, &input_items).await;
    println!("output: {:#?}", &output);
    let other_output = DigestOutput {
        selected_items: vec![
            DigestSelectedItem {
                input_item_uri: item0_uri.to_owned(),
                references: vec![
                    InputItemReference {
                        text: "It said hello world".to_owned(),
                    },
                ],
            },
        ],
        text: "They said the usual hello world".to_owned(),
    };
    let memory_out = BaselineDigestModel::reflect(&spec, &memory_in, &preferences, &input_items, &output, &other_output, true).await;
    println!("memory_out: {:#?}", &memory_out);
}
