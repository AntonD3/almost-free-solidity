use crate::evm::consts::WORD_BYTES;
use crate::evm::machine::{ControlFlow, EvmError, ExitReason, ExitSuccess, Machine};
use crate::evm::opcode::Opcode;
use crate::evm::{helpers::*};
use primitive_types::{U256};
use sha3::{Digest, Keccak256};

pub fn eval(machine: &mut Machine) -> ControlFlow {
    let opcode = machine.opcode();
    match opcode {
        Opcode::STOP => stop(machine),
        Opcode::ADD => add(machine),
        Opcode::MUL => mul(machine),
        Opcode::SUB => sub(machine),
        Opcode::DIV => div(machine),
        Opcode::SDIV => sdiv(machine),
        Opcode::MOD => modulus(machine),
        Opcode::SMOD => smodulus(machine),
        Opcode::ADDMOD => add_modulus(machine),
        Opcode::MULMOD => mul_modulus(machine),
        Opcode::EXP => exp(machine),
        Opcode::SIGNEXTEND => sign_extend(machine),
        Opcode::LT => lt(machine),
        Opcode::GT => gt(machine),
        Opcode::SLT => slt(machine),
        Opcode::SGT => sgt(machine),
        Opcode::EQ => eq(machine),
        Opcode::ISZERO => iszero(machine),
        Opcode::AND => and(machine),
        Opcode::OR => or(machine),
        Opcode::XOR => xor(machine),
        Opcode::NOT => not(machine),
        Opcode::BYTE => byte(machine),
        Opcode::SHL => shl(machine),
        Opcode::SHR => shr(machine),
        Opcode::SAR => sar(machine),
        Opcode::KECCAK256 => keccak256(machine),
        Opcode::CALLDATALOAD => calldataload(machine),
        Opcode::CALLDATASIZE => calldatasize(machine),
        Opcode::CALLDATACOPY => calldatacopy(machine),

        Opcode::POP => eval_pop(machine),
        Opcode::MLOAD => mload(machine),
        Opcode::MSTORE => mstore(machine),

        Opcode::REVERT => revert(machine),
        Opcode::INVALID => invalid(machine),

        Opcode::MSTORE8 => mstore8(machine),
        Opcode::JUMP => jump(machine),
        Opcode::JUMPI => jumpi(machine),
        Opcode::PC => pc(machine),
        Opcode::MSIZE => msize(machine),

        Opcode::PUSH0 => push_zero(machine),
        Opcode::PUSH1..=Opcode::PUSH32 => eval_push(machine),
        Opcode::DUP1..=Opcode::DUP16 => dup(machine),
        Opcode::SWAP1..=Opcode::SWAP16 => swap(machine),

        Opcode::RETURN => eval_return(machine),

        Opcode::DELEGATECALL => forbidden(machine),
        Opcode::STATICCALL => forbidden(machine),
        Opcode::SELFDESTRUCT => forbidden(machine),
        Opcode::ADDRESS => forbidden(machine),
        Opcode::BALANCE => forbidden(machine),
        Opcode::ORIGIN => forbidden(machine),
        Opcode::CALLER => forbidden(machine),
        Opcode::CODESIZE => forbidden(machine),
        Opcode::CODECOPY => forbidden(machine),
        Opcode::BLOCKHASH => forbidden(machine),
        Opcode::GASPRICE => forbidden(machine),
        Opcode::EXTCODESIZE => forbidden(machine),
        Opcode::EXTCODECOPY => forbidden(machine),
        Opcode::EXTCODEHASH => forbidden(machine),
        Opcode::RETURNDATASIZE => forbidden(machine),
        Opcode::RETURNDATACOPY => forbidden(machine),
        Opcode::COINBASE => forbidden(machine),
        Opcode::TIMESTAMP => forbidden(machine),
        Opcode::NUMBER => forbidden(machine),
        Opcode::DIFFICULTY => forbidden(machine),
        Opcode::GASLIMIT => forbidden(machine),
        Opcode::CHAINID => forbidden(machine),
        Opcode::SELFBALANCE => forbidden(machine),
        Opcode::BASEFEE => forbidden(machine),
        Opcode::SLOAD => forbidden(machine),
        Opcode::SSTORE => forbidden(machine),
        Opcode::GAS => forbidden(machine),
        Opcode::JUMPDEST => jumpdest(machine),
        Opcode::LOG0..=Opcode::LOG4 => forbidden(machine),
        Opcode::CREATE => forbidden(machine),
        Opcode::CALL => forbidden(machine),
        // TODO: if fails - use zero
        Opcode::CALLVALUE => forbidden(machine),

        _ => exit_error(EvmError::InvalidInstruction),
    }
}

