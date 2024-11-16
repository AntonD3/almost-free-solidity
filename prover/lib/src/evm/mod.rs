pub mod block;
mod consts;
pub mod context;
mod eval;
pub mod helpers;
mod jump_map;
mod machine;
mod memory;
mod opcode;
mod stack;

use context::Context;
use machine::EvmResult;
use machine::Machine;

pub fn evm(
    code: impl AsRef<[u8]>,
    context: Context,
) -> EvmResult {
    Machine::new(code.as_ref(), context).execute()
}