use base64::prelude::*;
use libafl::inputs::{HasMutatorBytes, HasTargetBytes, Input};
use libafl_bolts::HasLen;
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

impl HasMutatorBytes for ByteCodeInput {
    fn bytes(&self) -> &[u8] {
        &self.opcodes
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.opcodes
    }

    fn resize(&mut self, new_len: usize, value: u8) {
        self.opcodes.resize(new_len, value);
    }

    fn extend<'a, I: IntoIterator<Item = &'a u8>>(&mut self, iter: I) {
        self.opcodes.extend(iter);
    }

    fn splice<R, I>(&mut self, range: R, replace_with: I) -> std::vec::Splice<'_, I::IntoIter>
    where
        R: std::ops::RangeBounds<usize>,
        I: IntoIterator<Item = u8>,
    {
        self.opcodes.splice(range, replace_with)
    }

    fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, u8>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.opcodes.drain(range)
    }
}

impl HasLen for ByteCodeInput {
    fn len(&self) -> usize {
        self.opcodes.len()
    }
}

impl Input for ByteCodeInput {
    fn generate_name(&self, _: Option<libafl::prelude::CorpusId>) -> String {
        BASE64_STANDARD.encode(&self.opcodes)
    }
}
