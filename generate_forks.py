#!/usr/bin/env python3

"""
Fork file generator for EOT
Run with: python3 generate_forks.py
"""

import os
import csv
from io import StringIO

def main():
    print("🚀 Generating EOT fork files...\n")
    
    # Create output directory
    os.makedirs("src/forks", exist_ok=True)
    
    # Generate all forks
    generate_frontier()
    generate_homestead()
    generate_byzantium() 
    generate_constantinople()
    generate_istanbul()
    generate_berlin()
    generate_london()
    generate_shanghai()
    generate_cancun()
    generate_forks_mod()
    
    print("✅ All fork files generated successfully!")

def get_frontier_opcodes():
    """Get all Frontier opcodes in CSV format"""
    return """opcode,name,gas,inputs,outputs,description,group,introduced_in,eip
0x00,STOP,0,0,0,Halts execution,StopArithmetic,Frontier,
0x01,ADD,3,2,1,Addition operation,StopArithmetic,Frontier,
0x02,MUL,5,2,1,Multiplication operation,StopArithmetic,Frontier,
0x03,SUB,3,2,1,Subtraction operation,StopArithmetic,Frontier,
0x04,DIV,5,2,1,Integer division operation,StopArithmetic,Frontier,
0x05,SDIV,5,2,1,Signed integer division operation,StopArithmetic,Frontier,
0x06,MOD,5,2,1,Modulo remainder operation,StopArithmetic,Frontier,
0x07,SMOD,5,2,1,Signed modulo remainder operation,StopArithmetic,Frontier,
0x08,ADDMOD,8,3,1,Modulo addition operation,StopArithmetic,Frontier,
0x09,MULMOD,8,3,1,Modulo multiplication operation,StopArithmetic,Frontier,
0x0a,EXP,10,2,1,Exponential operation,StopArithmetic,Frontier,
0x0b,SIGNEXTEND,5,2,1,Extend length of two's complement signed integer,StopArithmetic,Frontier,
0x10,LT,3,2,1,Less-than comparison,ComparisonBitwiseLogic,Frontier,
0x11,GT,3,2,1,Greater-than comparison,ComparisonBitwiseLogic,Frontier,
0x12,SLT,3,2,1,Signed less-than comparison,ComparisonBitwiseLogic,Frontier,
0x13,SGT,3,2,1,Signed greater-than comparison,ComparisonBitwiseLogic,Frontier,
0x14,EQ,3,2,1,Equality comparison,ComparisonBitwiseLogic,Frontier,
0x15,ISZERO,3,1,1,Simple not operator,ComparisonBitwiseLogic,Frontier,
0x16,AND,3,2,1,Bitwise AND operation,ComparisonBitwiseLogic,Frontier,
0x17,OR,3,2,1,Bitwise OR operation,ComparisonBitwiseLogic,Frontier,
0x18,XOR,3,2,1,Bitwise XOR operation,ComparisonBitwiseLogic,Frontier,
0x19,NOT,3,1,1,Bitwise NOT operation,ComparisonBitwiseLogic,Frontier,
0x1a,BYTE,3,2,1,Retrieve single byte from word,ComparisonBitwiseLogic,Frontier,
0x20,KECCAK256,30,2,1,Compute Keccak-256 hash,Sha3,Frontier,
0x30,ADDRESS,2,0,1,Get address of currently executing account,EnvironmentalInformation,Frontier,
0x31,BALANCE,20,1,1,Get balance of the given account,EnvironmentalInformation,Frontier,
0x32,ORIGIN,2,0,1,Get execution origination address,EnvironmentalInformation,Frontier,
0x33,CALLER,2,0,1,Get caller address,EnvironmentalInformation,Frontier,
0x34,CALLVALUE,2,0,1,Get deposited value by instruction/transaction,EnvironmentalInformation,Frontier,
0x35,CALLDATALOAD,3,1,1,Get input data of current environment,EnvironmentalInformation,Frontier,
0x36,CALLDATASIZE,2,0,1,Get size of input data in current environment,EnvironmentalInformation,Frontier,
0x37,CALLDATACOPY,3,3,0,Copy input data in current environment to memory,EnvironmentalInformation,Frontier,
0x38,CODESIZE,2,0,1,Get size of code running in current environment,EnvironmentalInformation,Frontier,
0x39,CODECOPY,3,3,0,Copy code running in current environment to memory,EnvironmentalInformation,Frontier,
0x3a,GASPRICE,2,0,1,Get price of gas in current environment,EnvironmentalInformation,Frontier,
0x3b,EXTCODESIZE,20,1,1,Get size of an account's code,EnvironmentalInformation,Frontier,
0x3c,EXTCODECOPY,20,4,0,Copy an account's code to memory,EnvironmentalInformation,Frontier,
0x40,BLOCKHASH,20,1,1,Get hash of one of the 256 most recent complete blocks,BlockInformation,Frontier,
0x41,COINBASE,2,0,1,Get the block's beneficiary address,BlockInformation,Frontier,
0x42,TIMESTAMP,2,0,1,Get the block's timestamp,BlockInformation,Frontier,
0x43,NUMBER,2,0,1,Get the block's number,BlockInformation,Frontier,
0x44,DIFFICULTY,2,0,1,Get the block's difficulty,BlockInformation,Frontier,
0x45,GASLIMIT,2,0,1,Get the block's gas limit,BlockInformation,Frontier,
0x50,POP,2,1,0,Remove item from stack,StackMemoryStorageFlow,Frontier,
0x51,MLOAD,3,1,1,Load word from memory,StackMemoryStorageFlow,Frontier,
0x52,MSTORE,3,2,0,Save word to memory,StackMemoryStorageFlow,Frontier,
0x53,MSTORE8,3,2,0,Save byte to memory,StackMemoryStorageFlow,Frontier,
0x54,SLOAD,50,1,1,Load word from storage,StackMemoryStorageFlow,Frontier,
0x55,SSTORE,0,2,0,Save word to storage,StackMemoryStorageFlow,Frontier,
0x56,JUMP,8,1,0,Alter the program counter,StackMemoryStorageFlow,Frontier,
0x57,JUMPI,10,2,0,Conditionally alter the program counter,StackMemoryStorageFlow,Frontier,
0x58,PC,2,0,1,Get the value of the program counter prior to increment,StackMemoryStorageFlow,Frontier,
0x59,MSIZE,2,0,1,Get the size of active memory in bytes,StackMemoryStorageFlow,Frontier,
0x5a,GAS,2,0,1,Get the amount of available gas,StackMemoryStorageFlow,Frontier,
0x5b,JUMPDEST,1,0,0,Mark a valid destination for jumps,StackMemoryStorageFlow,Frontier,"""