// TODO: remove unwraps and handle failed stack pops
// TODO: remove unnecessary mut references for machine
// TODO: add and handle as_usize or fail
// TODO: add 1024 stack limit

fn stop(_machine: &mut Machine) -> ControlFlow {
    exit_success(ExitSuccess::Stop)
}

fn add(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let res = a.overflowing_add(b).0;
    machine.stack.push(res);

    ControlFlow::Continue(1)
}

fn mul(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let res = a.overflowing_mul(b).0;
    machine.stack.push(res);

    ControlFlow::Continue(1)
}

fn sub(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let res = a.overflowing_sub(b).0;
    machine.stack.push(res);

    ControlFlow::Continue(1)
}

fn div(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let res = a.checked_div(b);
    match res {
        Some(result) => machine.stack.push(result),
        None => machine.stack.push(0.into()),
    }

    ControlFlow::Continue(1)
}

fn sdiv(machine: &mut Machine) -> ControlFlow {
    let mut a = machine.stack.pop().unwrap();
    let mut b = machine.stack.pop().unwrap();

    // If the first bit is 1, then the value is negative, according to the rules of two's compliment
    let a_is_negative = is_negative(a);
    let b_is_negative = is_negative(b);

    // If the value is negative, we need to switch it into a positive value
    if a_is_negative {
        a = convert_twos_compliment(a);
    }
    // We do this for either of the numbers if they are negative, to find their absolute value
    if b_is_negative {
        b = convert_twos_compliment(b);
    }

    // now res = |a| / |b|
    let res = a.checked_div(b);

    match res {
        Some(mut result) => match result {
            // if the result is 0, push 0 straight onto stack
            i if i == 0.into() => machine.stack.push(i),
            _ => {
                // If only one of the numbers is negative, the result will be negative
                if a_is_negative ^ b_is_negative {
                    // We need to perform two's compliment again to provide a negative result
                    result = convert_twos_compliment(result);
                }
                machine.stack.push(result);
            }
        },
        None => machine.stack.push(U256::zero()),
    }

    ControlFlow::Continue(1)
}

fn modulus(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let res = a.checked_rem(b);
    match res {
        Some(result) => machine.stack.push(result),
        None => machine.stack.push(0.into()),
    }

    ControlFlow::Continue(1)
}

fn smodulus(machine: &mut Machine) -> ControlFlow {
    let mut a = machine.stack.pop().unwrap();
    let mut b = machine.stack.pop().unwrap();

    let a_is_negative = is_negative(a);
    let b_is_negative = is_negative(b);

    if a_is_negative {
        a = convert_twos_compliment(a);
    }
    if b_is_negative {
        b = convert_twos_compliment(b);
    }

    let res = a.checked_rem(b);

    match res {
        Some(mut result) => match result {
            i if i == 0.into() => machine.stack.push(i),
            _ => {
                if a_is_negative {
                    result = convert_twos_compliment(result);
                }
                machine.stack.push(result);
            }
        },
        None => machine.stack.push(0.into()),
    }

    ControlFlow::Continue(1)
}

fn add_modulus(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let c = machine.stack.pop().unwrap();
    let res = a.overflowing_add(b).0.checked_rem(c);
    match res {
        Some(result) => machine.stack.push(result),
        None => machine.stack.push(0.into()),
    }

    ControlFlow::Continue(1)
}

fn mul_modulus(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let c = machine.stack.pop().unwrap();
    let res_mul = a.full_mul(b);
    let res_modulo = res_mul.checked_rem(c.into());
    match res_modulo {
        Some(result) => machine
            .stack
            .push(result.try_into().expect(
                "c <= U256::MAX, result = res_mul % c, ∴ result <  U256::MAX, ∴ overflow impossible; qed"
            )),
        None => machine.stack.push(0.into()),
    }

    ControlFlow::Continue(1)
}

fn exp(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let res = a.overflowing_pow(b).0;
    machine.stack.push(res);

    ControlFlow::Continue(1)
}

