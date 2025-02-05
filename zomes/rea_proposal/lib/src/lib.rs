/**
* Holo-REA proposal zome library API
*
* Contains helper methods that can be used to manipulate `Proposal` data
* structures in either the local Holochain zome, or a separate DNA-local zome.
*
* @package Holo-REA
*/
use paste::paste;
use hdk_records::{
    RecordAPIResult,
    records::{
        create_record,
        delete_record,
        read_record_entry,
        update_record,
    },
};
use hdk_semantic_indexes_client_lib::*;

use hc_zome_rea_proposal_rpc::*;
use hc_zome_rea_proposal_storage::*;

pub fn handle_create_proposal<S>(entry_def_id: S, proposal: CreateRequest) -> RecordAPIResult<ResponseData>
    where S: AsRef<str>,
{
    let (revision_id, base_address, entry_resp): (_,_, EntryData) = create_record(&entry_def_id, proposal)?;
    Ok(construct_response(&base_address, &revision_id, &entry_resp, get_link_fields(&base_address)?))
}

pub fn handle_get_proposal<S>(entry_def_id: S, address: ProposalAddress) -> RecordAPIResult<ResponseData>
    where S: AsRef<str>,
{
    let (revision, base_address, entry) = read_record_entry::<EntryData, EntryStorage, _,_>(&entry_def_id, address.as_ref())?;
    Ok(construct_response(&base_address, &revision, &entry, get_link_fields(&base_address)?))
}

pub fn handle_update_proposal<S>(entry_def_id: S, proposal: UpdateRequest) -> RecordAPIResult<ResponseData>
    where S: AsRef<str>,
{
    let old_revision = proposal.get_revision_id().to_owned();
    let (revision_id, base_address, new_entry, _prev_entry): (_, ProposalAddress, EntryData, EntryData) = update_record(entry_def_id, &old_revision, proposal)?;
    Ok(construct_response(&base_address, &revision_id, &new_entry, get_link_fields(&base_address)?))
}

pub fn handle_delete_proposal(address: RevisionHash) -> RecordAPIResult<bool> {
    delete_record::<EntryStorage,_>(&address)
}

/// Create response from input DHT primitives
fn construct_response<'a>(
    address: &ProposalAddress,
    revision_id: &RevisionHash,
    e: &EntryData,
    (publishes, published_to): (
        Vec<ProposedIntentAddress>,
        Vec<ProposedToAddress>,
    ),
) -> ResponseData {
    ResponseData {
        proposal: Response {
            // entry fields
            id: address.to_owned(),
            revision_id: revision_id.to_owned(),
            name: e.name.to_owned(),
            has_beginning: e.has_beginning.to_owned(),
            has_end: e.has_end.to_owned(),
            unit_based: e.unit_based.to_owned(),
            created: e.created.to_owned(),
            note: e.note.to_owned(),
            in_scope_of: e.in_scope_of.to_owned(),
            // link fields
            publishes: publishes.to_owned(),
            published_to: published_to.to_owned(),
        },
    }
}

/// Properties accessor for zome config
fn read_proposal_index_zome(conf: DnaConfigSlice) -> Option<String> {
    Some(conf.proposal.index_zome)
}

fn get_link_fields<'a>(
    proposal: &ProposalAddress,
) -> RecordAPIResult<(
    Vec<ProposedIntentAddress>,
    Vec<ProposedToAddress>,
)> {
    Ok((
        read_index!(proposal(proposal).publishes)?,
        read_index!(proposal(proposal).published_to)?,
    ))
}