def get_push_opcodes():
    """Generate PUSH1-PUSH32 opcodes"""
    opcodes = []
    for i in range(1, 33):
        opcodes.append(f"0x{0x5f + i:02x},PUSH{i},3,0,1,Place {i}-byte item on stack,Push,Frontier,")
    return "\n".join(opcodes)

def get_dup_opcodes():
    """Generate DUP1-DUP16 opcodes"""
    opcodes = []
    for i in range(1, 17):
        ordinal = {1: "1st", 2: "2nd", 3: "3rd"}.get(i, f"{i}th")
        opcodes.append(f"0x{0x7f + i:02x},DUP{i},3,{i},{i+1},Duplicate {ordinal} stack item,Duplication,Frontier,")
    return "\n".join(opcodes)

def get_swap_opcodes():
    """Generate SWAP1-SWAP16 opcodes"""
    opcodes = []
    for i in range(1, 17):
        ordinal = {1: "2nd", 2: "3rd", 3: "4th"}.get(i+1, f"{i+1}th")
        opcodes.append(f"0x{0x8f + i:02x},SWAP{i},3,{i+1},{i+1},Exchange 1st and {ordinal} stack items,Exchange,Frontier,")
    return "\n".join(opcodes)

def get_log_opcodes():
    """Generate LOG0-LOG4 opcodes"""
    opcodes = []
    for i in range(5):
        topics = "no" if i == 0 else str(i)
        opcodes.append(f"0x{0xa0 + i:02x},LOG{i},{375 + i * 375},{i + 2},0,Append log record with {topics} topics,Logging,Frontier,")
    return "\n".join(opcodes)

def get_system_opcodes():
    """Get system opcodes"""
    return """0xf0,CREATE,32000,3,1,Create a new account with associated code,System,Frontier,
0xf1,CALL,100,7,1,Message-call into an account,System,Frontier,
0xf2,CALLCODE,100,7,1,Message-call with alternative account's code,System,Frontier,
0xf3,RETURN,0,2,0,Halt execution returning output data,System,Frontier,
0xfe,INVALID,0,0,0,Designated invalid instruction,System,Frontier,
0xff,SELFDESTRUCT,5000,1,0,Halt execution and register account for deletion,System,Frontier,"""

