/**
 * Holo-REA 'economic resource' zome library API
 *
 * Contains helper methods that can be used to manipulate economic resource data
 * structures in either the local Holochain zome, or a separate DNA-local zome.
 *
 * @package Holo-REA
 */
use paste::paste;
use hdk_records::{
    DataIntegrityError, RecordAPIResult, MaybeUndefined,
    local_indexes::{
        query_root_index,
    },
    records::{
        get_latest_header_hash,
        create_record,
        read_record_entry,
        update_record,
    },
    EntryHash,
};
use hdk_semantic_indexes_client_lib::*;
use hdk_relay_pagination::PageInfo;

use vf_attributes_hdk::{
    EconomicResourceAddress,
    EconomicEventAddress,
    ActionId,
    ProcessSpecificationAddress,
};

pub use hc_zome_rea_economic_resource_storage_consts::*;
pub use hc_zome_rea_economic_event_storage_consts::{EVENT_ENTRY_TYPE};
pub use hc_zome_rea_process_storage_consts::{PROCESS_ENTRY_TYPE};
pub use hc_zome_rea_resource_specification_storage_consts::{ECONOMIC_RESOURCE_SPECIFICATION_ENTRY_TYPE};

use hc_zome_rea_economic_resource_zome_api::*;
use hc_zome_rea_economic_resource_storage::*;
use hc_zome_rea_economic_resource_rpc::*;
use hc_zome_rea_process_storage::{
    EntryData as ProcessData,
    EntryStorage as ProcessStorage,
};
use hc_zome_rea_economic_event_storage::{EntryData as EventData, EntryStorage as EventStorage};
use hc_zome_rea_economic_event_rpc::{
    ResourceResponse as Response,
    ResourceResponseData as ResponseData,
    ResourceResponseCollection as Collection,
    ResourceResponseEdge as Edge,
    ResourceInventoryType,
    CreateRequest as EventCreateRequest,
};

// :SHONK: needed to re-export for zome `entry_defs()` where macro-assigned defs are overridden
pub use hdk_records::CAP_STORAGE_ENTRY_DEF_ID;

/// Trait object defining the default ValueFlows EconomicResource zome API.
/// 'Permissable' denotes the interface as a highly-permissable one, where little
/// validation on entry contents is performed.
pub struct EconomicResourceZomePermissableDefault;

impl API for EconomicResourceZomePermissableDefault {
    type S = &'static str;

    /// Handle creation of new resources via events + resource metadata.
    ///
    /// :WARNING: Should only ever be wired up as the dependency of an EconomicEvent zome.
    ///           API is not for direct use and could lead to an inconsistent database state.
    ///
    /// :TODO: assess whether this should use the same standardised API format as external endpoints
    ///
    fn create_inventory_from_event(resource_entry_def_id: Self::S, params: CreationPayload) -> RecordAPIResult<(RevisionHash, EconomicResourceAddress, EntryData)>
    {
        // :TODO: move this assertion to validation callback
        if let MaybeUndefined::Some(_sent_inventory_id) = &params.get_event_params().resource_inventoried_as {
            return Err(DataIntegrityError::RemoteRequestError("cannot create a new EconomicResource and specify an inventoried resource ID in the same event".to_string()));
        }

        let resource_params = params.get_resource_params().clone();
        let resource_spec = params.get_resource_specification_id();

        let (revision_id, base_address, entry_resp): (_, EconomicResourceAddress, EntryData) = create_record(
            &resource_entry_def_id,
            params.with_inventory_type(ResourceInventoryType::ProvidingInventory),  // inventories can only be inited by their owners initially
        )?;

        // :NOTE: this will always run- resource without a specification ID would fail entry validation (implicit in the above)
        if let Some(conforms_to) = resource_spec {
            let _ = create_index!(Remote(economic_resource.conforms_to(conforms_to), resource_specification.conforming_resources(&base_address)));
        }
        if let Some(contained_in) = resource_params.get_contained_in() {
            create_index!(Self(economic_resource(&base_address).contained_in(&contained_in)))?;
        };

        Ok((revision_id, base_address, entry_resp))
    }

