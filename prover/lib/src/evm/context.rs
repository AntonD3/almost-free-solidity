use primitive_types::{U256};

pub struct Context<'a> {
    // TODO: update to U256, handle overlap with State.Account.balance -> maybe not
    pub call_data: &'a [u8],
}

// TODO: remove lifetime parameter where possible
impl<'a> Context<'a> {
    pub fn new(
        call_data: &'a [u8],
    ) -> Self {
        Self {
            call_data,
        }
    }

    pub fn calldata_size(&self) -> U256 {
        let call_data_size = self.call_data.len();
        call_data_size.into()
    }

    pub fn load_calldata(&self, byte_offset: usize, target_size: usize) -> U256 {
        let mut res: Vec<u8> = vec![0; target_size];

        for i in 0..target_size {
            let data_index = i + byte_offset;
            if data_index < self.call_data.len() {
                let val = self.call_data[data_index];
                res[i] = val;
            }
        }

        U256::from_big_endian(&res)
    }
}