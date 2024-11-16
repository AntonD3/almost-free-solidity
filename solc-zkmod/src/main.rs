use std::fs;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use regex::Regex;
use sha3::Digest;

///
/// Compile solidity contract into EVM bytecode, returns hex string.
/// Returns creation bytecode if `runtime == false`, runtime otherwise.
///
fn compile_contract(source_code: &str, runtime: bool) -> String {
    let mut tmp_file = File::create("tmp.sol").unwrap();
    tmp_file.write_all(source_code.as_bytes()).unwrap();

    let mut solc = std::process::Command::new("solc");
    solc.stdin(std::process::Stdio::piped());
    solc.stdout(std::process::Stdio::piped());
    solc.stderr(std::process::Stdio::piped());
    solc.arg("--input-file");
    solc.arg("tmp.sol");
    if runtime {
        solc.arg("--bin-runtime");
    } else {
        solc.arg("--bin");
    }

    let result = solc.spawn().unwrap().wait_with_output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    let re = Regex::new(r":\s*([^\s]+)\s*$").unwrap();
    println!("{:?}",  String::from_utf8(result.stderr).unwrap());
    let bytecode = re.captures(&stdout).unwrap().get(1).unwrap().as_str().to_string();
    bytecode
}

#[derive(Clone)]
struct Param {
    pub r#type: String,
    pub memory_location: String,
    pub name: String,
}

impl Param {
    pub fn from_str(input: &str) -> Self {
        let re = Regex::new(r"^([^\s]+)\s+memory\s+([^\s]+)$").unwrap();
        if let Some(captures) = re.captures(input) {
            return Self {
                r#type: captures.get(1).unwrap().as_str().to_string(),
                memory_location: "memory".to_string(),
                name: captures.get(2).unwrap().as_str().to_string()
            };
        }
        let re = Regex::new(r"^([^\s]+)\s+([^\s]+)$").unwrap();
        if let Some(captures) = re.captures(input) {
            return Self {
                r#type: captures.get(1).unwrap().as_str().to_string(),
                memory_location: "".to_string(),
                name: captures.get(2).unwrap().as_str().to_string()
            };
        }
        panic!("Invalid source code");
    }

