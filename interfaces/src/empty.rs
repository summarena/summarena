use crate::defs::DigestModel;
use crate::defs::DigestModelMemory;
use crate::defs::DigestModelSpec;
use crate::defs::DigestOutput;
use crate::defs::DigestPreferences;
use crate::defs::InputItem;

pub struct EmptyDigestModel;

impl DigestModel for EmptyDigestModel {
    fn digest(_spec: &DigestModelSpec, _memory: &DigestModelMemory, _preferences: &DigestPreferences, _input_items: &[InputItem]) -> DigestOutput {
        // Nothing matters, the ideal digest is empty.
        DigestOutput {
            selected_items: vec![],
            text: "".to_owned(),
        }
    }
    fn reflect(_spec: &DigestModelSpec, memory: &DigestModelMemory, _preferences: &DigestPreferences, _input_items: &[InputItem], _self_output: &DigestOutput, _opponent_output: &DigestOutput, _win: bool) -> DigestModelMemory {
        DigestModelMemory {
            text: memory.text.clone(),
        }
    }
}