// extend a signed integer to 32 bytes
// a = num_bytes = the number of bytes of the integer to extend - 1
// b = int_to_extend = the integer to extend
// e.g.
// a = 0, b = 00000001, int_to_extend with bytes => 00000001
// a = 1, b = 00000001, int_to_extend with bytes => 0000000000000001
// a = 1, b = 11111111, int_to_extend with bytes => 0000000011111111
// Full example:
// a = 0, b = 11111110, int_to_extend with bytes => 11111110
// bit_index = (8 * 0) + 7 = 7
// bit = 1
// mask  = 0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000011111111
// !mask = 1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111100000000
// res = int_to_extend | !mask
// = 11111110 | 1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111100000000
// = 1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111110
// = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE
fn sign_extend(machine: &mut Machine) -> ControlFlow {
    let num_bytes = machine.stack.pop().unwrap();
    let int_to_extend = machine.stack.pop().unwrap();

    if num_bytes >= U256::from(32) {
        // int is already fully extended, EVM is max 256 bits, 32 bytes = 256 bits
        // ∴ push int_to_extend straight to stack
        machine.stack.push(int_to_extend);
    } else {
        // t is the index from left to right of the first bit of the int_to_extend in a 32-byte word
        // x = num_bytes
        // t = 256 - 8(x + 1)
        // rearrange t to find the index from left to right
        // s = 255 - t = 8(x + 1)
        // where s is the index from left to right of the first bit of the int_to_extend in a 32-byte word
        // `low_u32` works since num_bytes < 32
        let bit_index = (8 * num_bytes.low_u32() + 7) as usize;
        // find whether the bit at bit_index is 1 or 0
        let bit = int_to_extend.bit(bit_index);
        // create a mask of 0s up to bit_index and then 1s from then on
        let mask = (U256::one() << bit_index) - U256::one();
        if bit {
            // append 1s to int_to_extend
            machine.stack.push(int_to_extend | !mask);
        } else {
            // append 0s to int_to_extend
            machine.stack.push(int_to_extend & mask);
        }
    }
    ControlFlow::Continue(1)
}

fn lt(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let res = (a < b) as u32;
    machine.stack.push(U256::from(res));

    ControlFlow::Continue(1)
}

fn gt(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();
    let res = (a > b) as u32;
    machine.stack.push(U256::from(res));

    ControlFlow::Continue(1)
}

fn slt(machine: &mut Machine) -> ControlFlow {
    let mut a = machine.stack.pop().unwrap();
    let mut b = machine.stack.pop().unwrap();

    if a == b {
        machine.stack.push(U256::zero());
        return ControlFlow::Continue(1);
    }

    let a_is_negative = is_negative(a);
    let b_is_negative = is_negative(b);

    if a_is_negative && !b_is_negative {
        machine.stack.push(U256::one());
        return ControlFlow::Continue(1);
    } else if !a_is_negative && b_is_negative {
        machine.stack.push(U256::zero());
        return ControlFlow::Continue(1);
    }

    if a_is_negative {
        a = convert_twos_compliment(a);
    }
    if b_is_negative {
        b = convert_twos_compliment(b);
    }

    let mut res = a < b;

    if a_is_negative && b_is_negative {
        res = !res;
    }

    machine.stack.push(U256::from(res as u32));

    ControlFlow::Continue(1)
}

fn sgt(machine: &mut Machine) -> ControlFlow {
    let mut a = machine.stack.pop().unwrap();
    let mut b = machine.stack.pop().unwrap();

    if a == b {
        machine.stack.push(U256::zero());
        return ControlFlow::Continue(1);
    }

    let a_is_negative = is_negative(a);
    let b_is_negative = is_negative(b);

    if a_is_negative && !b_is_negative {
        machine.stack.push(U256::zero());
        return ControlFlow::Continue(1);
    } else if !a_is_negative && b_is_negative {
        machine.stack.push(U256::one());
        return ControlFlow::Continue(1);
    }

    if a_is_negative {
        a = convert_twos_compliment(a);
    }
    if b_is_negative {
        b = convert_twos_compliment(b);
    }

    let mut res = a > b;

    if a_is_negative && b_is_negative {
        res = !res;
    }

    machine.stack.push(U256::from(res as u32));

    ControlFlow::Continue(1)
}

fn eq(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();

    if a == b {
        machine.stack.push(U256::one());
    } else {
        machine.stack.push(U256::zero());
    }

    ControlFlow::Continue(1)
}

fn iszero(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();

    if a == U256::zero() {
        machine.stack.push(U256::one());
    } else {
        machine.stack.push(U256::zero());
    }

    ControlFlow::Continue(1)
}

