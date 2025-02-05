#![feature(proc_macro_hygiene)]
/**
 * Beef industry `EconomicResource` zome API definition
 *
 * Zome module which extends the standard REA `EconomicResource` module with
 * industry-specific attributes applicable in the beef trade.
 *
 * @package Holo-REA
 */
extern crate serde;
extern crate hdk;
extern crate hdk_proc_macros;

use hdk::prelude::*;
use hdk_proc_macros::zome;

use hc_zome_rea_resource_specification_storage_consts::{
    ECONOMIC_RESOURCE_SPECIFICATION_BASE_ENTRY_TYPE,
    RESOURCE_SPECIFICATION_CONFORMING_RESOURCE_LINK_TYPE,
};
use hc_zome_rea_economic_resource_storage_consts::RESOURCE_BASE_ENTRY_TYPE;

use hc_zome_rea_economic_resource_defs::{ entry_def, base_entry_def };
use hc_zome_rea_economic_resource_lib::*;
use hc_zome_rea_economic_resource_rpc::*;
use hc_zome_rea_economic_event_rpc::ResourceResponseData as ResponseData;

#[zome]
mod beef_economic_resource_zome {

    #[init]
    fn init() {
        Ok(())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData::<AgentId>) {
        Ok(())
    }

    #[entry_def]
    fn resource_entry_def() -> ValidatingEntryType {
        entry_def()
    }

    #[entry_def]
    fn resource_base_entry_def() -> ValidatingEntryType {
        base_entry_def()
    }

    // :TODO: move to separate zome
    #[entry_def]
    fn resource_specification_base_entry_def() -> ValidatingEntryType {
        entry!(
            name: ECONOMIC_RESOURCE_SPECIFICATION_BASE_ENTRY_TYPE,
            description: "Base anchor for external ResourceSpecification records to provide lookup functionality",
            sharing: Sharing::Public,
            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },
            validation: |_validation_data: hdk::EntryValidationData<Address>| {
                Ok(())
            },
            links: [
                to!(
                    RESOURCE_BASE_ENTRY_TYPE,
                    link_type: RESOURCE_SPECIFICATION_CONFORMING_RESOURCE_LINK_TYPE,
                    validation_package: || {
                        hdk::ValidationPackageDefinition::Entry
                    },
                    validation: | _validation_data: hdk::LinkValidationData| {
                        Ok(())
                    }
                )
            ]
        )
    }

    #[zome_fn("hc_public")]
    fn get_resource(address: EconomicResourceAddress) -> ZomeApiResult<ResponseData> {
        handle_get_economic_resource(address)
    }

    #[zome_fn("hc_public")]
    fn update_resource(resource: UpdateRequest) -> ZomeApiResult<ResponseData> {
        handle_update_economic_resource(resource)
    }


    #[zome_fn("hc_public")]
    fn query_resources(params: QueryParams) -> ZomeApiResult<Vec<ResponseData>> {
        handle_query_economic_resources(params)
    }


    // :TODO:
    // receive: |from, payload| {
    //     format!("Received: {} from {}", payload, from)
    // }

}
