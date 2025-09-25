use std::{
    fs::File,
    io::Read,
    path::Path,
};

#[derive(Debug)]
struct EmulatorState {
    program_counter: u32,
    a_register: u8,
    b_register: u8,     // Volatile (is never side effected)
    jump_register: u16, // u12?
    carry_flag: bool,

    memory: [u8; 2048], // 2 KiB
    halted: bool,
}

pub struct FlewittPCEmulator {
    state: EmulatorState,
}

impl FlewittPCEmulator {
    pub fn new_from_binary_file(path: impl AsRef<Path>) -> Self {
        let mut memory = [0; 2048]; // 2 KiB
        File::open(path)
            .expect("Unable to open file")
            .read(&mut memory)
            .expect("Unable to read file");

        let state = EmulatorState {
            memory: memory,

            program_counter: Default::default(),
            a_register: Default::default(),
            b_register: Default::default(),
            jump_register: Default::default(),
            carry_flag: Default::default(),
            halted: Default::default(),
        };

        Self { state }
    }

    pub fn run(&mut self) {
        while !self.state.halted {
            self.step();
        }
    }

    pub fn step(&mut self) {
        // Read inscrution at program counter
        let instruction = self.read_instruction().expect("Unable to read instruction");
        // Execute it, Updating program counter accordingly
        self.handle_instruction(&instruction);

        let mut _s = String::new();
        std::io::stdin().read_line(&mut _s).expect("meow");
    }

    fn handle_instruction(&mut self, instruction: &Instruction) {
        // TODO: Remove debug stuff
        dbg!(&self.state);
        dbg!(&instruction);
        match instruction {
            Instruction::nop => {
                self.state.program_counter += 1; // instruction
            },
            Instruction::add => {
                let data = self.state.a_register.overflowing_add(self.state.b_register);
                self.state.a_register = data.0;
                self.state.carry_flag = data.1;
                self.state.program_counter += 1; // instruction
            },
            Instruction::sub => {
                let data = self.state.a_register.overflowing_sub(self.state.b_register);
                self.state.a_register = data.0;
                self.state.carry_flag = data.1;
                self.state.program_counter += 1; // instruction
            },
            Instruction::load(constant) => {
                self.state.a_register = *constant;
                self.state.program_counter += 2; // instruction + 1 byte operand
            },
            Instruction::loadj(short_constant) => {
                self.state.jump_register = *short_constant;
                self.state.program_counter += 3; // instruction + 2 byte operand
            },
            Instruction::fetch => {
                let data = self.state.memory[self.state.jump_register as usize];
                self.state.a_register = data;
                self.state.program_counter += 1; // instruction
            },
            Instruction::fetchj => {
                let data = u16::from_le_bytes([
                    self.state.memory[(self.state.jump_register) as usize],
                    self.state.memory[(self.state.jump_register + 1) as usize],
                ]);
                self.state.jump_register = data;
                self.state.program_counter += 1; // instruction
            },
            Instruction::write => {
                self.state.memory[self.state.jump_register as usize] = self.state.a_register;
                self.state.program_counter += 1; // instruction
            },
            Instruction::copyab => {
                self.state.b_register = self.state.a_register;
                self.state.program_counter += 1; // instruction
            },
            Instruction::copyba => {
                self.state.a_register = self.state.b_register;
                self.state.program_counter += 1; // instruction
            },
            Instruction::jmp => {
                self.state.program_counter = self.state.jump_register as u32;
            },
            Instruction::jz => {
                if self.state.a_register == 0 {
                    self.state.program_counter = self.state.jump_register as u32;
                }
            },
            Instruction::jc => {
                if self.state.carry_flag {
                    self.state.program_counter = self.state.jump_register as u32;
                }
            },
            Instruction::halt => {
                self.state.halted = true;
                self.state.program_counter += 1; // instruction
            },
        };

    }

    pub fn read_instruction(&self) -> std::io::Result<Instruction> {
        self.read_instruction_at(self.state.program_counter)
    }

    pub fn read_instruction_at(&self, location: u32) -> std::io::Result<Instruction> {
        let instruction_byte = self.state.memory[location as usize];
        // NOTE: Make sure you handle all instructions from Instruction enum
        match instruction_byte {
            0 => Ok(Instruction::nop),
            1 => Ok(Instruction::add),
            2 => Ok(Instruction::sub),

            3 => Ok(Instruction::load(
                self.state.memory[(location + 1) as usize],
            )),
            4 => Ok(Instruction::loadj(u16::from_le_bytes([
                self.state.memory[(location + 1) as usize],
                self.state.memory[(location + 2) as usize],
            ]))),

            5 => Ok(Instruction::fetch),
            6 => Ok(Instruction::fetchj),
            7 => Ok(Instruction::write),
            8 => Ok(Instruction::copyab),
            9 => Ok(Instruction::copyba),
            10 => Ok(Instruction::jmp),
            11 => Ok(Instruction::jz),
            12 => Ok(Instruction::jc),
            /* 13 => Ok(Instruction::writed),*/
            /* 14 => Ok(Instruction::readd),*/
            255 => Ok(Instruction::halt),

            _ => Result::Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown instruction",
            )),
        }
    }
}

/*
https://github.com/Unbox101/FlewittPC_SDK/blob/583bf4439f7f248faf3d92d422915e3a7c81698a/main.c#L10
| Opcode | Byte | Operands | Cycles | Description                                                |
| :----- | :--: | :------: | :----: | ---------------------------------------------------------- |
| nop    |  0   |    -     |   ?    | No operation.                                              |
| add    |  1   |    -     |   ?    | Add registers a and b. Outputs to register a.              |
| sub    |  2   |    -     |   ?    | Sub registers a and b. Outputs to register a.              |
| load   |  3   |   byte   |   ?    | Load the next constant into register a.                    |
| loadj  |  4   |  short   |   ?    | Load the next 2 constants into jump-register.              |
| fetch  |  5   |    -     |   ?    | Load memory from jump-register address into register a.    |
| fetchj |  6   |    -     |   ?    | Load memory from jump-register address into jump-register. |
| write  |  7   |    -     |   ?    | Write memory from register a to jump-register address.     |
| copyab |  8   |    -     |   ?    | Copy register a to register b.                             |
| copyba |  9   |    -     |   ?    | Copy register b to register a.                             |
| jmp    |  10  |    -     |   ?    | Copies jump-register address into program_counter.         |
| jz     |  11  |    -     |   ?    | "jmp" if register a is zero.                               |
| jc     |  12  |    -     |   ?    | "jmp" if the register a + register b > 255.                |
| writed |  13  |    -     |   ?    | Write the next constant into the LCD display.              |
| readd  |  14  |    -     |   ?    | Copy LCD display output into register a.                   |
| halt   | 255  |    -     |   ?    | Stops the computer.                                        |
*/
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug)]
pub enum Instruction {
    nop = 0,
    add = 1,
    sub = 2,
    load(u8) = 3,
    loadj(u16) = 4,
    fetch = 5,
    fetchj = 6,
    write = 7,
    copyab = 8,
    copyba = 9,
    jmp = 10,
    jz = 11,
    jc = 12,
    /* writed(u8) = 13, */ // Unimplemented in real hardware i think...
    /* readd = 14, */ // Unimplemented in real hardware i think...
    halt = 255,
}
