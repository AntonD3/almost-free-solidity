
///
/// Compile solidity contract into EVM bytecode, returns hex string.
/// Returns creation bytecode if `runtime == false`, runtime otherwise.
///
fn compile_contract(source_code: &str, runtime: bool) -> &str {

}

///
/// Returns solidity implementation of the execution verification using the `ExecutionOracle` contract.
///
fn execution_verification_sol(code_hash: &str, args_names: Vec<&str>, output_types: Vec<&str>) -> String {
    format!(r#"// load needed witnesses from the scratch space after calldata
bytes calldata output;
bytes32[] calldata proof;
assembly {{
    output := add(calldatasize(), 32)
    let outSize := calldataload(output)
    proof := add(output, outSize)
}}

// verify execution
bytes32 inputHash = keccak256(abi.encode({}));
bytes32 outputHash = keccak256(output);
bytes memory calldata_buffer = new bytes(100 + proof.length);
assembly {{
    mstore(add(calldata_buffer, 32), /*selector*/)
    mstore(add(calldata_buffer, 36), {})
    mstore(add(calldata_buffer, 68), inputHash)
    mstore(add(calldata_buffer, 100), outputHash)
    calldatacopy(add(calldata_buffer, 132), add(proof, 32), calldataload(proof))
    let success := call(gas(), {}, 0, add(calldata_buffer, 32), mload(calldata_buffer), 0, 0)
    if iszero(success) {{
        revert(0, 0)
    }}
}}
return abi.decode(output, ({}))"#,
        args_names.join(", "),
        code_hash,
        EXEUCTION_ORACLE_ADDRESS,
        output_types.join(", "),
    )
}

struct Param {
    pub r#type: String,
    pub memory_location: String,
    pub name: String,
}

impl Param {
    pub fn definition(&self) -> String {
        format!("{} {} {}", self.r#type, self.memory_location, self.name)
    }
}

fn provable_function_wrapper_contract(function: &str, function_name: &str, inputs: Vec<Param>) -> String {
    format!(r#"
contract Provable {{
    fallback(bytes calldata input) external payable returns(bytes memory output) {{
        ({}) = abi.decode(input, ({}));
        return abi.encode({}({}));
    }}

    {}
}}"#,
        inputs.iter().map(|arg| arg.definition()).join(", "),
        inputs.iter().map(|arg| arg.r#type).join(", "),
        function_name,
        inputs.iter().map(|arg| arg.name).join(", "),
        function
    )
}

const EXEUCTION_ORACLE_ADDRESS: &'static str = "0xTODO";
const VERIFY_COMPUTATION_FUNCTION_SELECTOR: &'static str = "0xTODO";

fn main() {
    let mut command = std::process::Command::new("solc");
    command.stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    command.arg("--input-file");
    command.arg("/Users/antond/Desktop/free-computation/examples/Example.sol");
    command.arg("--bin");

    let mut process = command.spawn().unwrap();
    let result = process.wait_with_output().unwrap();
    println!("{:?}", String::from_utf8(result.stdout).unwrap());
}
