/**
 * Holo-REA proposed intents: maintains relationships between coordinated proposals and the individual intents that describe their planned enaction. zome internal data structures
 *
 * Required by the zome itself, and for any DNA-local zomes interacting with its
 * storage API directly.
 *
 * @package Holo-REA
 */
use hdk::prelude::*;

use hdk_records::{
    generate_record_entry,
};

use vf_attributes_hdk::{ProposedIntentAddress, IntentAddress, ProposalAddress};

use hc_zome_rea_proposed_intent_rpc::CreateRequest;

//--------------- ZOME CONFIGURATION ATTRIBUTES ----------------

// :TODO: remove this, replace with reference to appropriate namespacing of zome config
#[derive(Clone, Serialize, Deserialize, SerializedBytes, PartialEq, Debug)]
pub struct DnaConfigSlice {
    pub proposed_intent: ProposedIntentZomeConfig,
}

#[derive(Clone, Serialize, Deserialize, SerializedBytes, PartialEq, Debug)]
pub struct ProposedIntentZomeConfig {
    pub proposal_index_zome: String,
    pub index_zome: String,
}

//---------------- RECORD INTERNALS & VALIDATION ----------------

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone)]
pub struct EntryData {
    pub reciprocal: bool,
    pub publishes: IntentAddress,
    pub published_in: ProposalAddress,
}

generate_record_entry!(EntryData, ProposedIntentAddress, EntryStorage);

//---------------- CREATE ----------------

/// Pick relevant fields out of I/O record into underlying DHT entry
impl From<CreateRequest> for EntryData {
    fn from(e: CreateRequest) -> EntryData {
        EntryData {
            reciprocal: e.reciprocal,
            publishes: e.publishes.into(),
            published_in: e.published_in.into(),
        }
    }
}