    fn get_economic_resource(entry_def_id: Self::S, event_entry_def_id: Self::S, process_entry_def_id: Self::S, address: EconomicResourceAddress) -> RecordAPIResult<ResponseData>
    {
        let (revision, base_address, entry) = read_record_entry::<EntryData, EntryStorage, _,_>(&entry_def_id, address.as_ref())?;
        construct_response(&base_address, &revision, &entry, get_link_fields(&event_entry_def_id, &process_entry_def_id, &address)?)
    }

    /// Handle update of resources by iterative reduction of event records over time.
    ///
    fn update_inventory_from_event(
        resource_entry_def_id: Self::S,
        event: EventCreateRequest,
    ) -> RecordAPIResult<Vec<(RevisionHash, EconomicResourceAddress, EntryData, EntryData)>>
    {
        let mut resources_affected: Vec<(RevisionHash, EconomicResourceAddress, EntryData, EntryData)> = vec![];

        // if the event is a transfer-like event, run the receiver's update first
        if let MaybeUndefined::Some(receiver_inventory) = &event.to_resource_inventoried_as {
            let inv_entry_hash: &EntryHash = receiver_inventory.as_ref();
            resources_affected.push(handle_update_inventory_resource(
                &resource_entry_def_id,
                &get_latest_header_hash(inv_entry_hash.clone())?,   // :TODO: temporal reduction here! Should error on mismatch and return latest valid ID
                event.with_inventory_type(ResourceInventoryType::ReceivingInventory),
            )?);
        }
        // after receiver, run provider. This entry data will be returned in the response.
        if let MaybeUndefined::Some(provider_inventory) = &event.resource_inventoried_as {
            let inv_entry_hash: &EntryHash = provider_inventory.as_ref();
            resources_affected.push(handle_update_inventory_resource(
                &resource_entry_def_id,
                &get_latest_header_hash(inv_entry_hash.clone())?,   // :TODO: temporal reduction here! Should error on mismatch and return latest valid ID
                event.with_inventory_type(ResourceInventoryType::ProvidingInventory),
            )?);
        }

        Ok(resources_affected)
    }

    fn update_economic_resource(entry_def_id: Self::S, event_entry_def_id: Self::S, process_entry_def_id: Self::S, resource: UpdateRequest) -> RecordAPIResult<ResponseData>
    {
        let address = resource.get_revision_id().clone();
        let (revision_id, identity_address, entry, prev_entry): (_,_, EntryData, EntryData) = update_record(&entry_def_id, &address, resource)?;

        // :TODO: this may eventually be moved to an EconomicEvent update, see https://lab.allmende.io/valueflows/valueflows/-/issues/637
        let now_contained = if let Some(contained) = &entry.contained_in { vec![contained.clone()] } else { vec![] };
        let prev_contained = if let Some(contained) = &prev_entry.contained_in { vec![contained.clone()] } else { vec![] };
        update_index!(Self(economic_resource(&identity_address).contained_in(now_contained.as_slice()).not(prev_contained.as_slice())))?;

        // :TODO: optimise this- should pass results from `replace_direct_index` instead of retrieving from `get_link_fields` where updates
        construct_response(&identity_address, &revision_id, &entry, get_link_fields(&event_entry_def_id, &process_entry_def_id, &identity_address)?)
    }

    fn get_all_economic_resources(entry_def_id: Self::S, event_entry_def_id: Self::S, process_entry_def_id: Self::S) -> RecordAPIResult<Collection>
    {
        let entries_result = query_root_index::<EntryData, EntryStorage, _,_>(&entry_def_id)?;

        handle_list_output(event_entry_def_id, process_entry_def_id, entries_result)
    }
}

