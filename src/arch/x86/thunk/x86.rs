use std::mem;
use generic_array::{GenericArray, typenum};
use pic::{Thunkable, StaticThunk};

#[repr(packed)]
pub struct JumpRel {
    opcode: u8,
    operand: u32,
}

/// Constructs either a relative jump or call.
fn relative32(destination: usize, is_jump: bool) -> Box<Thunkable> {
    const CALL: u8 = 0xE8;
    const JMP: u8 = 0xE9;

    Box::new(StaticThunk::<typenum::U5>::new(move |source| {
        let code = JumpRel {
            opcode: if is_jump { JMP } else { CALL },
            operand: calculate_displacement(source, destination, mem::size_of::<JumpRel>()),
        };

        let slice: [u8; 5] = unsafe { mem::transmute(code) };
        GenericArray::clone_from_slice(&slice)
    }))
}

/// Constructs a relative call operation.
pub fn call_rel32(destination: usize) -> Box<Thunkable> {
    relative32(destination, false)
}

/// Constructs a relative jump operation.
pub fn jmp_rel32(destination: usize) -> Box<Thunkable> {
    relative32(destination, true)
}

#[repr(packed)]
pub struct JccRel {
    opcode0: u8,
    opcode1: u8,
    operand: u32,
}

/// Constructs a conditional relative jump operation.
pub fn jcc_rel32(destination: usize, condition: u8) -> Box<Thunkable> {
    Box::new(StaticThunk::<typenum::U6>::new(move |source| {
        let code = JccRel {
            opcode0: 0x0F,
            opcode1: 0x80 | condition,
            operand: calculate_displacement(source, destination, mem::size_of::<JccRel>()),
        };

        let slice: [u8; 6] = unsafe { mem::transmute(code) };
        GenericArray::clone_from_slice(&slice)
    }))
}

#[repr(packed)]
pub struct JumpShort {
    opcode: u8,
    operand: i8,
}

/// Constructs a relative short jump.
pub fn jmp_rel8(displacement: i8) -> Box<Thunkable> {
    Box::new(StaticThunk::<typenum::U2>::new(move |_| {
        let code = JumpShort {
            opcode: 0xEB,
            operand: displacement - mem::size_of::<JumpShort>() as i8,
        };

        let slice: [u8; 2] = unsafe { mem::transmute(code) };
        GenericArray::clone_from_slice(&slice)
    }))
}

/// Calculates the relative displacement for an instruction.
fn calculate_displacement(source: usize,
                          destination: usize,
                          instruction_size: usize) -> u32 {
    let displacement = (destination as isize).wrapping_sub(source as isize + instruction_size as isize);

    // Ensure that the detour can be reached with a relative jump (+/- 2GB).
    // This only needs to be asserted on x64, since it wraps around on x86.
    #[cfg(target_arch = "x86_64")]
    assert!(::arch::x86::is_within_2gb(displacement));

    displacement as u32
}
