use crate::defs::DigestModel;
use crate::defs::DigestModelMemory;
use crate::defs::DigestModelSpec;
use crate::defs::DigestOutput;
use crate::defs::DigestPreferences;
use crate::defs::DigestSelectedItem;
use crate::defs::InputItemReference;
use crate::defs::InputItem;

struct PonderedPreferences {
    pub look_out_for: String,
}

async fn ponder_preferences(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences) -> PonderedPreferences {
    PonderedPreferences {
        look_out_for: preferences.description.clone(),
    }
}

#[derive(Clone)]
struct FocusedSummary {
    pub summary_text: String,
    pub references: Vec<InputItemReference>,
}

async fn ponder_relevance_and_summarize(spec: &DigestModelSpec, pondered_preferences: &PonderedPreferences, input_item: &InputItem) -> FocusedSummary {
    FocusedSummary {
        summary_text: input_item.text.clone(),
        references: vec![InputItemReference { text_start_index: 0, text_end_index: input_item.text.len() }],
    }
}

async fn select_best(spec: &DigestModelSpec, pondered_preferences: &PonderedPreferences, focused_summaries: &[FocusedSummary]) -> Vec<usize> {
    (0..focused_summaries.len()).collect()
}

async fn compose_digest(spec: &DigestModelSpec, pondered_preferences: &PonderedPreferences, best_summaries: &[FocusedSummary]) -> String {
    best_summaries.iter().map(|summary| summary.summary_text.clone()).collect::<Vec<String>>().join("\n")
}

async fn reflect(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem], self_output: &DigestOutput, opponent_output: &DigestOutput, win: bool) -> DigestModelMemory {
    DigestModelMemory {
        text: memory.text.clone(),
    }
}

pub struct BaselineDigestModel;

impl DigestModel for BaselineDigestModel {

    async fn digest(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem]) -> DigestOutput {
        let pondered_preferences = ponder_preferences(spec, memory, preferences).await;
        let focused_summaries = futures::future::join_all(input_items.iter().map(|input_item| ponder_relevance_and_summarize(spec, &pondered_preferences, input_item))).await;
        let best_summary_indices = select_best(spec, &pondered_preferences, &focused_summaries).await;
        let best_summaries = best_summary_indices.iter().map(|index| focused_summaries[*index].clone()).collect::<Vec<FocusedSummary>>();
        let digest_text = compose_digest(spec, &pondered_preferences, &best_summaries).await;
        DigestOutput {
            selected_items: best_summary_indices.iter().map(|index| DigestSelectedItem { input_item_uri: input_items[*index].uri.clone(), references: focused_summaries[*index].references.clone() }).collect::<Vec<DigestSelectedItem>>(),
            text: digest_text,
        }
    }

    async fn reflect(spec: &DigestModelSpec, memory: &DigestModelMemory, preferences: &DigestPreferences, input_items: &[InputItem], self_output: &DigestOutput, opponent_output: &DigestOutput, win: bool) -> DigestModelMemory {
        reflect(spec, memory, preferences, input_items, self_output, opponent_output, win).await
    }

}