fn not(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();

    machine.stack.push(!a);

    ControlFlow::Continue(1)
}

fn byte(machine: &mut Machine) -> ControlFlow {
    let byte_offset = machine.stack.pop().unwrap();
    let value = machine.stack.pop().unwrap();

    if byte_offset >= 32.into() {
        machine.stack.push(U256::zero());
        return ControlFlow::Continue(1);
    }

    let byte_index = U256::from(31) - byte_offset;

    let res = value.byte(byte_index.as_usize());

    machine.stack.push(res.into());

    ControlFlow::Continue(1)
}

fn and(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();

    machine.stack.push(a & b);

    ControlFlow::Continue(1)
}

fn or(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();

    machine.stack.push(a | b);

    ControlFlow::Continue(1)
}

fn xor(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let b = machine.stack.pop().unwrap();

    machine.stack.push(a ^ b);

    ControlFlow::Continue(1)
}

fn shl(machine: &mut Machine) -> ControlFlow {
    let shift = machine.stack.pop().unwrap();
    let value = machine.stack.pop().unwrap();

    let shifted = value << shift;
    machine.stack.push(shifted);

    ControlFlow::Continue(1)
}

fn shr(machine: &mut Machine) -> ControlFlow {
    let shift = machine.stack.pop().unwrap();
    let value = machine.stack.pop().unwrap();

    let shifted = value >> shift;
    machine.stack.push(shifted);

    ControlFlow::Continue(1)
}

fn sar(machine: &mut Machine) -> ControlFlow {
    // shift value is unsigned
    let shift = machine.stack.pop().unwrap();
    // value is signed
    let mut value = machine.stack.pop().unwrap();

    let value_is_negative = is_negative(value);

    if value_is_negative {
        value = convert_twos_compliment(value);
    }

    let mut shifted = value >> shift;

    if value_is_negative {
        shifted = convert_twos_compliment(shifted);
    }

    machine.stack.push(shifted);

    ControlFlow::Continue(1)
}

fn keccak256(machine: &mut Machine) -> ControlFlow {
    let offset = machine.stack.pop().unwrap();
    let size = machine.stack.pop().unwrap();

    let data_to_hash = machine.memory.get(offset.as_usize(), size.as_usize());
    let hashed_data = Keccak256::digest(data_to_hash);

    machine.stack.push(U256::from_big_endian(&hashed_data));

    ControlFlow::Continue(1)
}

fn calldataload(machine: &mut Machine) -> ControlFlow {
    let byte_offset = machine.stack.pop().unwrap();

    machine.stack.push(
        machine
            .context
            .load_calldata(byte_offset.as_usize(), WORD_BYTES),
    );

    ControlFlow::Continue(1)
}

fn calldatasize(machine: &mut Machine) -> ControlFlow {
    machine.stack.push(machine.context.calldata_size());

    ControlFlow::Continue(1)
}

// TODO: move all possible .as_usize()'s to the initial values
fn calldatacopy(machine: &mut Machine) -> ControlFlow {
    let dest_offset = machine.stack.pop().unwrap();
    let offset = machine.stack.pop().unwrap();
    let size = machine.stack.pop().unwrap();

    let calldata = machine
        .context
        .load_calldata(offset.as_usize(), size.as_usize());

    machine
        .memory
        .set(dest_offset.as_usize(), calldata, size.as_usize());

    ControlFlow::Continue(1)
}

fn eval_pop(machine: &mut Machine) -> ControlFlow {
    machine.stack.pop();

    ControlFlow::Continue(1)
}

fn mload(machine: &mut Machine) -> ControlFlow {
    let byte_offset = machine.stack.pop().unwrap();

    let res = machine.memory.get(byte_offset.as_usize(), WORD_BYTES);
    let res_word = U256::from_big_endian(res);

    machine.stack.push(res_word);
    ControlFlow::Continue(1)
}

fn mstore(machine: &mut Machine) -> ControlFlow {
    let byte_offset = machine.stack.pop().unwrap();
    let value = machine.stack.pop().unwrap();

    machine
        .memory
        .set(byte_offset.as_usize(), value, WORD_BYTES);

    ControlFlow::Continue(1)
}

fn mstore8(machine: &mut Machine) -> ControlFlow {
    let byte_offset = machine.stack.pop().unwrap();
    let value = machine.stack.pop().unwrap();

    machine.memory.set(byte_offset.as_usize(), value, 1);

    ControlFlow::Continue(1)
}

