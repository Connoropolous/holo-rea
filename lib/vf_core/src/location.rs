// :TODO: this is likely to be deprecated
// @see https://github.com/holo-rea/ecosystem/wiki/Modules-in-the-HoloREA-framework#location-mapping

use hdk::{
    holochain_json_api::{
        json::JsonString,
        error::JsonError,
    },
};
use holochain_json_derive::{ DefaultJson };

vfRecord! {
    pub struct Location {
        name: String,
        address: Option<String>,
        latitude: Option<f32>,
        longitude: Option<f32>,
    }
}