use crate::defs::DigestModel;
use crate::defs::DigestModelMemory;
use crate::defs::DigestModelSpec;
use crate::defs::DigestOutput;
use crate::defs::DigestPreferences;
use crate::defs::InputItem;

pub struct EmptyDigestModel;

impl DigestModel for EmptyDigestModel {
    async fn digest(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem]) -> DigestOutput {
        // Nothing matters, the ideal digest is empty.
        DigestOutput {
            selected_items: vec![],
            text: "".to_owned(),
        }
    }
    async fn reflect(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem], self_output: &DigestOutput, opponent_output: &DigestOutput, win: bool) -> DigestModelMemory {
        DigestModelMemory {
            text: memory.text.clone(),
        }
    }
}
