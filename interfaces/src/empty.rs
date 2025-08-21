use crate::defs::DigestModel;
use crate::defs::DigestModelMemory;
use crate::defs::DigestModelSpec;
use crate::defs::DigestOutput;
use crate::defs::DigestPreferences;
use crate::defs::InputItem;

pub struct EmptyDigestModel;

impl DigestModel for EmptyDigestModel {
    async fn digest(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem]) -> DigestOutput {
        _ = spec;
        _ = memory;
        _ = preferences;
        _ = input_items;
        // Nothing matters, the ideal digest is empty.
        DigestOutput {
            selected_items: vec![],
            text: "".to_owned(),
        }
    }
    async fn reflect(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem], self_output: &DigestOutput, opponent_output: &DigestOutput, win: bool) -> DigestModelMemory {
        _ = spec;
        _ = memory;
        _ = preferences;
        _ = input_items;
        _ = self_output;
        _ = opponent_output;
        _ = win;
        // Learn nothing, leave the memory as is.
        DigestModelMemory {
            text: memory.text.clone(),
        }
    }
}
