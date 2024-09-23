use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct Output {
    pub status: String,
    pub errmsg: String,
    pub lastop: u8,
    pub estack: Vec<StackItem>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct StackItem {
    #[serde(rename = "type")]
    pub itype: String,
    #[serde(rename = "value")]
    pub ivalue: Value,
}

pub fn parse(data: &Vec<u8>) -> Output {
    serde_json::from_slice(&data).expect(&format!(
        "failed to read json: {}",
        String::from_utf8(data.clone()).unwrap()
    ))
}