fn forbidden(machine: &mut Machine) -> ControlFlow {
    println!("{}", machine.opcode());
    let forbidden_code: U256 = U256::from(123456);

    ControlFlow::Exit(ExitReason::Error(
        EvmError::Revert(forbidden_code)
    ))
}

fn jump(machine: &mut Machine) -> ControlFlow {
    let a = machine.stack.pop().unwrap();
    let is_valid = true;
    // let is_valid = machine.jump_map.is_valid(a);

    if is_valid {
        ControlFlow::Jump(a.as_usize())
    } else {
        exit_error(EvmError::InvalidJump)
    }
}

fn jumpi(machine: &mut Machine) -> ControlFlow {
    let jump_to = machine.stack.pop().unwrap();
    let should_jump = machine.stack.pop().unwrap();

    if should_jump.is_zero() {
        return ControlFlow::Continue(1);
    }

    let is_valid = true;
    // let is_valid = machine.jump_map.is_valid(jump_to);

    if is_valid {
        ControlFlow::Jump(jump_to.as_usize())
    } else {
        exit_error(EvmError::InvalidJump)
    }
}

fn pc(machine: &mut Machine) -> ControlFlow {
    machine.stack.push(machine.pc.into());

    ControlFlow::Continue(1)
}

fn msize(machine: &mut Machine) -> ControlFlow {
    let res = machine.memory.size();
    machine.stack.push(res.into());

    ControlFlow::Continue(1)
}

fn gas(machine: &mut Machine) -> ControlFlow {
    // TODO: update to calculate gas properly (update tests first)
    machine.stack.push(U256::MAX);

    ControlFlow::Continue(1)
}
fn jumpdest(_machine: &mut Machine) -> ControlFlow {
    ControlFlow::Continue(1)
}

fn push_zero(machine: &mut Machine) -> ControlFlow {
    machine.stack.push(U256::zero());

    ControlFlow::Continue(1)
}

fn eval_push(machine: &mut Machine) -> ControlFlow {
    let n = usize::from(machine.opcode() - (Opcode::PUSH1 - 1));
    let start = machine.pc + 1;
    let end = start + n;
    let bytes = &machine.code[start..end];
    let val_to_push = U256::from_big_endian(bytes);
    machine.stack.push(val_to_push);

    ControlFlow::Continue(n + 1)
}

fn dup(machine: &mut Machine) -> ControlFlow {
    let n = usize::from(machine.opcode() - Opcode::DUP1);

    let a = machine.stack.peek(n);

    match a {
        Ok(val) => machine.stack.push(val),
        Err(error) => return exit_error(error),
    }

    ControlFlow::Continue(1)
}

fn swap(machine: &mut Machine) -> ControlFlow {
    let n = usize::from(machine.opcode() - (Opcode::SWAP1 - 1));

    let a = match machine.stack.peek(0) {
        Ok(val) => val,
        Err(err) => return exit_error(err),
    };
    let b = match machine.stack.peek(n) {
        Ok(val) => val,
        Err(err) => return exit_error(err),
    };

    match machine.stack.set(a, n) {
        Ok(()) => (),
        Err(err) => return exit_error(err),
    }
    match machine.stack.set(b, 0) {
        Ok(()) => (),
        Err(err) => return exit_error(err),
    }

    ControlFlow::Continue(1)
}

// enum CallType {
//     CALL,
//     DELEGATECALL,
//     STATICCALL,
//     CREATE
// }

// fn message_call (machine: &mut Machine, call_type: CallType) -> ControlFlow {
//     ControlFlow::Continue(1)
// }

fn eval_return(machine: &mut Machine) -> ControlFlow {
    let offset = machine.stack.pop().unwrap().as_usize();
    let size = machine.stack.pop().unwrap().as_usize();

    let res = machine.memory.get(offset, size);

    exit_success(ExitSuccess::Return(res.to_vec()))
}

fn revert(machine: &mut Machine) -> ControlFlow {
    let offset = machine.stack.pop().unwrap().as_usize();
    let size = machine.stack.pop().unwrap().as_usize();

    let res = machine.memory.get(offset, size);

    exit_error(EvmError::Revert(U256::from_big_endian(res)))
}

fn invalid(_machine: &mut Machine) -> ControlFlow {
    exit_error(EvmError::InvalidInstruction)
}