    pub fn definition(&self) -> String {
        format!("{} {} {}", self.r#type, self.memory_location, self.name)
    }
}

///
/// Returns solidity implementation of the execution verification using the `ExecutionOracle` contract.
///
fn execution_verification_sol(code_hash: &str, inputs: Vec<Param>, outputs: Vec<Param>) -> String {
    format!(r#"        // load needed witnesses from the scratch space after calldata
        bytes calldata output;
        bytes32[] calldata proof;
        assembly {{
            output.length := calldataload(calldataload(sub(calldatasize(), 32)))
            output.offset := add(calldataload(sub(calldatasize(), 32)), 32)
            proof.length := calldataload(add(output.offset, output.length))
            proof.offset := add(add(output.offset, output.length), 32)
        }}

        // verify execution
        bytes memory input = abi.encode({});
        bytes32 inputHash = keccak256(input);
        bytes32 outputHash = keccak256(output);
        bytes memory calldata_buffer = new bytes(100 + proof.length);
        assembly {{
            mstore(add(calldata_buffer, 32), 0x38419e1000000000000000000000000000000000000000000000000000000000)
            mstore(add(calldata_buffer, 36), 0x{})
            mstore(add(calldata_buffer, 68), inputHash)
            mstore(add(calldata_buffer, 100), outputHash)
            calldatacopy(add(calldata_buffer, 132), add(proof.offset, 32), calldataload(proof.offset))
            let success := call(gas(), {}, 0, add(calldata_buffer, 32), mload(calldata_buffer), 0, 0)
            if iszero(success) {{
                mstore(0, 0xdeadbeef)
                mstore(32, 0x{})
                let inputLength := mload(input)
                mcopy(64, add(input, 32), inputLength)
                revert(0, add(64, inputLength))
            }}
        }}
        return abi.decode(output, ({}));"#,
        inputs.iter().map(|arg| arg.name.clone()).collect::<Vec<_>>().as_slice().join(", "),
        code_hash,
        EXEUCTION_ORACLE_ADDRESS,
        code_hash,
        outputs.iter().map(|arg| arg.r#type.clone()).collect::<Vec<_>>().as_slice().join(", "),
    )
}

fn provable_function_wrapper_contract(function_body: &str, function_name: &str, inputs: Vec<Param>, function_definition: &str) -> String {
    format!(r#"
contract Provable {{
    fallback(bytes calldata input) external payable returns(bytes memory output) {{
        ({}) = abi.decode(input, ({}));
        return abi.encode({}({}));
    }}

{}
    {}
    }}
}}"#,
        inputs.iter().map(|arg| arg.definition()).collect::<Vec<_>>().as_slice().join(", "),
        inputs.iter().map(|arg| arg.r#type.clone()).collect::<Vec<_>>().as_slice().join(", "),
        function_name,
        inputs.iter().map(|arg| arg.name.clone()).collect::<Vec<_>>().as_slice().join(", "),
        function_definition,
        function_body
    )
}

// TODO:
const EXEUCTION_ORACLE_ADDRESS: &'static str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const VERIFY_COMPUTATION_FUNCTION_SELECTOR: &'static str = "0xTODO";

///
/// We are pasting witnesses to the calldata, but we don't want to change the calldatasize from the contract perspective.
/// So we are adding pure calldatasize(without witnesses) to the end of the calldata and this value should be used by the contract instead of calldatasize.
///
fn preprocess_calldatasize(bytecode: String) -> String {
    // calldatasize
    // ->
    // push1 0x20
    // calldatasize
    // sub
    // calldataload
    let mut result = String::new();
    let mut pref_updated = vec![0u8; bytecode.len() / 2];
    for i in 0..bytecode.len()/2 {
        if i != 0 {
            pref_updated[i] = pref_updated[i - 1];
        }
        if &bytecode[i*2..i*2+2] == "36" {
            result.push_str("6020360335");
            pref_updated[i] += 1;
        } else {
            result.push_str(&bytecode[i*2..i*2+2]);
        }
    }
    // we should update jump dests, as code was changed
    for i in 0..result.len() / 2 {
        if &result[i*2..i*2+2] == "56" || &result[i*2..i*2+2] == "57" {
            let mut dest = u8::from_str_radix(&result[i*2-2..i*2], 16).unwrap();
            dest.checked_add(pref_updated[dest as usize] * 4).expect("Jump dest overflow");
            let encoded = hex::encode(&[dest]);
            result.replace_range(i*2-2..i*2, &encoded);
        }
    }
    result
}

fn bytecode_hash(bytecode: &str) -> String {
    let mut keccak = sha3::Keccak256::default();
    keccak.update(hex::decode(bytecode).unwrap().as_slice());
    hex::encode(&keccak.finalize()[..])
}

fn main() {
    let file_path = std::env::args().nth(1).unwrap();
    let content = fs::read_to_string(file_path).unwrap();
    #[derive(PartialEq)]
    enum Action {
        Nothing,
        ReadDefinition,
        ReadBody
    };
    let mut action = Action::Nothing;

    let mut current_body = String::new();
    let mut current_definition = String::new();
    let mut current_name = String::new();
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mut result = String::new();
    for line in content.lines() {
        if line == "    @free" {
            if action != Action::Nothing {
                panic!("Invalid source");;
            }
            action = Action::ReadDefinition;
        } else if action == Action::ReadDefinition {
            result.push_str(line);
            result.push('\n');
            current_definition = line.to_string();
            let re = Regex::new(r"\s*function\s+(\w+)\(([^)]*)\)[^)]*\(([^)]*)\)").unwrap();
            let captures = re.captures(&current_definition).unwrap();
            current_name = captures.get(1).unwrap().as_str().to_string();
            let inputs_str = captures.get(2).unwrap().as_str();
            let outputs_str = captures.get(3).unwrap().as_str();
            inputs = inputs_str.split(", ").map(|arg| Param::from_str(arg)).collect();
            outputs = outputs_str.split(", ").map(|arg| Param::from_str(arg)).collect();
            action = Action::ReadBody
        } else if action == Action::ReadBody {
            if line == "    }" {
                let function_wrapper = provable_function_wrapper_contract(&current_body, &current_name, inputs.clone(), &current_definition);
                let bytecode = compile_contract(&function_wrapper, true);
                println!("Function bytecode: {}", bytecode);
                let bytecode_hash = bytecode_hash(&bytecode);
                let body_to_verify = execution_verification_sol(&bytecode_hash, inputs.clone(), outputs.clone());
                result.push_str(&body_to_verify);
                result.push('\n');
                result.push_str(line);
                result.push('\n');
                action = Action::Nothing;
            } else {
                current_body.push_str(line);
                current_body.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // println!("{}", result);
    println!("Contract bytecode {}", compile_contract(&result, false));
}
