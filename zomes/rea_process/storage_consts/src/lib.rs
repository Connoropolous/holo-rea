/**
 * Storage constants for zome entry & link type identifiers
 *
 * Used by modules interfacing with the underlying Holochain storage system directly.
 *
 * @package Holo-REA
 */
pub const PROCESS_ENTRY_TYPE: &str = "vf_process";
pub const PROCESS_EVENT_INPUTS_LINK_TAG: &str = "inputs";
pub const PROCESS_EVENT_OUTPUTS_LINK_TAG: &str = "outputs";
pub const PROCESS_COMMITMENT_INPUTS_LINK_TAG: &str = "committed_inputs";
pub const PROCESS_COMMITMENT_OUTPUTS_LINK_TAG: &str = "committed_outputs";
pub const PROCESS_INTENT_INPUTS_LINK_TAG: &str = "intended_inputs";
pub const PROCESS_INTENT_OUTPUTS_LINK_TAG: &str = "intended_outputs";

pub const PROCESS_EVENT_INPUTS_READ_API_METHOD: &str = "_internal_read_process_inputs";
pub const PROCESS_EVENT_OUTPUTS_READ_API_METHOD: &str = "_internal_read_process_outputs";
pub const PROCESS_COMMITMENT_INPUTS_READ_API_METHOD: &str = "_internal_read_process_committed_inputs";
pub const PROCESS_COMMITMENT_OUTPUTS_READ_API_METHOD: &str = "_internal_read_process_committed_outputs";
pub const PROCESS_INTENT_INPUTS_READ_API_METHOD: &str = "_internal_read_process_intended_inputs";
pub const PROCESS_INTENT_OUTPUTS_READ_API_METHOD: &str = "_internal_read_process_intended_outputs";