/// Properties accessor for zome config
fn read_economic_resource_index_zome(conf: DnaConfigSlice) -> Option<String> {
    Some(conf.economic_resource.index_zome)
}

fn handle_update_inventory_resource<S>(
    resource_entry_def_id: S,
    resource_addr: &RevisionHash,
    event: EventCreateRequest,
) -> RecordAPIResult<(RevisionHash, EconomicResourceAddress, EntryData, EntryData)>
    where S: AsRef<str>,
{
    Ok(update_record(&resource_entry_def_id, resource_addr, event)?)
}

fn handle_list_output<S>(event_entry_def_id: S, process_entry_def_id: S, entries_result: Vec<RecordAPIResult<(RevisionHash, EconomicResourceAddress, EntryData)>>) -> RecordAPIResult<Collection>
    where S: AsRef<str>
{
    let edges = entries_result.iter()
        .cloned()
        .filter_map(Result::ok)
        .map(|(revision_id, entry_base_address, entry)| {
            construct_list_response(
                &entry_base_address, &revision_id, &entry,
                get_link_fields(&event_entry_def_id, &process_entry_def_id, &entry_base_address)?
            )
        })
        .filter_map(Result::ok);

    let mut edge_cursors = edges.clone().map(|e| { e.cursor });
    let first_cursor = edge_cursors.next().unwrap_or("0".to_string());

    Ok(Collection {
        edges: edges.collect(),
        page_info: PageInfo {
            end_cursor: edge_cursors.last().unwrap_or(first_cursor.clone()),
            start_cursor: first_cursor,
            // :TODO:
            has_next_page: true,
            has_previous_page: true,
            page_limit: None,
            total_count: None,
        },
    })
}

/// Create response from input DHT primitives
pub fn construct_response<'a>(
    address: &EconomicResourceAddress, revision_id: &RevisionHash, e: &EntryData, (
        contained_in,
        stage,
        state,
        contains,
     ): (
        Option<EconomicResourceAddress>,
        Option<ProcessSpecificationAddress>,
        Option<ActionId>,
        Vec<EconomicResourceAddress>,
    ),
) -> RecordAPIResult<ResponseData> {
    Ok(ResponseData {
        economic_resource: construct_response_record(address, revision_id, e, (contained_in, stage, state, contains))?
    })
}

/// Create response from input DHT primitives
pub fn construct_response_record<'a>(
    address: &EconomicResourceAddress, revision_id: &RevisionHash, e: &EntryData, (
        contained_in,
        stage,
        state,
        contains,
     ): (
        Option<EconomicResourceAddress>,
        Option<ProcessSpecificationAddress>,
        Option<ActionId>,
        Vec<EconomicResourceAddress>,
    ),
) -> RecordAPIResult<Response> {
    Ok(Response {
        // entry fields
        id: address.to_owned(),
        revision_id: revision_id.to_owned(),
        conforms_to: e.conforms_to.to_owned(),
        classified_as: e.classified_as.to_owned(),
        tracking_identifier: e.tracking_identifier.to_owned(),
        lot: e.lot.to_owned(),
        image: e.image.to_owned(),
        accounting_quantity: e.accounting_quantity.to_owned(),
        onhand_quantity: e.onhand_quantity.to_owned(),
        unit_of_effort: e.unit_of_effort.to_owned(),
        stage: stage.to_owned(),
        state: state.to_owned(),
        current_location: e.current_location.to_owned(),
        note: e.note.to_owned(),

        // link fields
        contained_in: contained_in.to_owned(),
        contains: contains.to_owned(),
    })
}

