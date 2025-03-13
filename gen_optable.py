import json
import sys

with open(sys.argv[1]) as f:
    j = json.load(f)

    j = j["unprefixed"]

    for opcode in j:
        strr = f"""{opcode}u16 => Instruction {{ 
    name: \"{j[opcode]["mnemonic"]}\",
    addr_mode: AddrMode::R_R,
    func: ,
"""
        for i, o in enumerate(j[opcode]["operands"]):
            n = f'Some(Register::{o["name"]})' if o["name"].isupper() else "None"
            strr += f"""    reg{i+1}: {n},
"""

        strr += """    ..Instruction::default()
},"""
        print(strr)