def get_historical_additions():
    """Get opcodes added in later forks"""
    return {
        'homestead': "0xf4,DELEGATECALL,40,6,1,Message-call with alternative account's code persisting current context,System,Homestead,",
        'byzantium': """0x3d,RETURNDATASIZE,2,0,1,Get size of output data from previous call,EnvironmentalInformation,Byzantium,211
0x3e,RETURNDATACOPY,3,3,0,Copy output data from previous call to memory,EnvironmentalInformation,Byzantium,211
0xfa,STATICCALL,40,6,1,Static message-call into an account,System,Byzantium,214
0xfd,REVERT,0,2,0,Stop execution and revert state changes,System,Byzantium,140""",
        'constantinople': """0x1b,SHL,3,2,1,Left shift operation,ComparisonBitwiseLogic,Constantinople,145
0x1c,SHR,3,2,1,Logical right shift operation,ComparisonBitwiseLogic,Constantinople,145
0x1d,SAR,3,2,1,Arithmetic right shift operation,ComparisonBitwiseLogic,Constantinople,145
0x3f,EXTCODEHASH,100,1,1,Get hash of an account's code,EnvironmentalInformation,Constantinople,1052
0xf5,CREATE2,32000,4,1,Create account with associated code at specified address,System,Constantinople,1014""",
        'istanbul': """0x46,CHAINID,2,0,1,Get the chain ID,BlockInformation,Istanbul,1344
0x47,SELFBALANCE,5,0,1,Get balance of currently executing account,BlockInformation,Istanbul,1884""",
        'london': "0x48,BASEFEE,2,0,1,Get the base fee,BlockInformation,London,3198",
        'shanghai': "0x5f,PUSH0,2,0,1,Place 0 byte item on stack,Push,Shanghai,3855",
        'cancun': """0x49,BLOBHASH,3,1,1,Get versioned hash at index,BlockInformation,Cancun,4844
0x4a,BLOBBASEFEE,2,0,1,Get the current blob base fee,BlockInformation,Cancun,7516
0x5c,TLOAD,100,1,1,Load word from transient storage,StackMemoryStorageFlow,Cancun,1153
0x5d,TSTORE,100,2,0,Save word to transient storage,StackMemoryStorageFlow,Cancun,1153
0x5e,MCOPY,3,3,0,Copy memory areas,StackMemoryStorageFlow,Cancun,5656"""
    }

def generate_fork_file(fork_name, csv_data, base_fork=True):
    """Generate a fork file from CSV data"""
    
    header = f"""//! {fork_name} fork opcodes

use crate::{{opcodes, OpCode}};

opcodes! {{
    /// {fork_name} fork opcodes
    {fork_name} => {fork_name} {{"""
    
    footer = """    }
}"""
    
    opcodes_section = ""
    reader = csv.DictReader(StringIO(csv_data))
    
    for row in reader:
        opcode = row['opcode']
        name = row['name']
        gas = row['gas']
        inputs = row['inputs']
        outputs = row['outputs']
        description = row['description']
        group = row['group']
        introduced_in = row.get('introduced_in', 'Frontier')
        eip = row.get('eip', '')
        
        eip_value = f"Some({eip})" if eip else "None"
        
        opcodes_section += f"""        {opcode} => {name} {{
            gas: {gas},
            inputs: {inputs},
            outputs: {outputs},
            description: "{description}",
            introduced_in: {introduced_in},
            group: {group},
            eip: {eip_value},
            gas_history: [],
        }},
"""
    
    return header + "\n" + opcodes_section + footer

def combine_csvs(*csvs):
    """Combine multiple CSV strings"""
    combined = "opcode,name,gas,inputs,outputs,description,group,introduced_in,eip\n"
    for csv_data in csvs:
        if csv_data.strip():
            lines = csv_data.strip().split('\n')
            # Skip header if it exists
            start_idx = 1 if lines[0].startswith('opcode,') else 0
            for line in lines[start_idx:]:
                if line.strip():
                    combined += line + "\n"
    return combined

def generate_frontier():
    print("📝 Generating Frontier...")
    
    # Combine all base opcodes
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(), 
        get_swap_opcodes(),
        get_log_opcodes(),
        get_system_opcodes()
    )
    
    content = generate_fork_file("Frontier", csv_data)
    with open("src/forks/frontier.rs", "w") as f:
        f.write(content)

