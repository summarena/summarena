pub struct LiveSourceSpec {
    pub uri: String,
}

pub struct InputItem {
    pub uri: String,
    pub live_source_uri: String,
    pub text: String,
    pub vision: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct InputItemReference {
    // tbd
    pub text_start_index: usize,
    pub text_end_index: usize,
}

pub struct WatchRest {
    pub wait_at_least_ms: u32,
}

pub trait Ingester {
    fn watch(source: &LiveSourceSpec) -> WatchRest;
}

pub struct DigestPreferences {
    pub uri: String,
    pub description: String,
}

pub struct DigestDataset {
    pub uri: String,
    pub input_item_uris: Vec<String>,
}

pub struct DigestModelSpec {
    pub uri: String,
}

#[derive(Debug)]
pub struct DigestModelMemory {
    pub text: String,
}

#[derive(Debug)]
pub struct DigestSelectedItem {
    pub input_item_uri: String,
    pub references: Vec<InputItemReference>,
}

#[derive(Debug)]
pub struct DigestOutput {
    pub selected_items: Vec<DigestSelectedItem>,
    pub text: String,
}

// Object style note:
// Envision the implementations of these traits (Ingester, DigestModel, etc.)
// as written to run inside short lived single-task processes.
// Thus, they won't substantially manage internal state inside a struct.
// Typically you'll declare an empty type, e.g. `struct SampleDigestModel;`
// and load information in a database based on the spec parameter within each
// method.

pub trait DigestModel {
    fn digest(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem]) -> DigestOutput;
    fn reflect(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem], self_output: &DigestOutput, opponent_output: &DigestOutput, win: bool) -> DigestModelMemory;
}

pub struct DigestAttempt {
    pub uri: String,
    pub dataset_uri: String,
    pub model_uri: String,
    pub output: DigestOutput,
}
