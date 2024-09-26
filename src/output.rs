use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    pub status: String,
    pub errmsg: String,
    pub lastop: u8,
    pub estack: Vec<StackItem>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct StackItem {
    #[serde(rename = "type")]
    pub itype: String,
    #[serde(rename = "value")]
    pub ivalue: Value,
}

pub fn parse(data: &Vec<u8>) -> Option<Output> {
    serde_json::from_slice(&data).ok()
}