def generate_homestead():
    print("📝 Generating Homestead...")
    additions = get_historical_additions()
    
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(),
        get_swap_opcodes(), 
        get_log_opcodes(),
        get_system_opcodes(),
        additions['homestead']
    )
    
    content = generate_fork_file("Homestead", csv_data)
    with open("src/forks/homestead.rs", "w") as f:
        f.write(content)

def generate_byzantium():
    print("📝 Generating Byzantium...")
    additions = get_historical_additions()
    
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(),
        get_swap_opcodes(),
        get_log_opcodes(), 
        get_system_opcodes(),
        additions['homestead'],
        additions['byzantium']
    )
    
    content = generate_fork_file("Byzantium", csv_data)
    with open("src/forks/byzantium.rs", "w") as f:
        f.write(content)

def generate_constantinople():
    print("📝 Generating Constantinople...")
    additions = get_historical_additions()
    
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(),
        get_swap_opcodes(),
        get_log_opcodes(),
        get_system_opcodes(),
        additions['homestead'],
        additions['byzantium'],
        additions['constantinople']
    )
    
    content = generate_fork_file("Constantinople", csv_data)
    with open("src/forks/constantinople.rs", "w") as f:
        f.write(content)

def generate_istanbul():
    print("📝 Generating Istanbul...")
    additions = get_historical_additions()
    
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(),
        get_swap_opcodes(),
        get_log_opcodes(),
        get_system_opcodes(),
        additions['homestead'],
        additions['byzantium'],
        additions['constantinople'],
        additions['istanbul']
    )
    
    content = generate_fork_file("Istanbul", csv_data)
    with open("src/forks/istanbul.rs", "w") as f:
        f.write(content)

def generate_berlin():
    print("📝 Generating Berlin...")
    # Berlin has no new opcodes, just gas changes
    additions = get_historical_additions()
    
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(),
        get_swap_opcodes(),
        get_log_opcodes(),
        get_system_opcodes(),
        additions['homestead'],
        additions['byzantium'],
        additions['constantinople'],
        additions['istanbul']
    )
    
    content = generate_fork_file("Berlin", csv_data)
    with open("src/forks/berlin.rs", "w") as f:
        f.write(content)

def generate_london():
    print("📝 Generating London...")
    additions = get_historical_additions()
    
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(),
        get_swap_opcodes(),
        get_log_opcodes(),
        get_system_opcodes(),
        additions['homestead'],
        additions['byzantium'],
        additions['constantinople'],
        additions['istanbul'],
        additions['london']
    )
    
    content = generate_fork_file("London", csv_data)
    with open("src/forks/london.rs", "w") as f:
        f.write(content)

def generate_shanghai():
    print("📝 Generating Shanghai...")
    additions = get_historical_additions()
    
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(),
        get_swap_opcodes(),
        get_log_opcodes(),
        get_system_opcodes(),
        additions['homestead'],
        additions['byzantium'],
        additions['constantinople'],
        additions['istanbul'],
        additions['london'],
        additions['shanghai']
    )
    
    content = generate_fork_file("Shanghai", csv_data)
    with open("src/forks/shanghai.rs", "w") as f:
        f.write(content)

def generate_cancun():
    print("📝 Generating Cancun...")
    additions = get_historical_additions()
    
    csv_data = combine_csvs(
        get_frontier_opcodes(),
        get_push_opcodes(),
        get_dup_opcodes(),
        get_swap_opcodes(),
        get_log_opcodes(),
        get_system_opcodes(),
        additions['homestead'],
        additions['byzantium'],
        additions['constantinople'],
        additions['istanbul'],
        additions['london'],
        additions['shanghai'],
        additions['cancun']
    )
    
    content = generate_fork_file("Cancun", csv_data)
    with open("src/forks/cancun.rs", "w") as f:
        f.write(content)

def generate_forks_mod():
    print("📝 Generating forks/mod.rs...")
    
    content = """//! Fork-specific opcode implementations

pub mod frontier;
pub mod homestead;
pub mod byzantium;
pub mod constantinople;
pub mod istanbul;
pub mod berlin;
pub mod london;
pub mod shanghai;
pub mod cancun;

// Re-export all fork types for convenience
pub use frontier::Frontier;
pub use homestead::Homestead;
pub use byzantium::Byzantium;
pub use constantinople::Constantinople;
pub use istanbul::Istanbul;
pub use berlin::Berlin;
pub use london::London;
pub use shanghai::Shanghai;
pub use cancun::Cancun;
"""
    
    with open("src/forks/mod.rs", "w") as f:
        f.write(content)

if __name__ == "__main__":
    main()