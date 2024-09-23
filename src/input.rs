use base64::prelude::*;
use libafl::inputs::{HasTargetBytes, Input};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ByteCodeInput {
    pub opcodes: Vec<u8>,
}

impl HasTargetBytes for ByteCodeInput {
    fn target_bytes(&self) -> libafl_bolts::prelude::OwnedSlice<u8> {
        BASE64_STANDARD
            .encode(&self.opcodes)
            .as_bytes()
            .to_vec()
            .into()
    }
}

impl Input for ByteCodeInput {
    fn generate_name(&self, _: Option<libafl::prelude::CorpusId>) -> String {
        BASE64_STANDARD.encode(&self.opcodes)
    }
}
