/**
 * Holo-REA fulfillment zome API definition
 *
 * Defines the top-level zome configuration needed by Holochain's build system
 * to bundle the app. This basically involves wiring up the helper methods from the
 * related `_lib` module into a packaged zome WASM binary.
 *
 * @package Holo-REA
 */
use hdk::prelude::*;

use hc_zome_rea_fulfillment_lib_origin::*;
use hc_zome_rea_fulfillment_rpc::*;
use hc_zome_rea_fulfillment_storage_consts::*;

#[hdk_extern]
fn entry_defs(_: ()) -> ExternResult<EntryDefsCallbackResult> {
    Ok(EntryDefsCallbackResult::from(vec![
        temp_path::path::Path::entry_def(),
        EntryDef {
            id: FULFILLMENT_ENTRY_TYPE.into(),
            visibility: EntryVisibility::Public,
            crdt_type: CrdtType,
            required_validations: 1.into(),
            required_validation_type: RequiredValidationType::default(),
        }
    ]))
}

#[hdk_extern]
fn create_fulfillment(CreateParams { fulfillment }: CreateParams) -> ExternResult<ResponseData> {
    Ok(handle_create_fulfillment(FULFILLMENT_ENTRY_TYPE, fulfillment)?)
}

#[hdk_extern]
fn get_fulfillment(ByAddress { address }: ByAddress<FulfillmentAddress>) -> ExternResult<ResponseData> {
    Ok(handle_get_fulfillment(FULFILLMENT_ENTRY_TYPE, address)?)
}

#[hdk_extern]
fn update_fulfillment(UpdateParams { fulfillment }: UpdateParams) -> ExternResult<ResponseData> {
    Ok(handle_update_fulfillment(FULFILLMENT_ENTRY_TYPE, fulfillment)?)
}

#[hdk_extern]
fn delete_fulfillment(ByHeader { address }: ByHeader) -> ExternResult<bool> {
    Ok(handle_delete_fulfillment(address)?)
}