pub fn construct_list_response<'a>(
    address: &EconomicResourceAddress, revision_id: &RevisionHash, e: &EntryData, (
        contained_in,
        stage,
        state,
        contains,
    ): (
        Option<EconomicResourceAddress>,
        Option<ProcessSpecificationAddress>,
        Option<ActionId>,
        Vec<EconomicResourceAddress>,
    )
) -> RecordAPIResult<Edge> {
    let record_cursor: Vec<u8> = address.to_owned().into();
    Ok(Edge {
        node: construct_response(address, revision_id, e, (contained_in, stage, state, contains))?.economic_resource,
        // :TODO: use HoloHashb64 once API stabilises
        cursor: String::from_utf8(record_cursor).unwrap_or("".to_string())
    })
}

// field list retrieval internals
// @see construct_response
pub fn get_link_fields<'a, S>(event_entry_def_id: S, process_entry_def_id: S, resource: &EconomicResourceAddress) -> RecordAPIResult<(
    Option<EconomicResourceAddress>,
    Option<ProcessSpecificationAddress>,
    Option<ActionId>,
    Vec<EconomicResourceAddress>,
)>
    where S: AsRef<str>
{
    Ok((
        read_index!(economic_resource(resource).contained_in)?.pop(),
        get_resource_stage(&event_entry_def_id, &process_entry_def_id, resource)?,
        get_resource_state(&event_entry_def_id, resource)?,
        read_index!(economic_resource(resource).contains)?,
    ))
}

fn get_resource_state<S>(event_entry_def_id: S, resource: &EconomicResourceAddress) -> RecordAPIResult<Option<ActionId>>
    where S: AsRef<str>
{
    let events: Vec<EconomicEventAddress> = get_affecting_events(resource)?;

    // grab the most recent "pass" or "fail" action
    Ok(events.iter()
        .rev()
        .fold(None, move |result, event| {
            // already found it, just fall through
            // :TODO: figure out the Rust STL method to abort on first Some() value
            if let Some(_) = result {
                return result;
            }

            let evt = read_record_entry::<EventData, EventStorage, _,_>(&event_entry_def_id, event.as_ref());
            match evt {
                Err(_) => result, // :TODO: this indicates some data integrity error
                Ok((_, _, entry)) => {
                    match &*String::from(entry.action.clone()) {
                        "pass" | "fail" => Some(entry.action),  // found it! Return this as the current resource state.
                        _ => result,    // still not located, keep looking...
                    }
                },
            }
        })
    )
}

fn get_resource_stage<S>(event_entry_def_id: S, process_entry_def_id: S, resource: &EconomicResourceAddress) -> RecordAPIResult<Option<ProcessSpecificationAddress>>
    where S: AsRef<str>
{
    let events: Vec<EconomicEventAddress> = get_affecting_events(resource)?;

    // grab the most recent event with a process output association
    Ok(events.iter()
        .rev()
        .fold(None, move |result, event| {
            // already found it, just fall through
            // :TODO: figure out the Rust STL method to abort on first Some() value
            if let Some(_) = result {
                return result;
            }

            let evt = read_record_entry::<EventData, EventStorage, _,_>(&event_entry_def_id, event.as_ref());
            match evt {
                Err(_) => result, // :TODO: this indicates some data integrity error
                Ok((_, _, entry)) => {
                    match &entry.output_of {
                        Some(output_of) => {
                            // get the associated process
                            let maybe_process_entry = read_record_entry::<ProcessData, ProcessStorage, _,_>(&process_entry_def_id, output_of.as_ref());
                            // check to see if it has an associated specification
                            match &maybe_process_entry {
                                Ok((_,_, process_entry)) => match &process_entry.based_on {
                                    Some(based_on) => Some(based_on.to_owned()),   // found it!
                                    None => result, // still not located, keep looking...
                                },
                                Err(_) => result, // :TODO: this indicates some data integrity error
                            }
                        },
                        None => result,    // still not located, keep looking...
                    }
                },
            }
        })
    )
}

/// Read all the EconomicEvents affecting a given EconomicResource
fn get_affecting_events(resource: &EconomicResourceAddress) -> RecordAPIResult<Vec<EconomicEventAddress>>
{
    read_index!(economic_resource(resource).affected_by)
}
