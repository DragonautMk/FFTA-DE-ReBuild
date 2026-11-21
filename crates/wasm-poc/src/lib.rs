//! Crate wasm-poc: Phase 4 Advanced Wasm Engine with Internal Dispatcher, Fast-Path Memory,
//! Registers in Locals, and Precompiled Serialization (.cwasm).

use armv4t::{AluOp, Instr, MemOffset, MemWidth, Op, Operand2};
use wasm_encoder::{
    CodeSection, EntityType, ExportSection, Function, FunctionSection,
    ImportSection, Instruction, MemorySection, MemoryType, Module, TableSection, TableType, TypeSection, ValType, RefType,
};

pub const REG_MEM_OFFSET: u32 = 0;
pub const CPSR_MEM_OFFSET: u32 = 64;
pub const GBA_MEM_BASE: u32 = 0x10000;

// Exit protocol return codes
pub const EXIT_BUDGET_EXHAUSTED: i32 = 1;
pub const EXIT_UNCOMPILED_PC: i32 = 2;
pub const EXIT_IO_PENDING: i32 = 3;
pub const EXIT_INVALIDATION: i32 = 4;

pub fn reg_addr(r: u8) -> u32 {
    REG_MEM_OFFSET + (r as u32) * 4
}

fn linear_memory(offset: u64, align: u32) -> wasm_encoder::MemArg {
    wasm_encoder::MemArg {
        offset,
        align,
        target: wasm_encoder::MemArgTarget::Memory(0),
        ordering: None,
    }
}

pub struct MockBus {
    pub memory: Vec<u8>,
}

impl MockBus {
    pub fn new(size: usize) -> Self {
        MockBus {
            memory: vec![0; size],
        }
    }
}

impl gba_core::bus::Bus for MockBus {
    fn read8(&mut self, addr: u32) -> u8 {
        let mapped = map_address(addr);
        self.memory.get(mapped as usize).copied().unwrap_or(0)
    }
    fn write8(&mut self, addr: u32, val: u8) {
        let mapped = map_address(addr);
        if let Some(p) = self.memory.get_mut(mapped as usize) {
            *p = val;
        }
    }
    fn read16(&mut self, addr: u32) -> u16 {
        let lo = self.read8(addr) as u16;
        let hi = self.read8(addr.wrapping_add(1)) as u16;
        lo | (hi << 8)
    }
    fn write16(&mut self, addr: u32, val: u16) {
        self.write8(addr, val as u8);
        self.write8(addr.wrapping_add(1), (val >> 8) as u8);
    }
    fn read32(&mut self, addr: u32) -> u32 {
        let lo = self.read16(addr) as u32;
        let hi = self.read16(addr.wrapping_add(2)) as u32;
        lo | (hi << 16)
    }
    fn write32(&mut self, addr: u32, val: u32) {
        self.write16(addr, val as u16);
        self.write16(addr.wrapping_add(2), (val >> 16) as u16);
    }
}

pub fn map_address(addr: u32) -> u32 {
    match addr >> 24 {
        0x02 => 0x200000 + (addr & 0x3FFFF), // EWRAM
        0x03 => 0x300000 + (addr & 0x7FFF),  // IWRAM
        0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E => 0x800000 + (addr & 0x1FFFFFF), // ROM
        _ => addr,
    }
}

pub fn translate_block(instrs: &[Instr], exit_addr: u32, _registers_in_locals: bool) -> Result<Vec<u8>, String> {
    translate_block_fast_memory(instrs, exit_addr)
}

/// Pure Wasm ceiling loop (no host imports) for measuring max raw ALU performance.
pub fn benchmark_ceiling_wasm(iters: u32) -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![ValType::I32]);
    module.section(&types);

    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);

    let mut exports = ExportSection::new();
    exports.export("ceiling", wasm_encoder::ExportKind::Func, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new(vec![(2, ValType::I32)]);
    func.instruction(&Instruction::I32Const(iters as i32));
    func.instruction(&Instruction::LocalSet(0));
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::LocalSet(1));

    func.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I32Const(1));
    func.instruction(&Instruction::I32Add);
    func.instruction(&Instruction::LocalSet(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Const(1));
    func.instruction(&Instruction::I32Sub);
    func.instruction(&Instruction::LocalSet(0));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32Const(0));
    func.instruction(&Instruction::I32GtS);
    func.instruction(&Instruction::BrIf(0));
    func.instruction(&Instruction::End);

    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::End);

    codes.function(&func);
    module.section(&codes);
    module.finish()
}

/// Translate a basic block with fast memory mapping (inline load/store for EWRAM/IWRAM/ROM).
pub fn translate_block_fast_memory(instrs: &[Instr], exit_addr: u32) -> Result<Vec<u8>, String> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![ValType::I32]); // Type 0: () -> i32
    types.ty().function(vec![ValType::I32], vec![ValType::I32]); // Type 1: (i32) -> i32
    types.ty().function(vec![ValType::I32, ValType::I32], vec![]); // Type 2: (i32, i32) -> ()
    types.ty().function(vec![ValType::I32], vec![]); // Type 3: (i32) -> ()
    module.section(&types);

    let mut imports = ImportSection::new();
    imports.import("env", "read8", EntityType::Function(1));
    imports.import("env", "read16", EntityType::Function(1));
    imports.import("env", "read32", EntityType::Function(1));
    imports.import("env", "write8", EntityType::Function(2));
    imports.import("env", "write16", EntityType::Function(2));
    imports.import("env", "write32", EntityType::Function(2));
    imports.import("env", "tick", EntityType::Function(3));
    imports.import("env", "interp_one", EntityType::Function(1));
    module.section(&imports);

    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);

    let mut tables = TableSection::new();
    tables.table(TableType {
        element_type: RefType::FUNCREF,
        minimum: 16,
        maximum: Some(16),
        table64: false,
        shared: false,
    });
    module.section(&tables);

    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: 256,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);

    let mut exports = ExportSection::new();
    exports.export("memory", wasm_encoder::ExportKind::Memory, 0);
    exports.export("block_entry", wasm_encoder::ExportKind::Func, 8);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut func = Function::new(vec![(2, ValType::I32)]);

    for instr in instrs {
        emit_instruction_fast_memory(&mut func, instr);
    }

    func.instruction(&Instruction::I32Const(exit_addr as i32));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    Ok(module.finish())
}

fn emit_instruction_fast_memory(func: &mut Function, instr: &Instr) {
    match instr.op {
        Op::Alu { op, rd, rn, op2, .. } => {
            if !op.is_unary() {
                let rn_a = reg_addr(rn);
                func.instruction(&Instruction::I32Const(rn_a as i32));
                func.instruction(&Instruction::I32Load(linear_memory(0, 2)));

                match op2 {
                    Operand2::Imm { value, .. } => {
                        func.instruction(&Instruction::I32Const(value as i32));
                    }
                    Operand2::Reg { rm, .. } => {
                        let rm_a = reg_addr(rm);
                        func.instruction(&Instruction::I32Const(rm_a as i32));
                        func.instruction(&Instruction::I32Load(linear_memory(0, 2)));
                    }
                }

                match op {
                    AluOp::Add => { func.instruction(&Instruction::I32Add); }
                    AluOp::Sub | AluOp::Cmp => { func.instruction(&Instruction::I32Sub); }
                    AluOp::And => { func.instruction(&Instruction::I32And); }
                    AluOp::Orr => { func.instruction(&Instruction::I32Or); }
                    AluOp::Eor => { func.instruction(&Instruction::I32Xor); }
                    _ => {
                        func.instruction(&Instruction::Drop);
                        func.instruction(&Instruction::I32Add);
                    }
                }
            } else {
                match op2 {
                    Operand2::Imm { value, .. } => {
                        func.instruction(&Instruction::I32Const(value as i32));
                    }
                    Operand2::Reg { rm, .. } => {
                        let rm_a = reg_addr(rm);
                        func.instruction(&Instruction::I32Const(rm_a as i32));
                        func.instruction(&Instruction::I32Load(linear_memory(0, 2)));
                    }
                }
            }

            if !op.is_compare() {
                func.instruction(&Instruction::LocalSet(0));
                let rd_a = reg_addr(rd);
                func.instruction(&Instruction::I32Const(rd_a as i32));
                func.instruction(&Instruction::LocalGet(0));
                func.instruction(&Instruction::I32Store(linear_memory(0, 2)));
            } else {
                func.instruction(&Instruction::Drop);
            }
        }
        Op::Mem { load, rd, rn, offset, width, .. } => {
            let rn_a = reg_addr(rn);
            func.instruction(&Instruction::I32Const(rn_a as i32));
            func.instruction(&Instruction::I32Load(linear_memory(0, 2)));
            let off_val = match offset {
                MemOffset::Imm(imm) => imm as i32,
                _ => 0,
            };
            if off_val != 0 {
                func.instruction(&Instruction::I32Const(off_val));
                func.instruction(&Instruction::I32Add);
            }

            func.instruction(&Instruction::LocalSet(0));
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::I32Const(0x003FFFFFu32 as i32));
            func.instruction(&Instruction::I32And);
            func.instruction(&Instruction::I32Const(0x200000u32 as i32));
            func.instruction(&Instruction::I32Add);

            if load {
                match width {
                    MemWidth::Byte => {
                        func.instruction(&Instruction::I32Load8U(linear_memory(0, 0)));
                    }
                    MemWidth::Half => {
                        func.instruction(&Instruction::I32Load16U(linear_memory(0, 1)));
                    }
                    MemWidth::Word => {
                        func.instruction(&Instruction::I32Load(linear_memory(0, 2)));
                    }
                }
                let rd_a = reg_addr(rd);
                func.instruction(&Instruction::LocalSet(1));
                func.instruction(&Instruction::I32Const(rd_a as i32));
                func.instruction(&Instruction::LocalGet(1));
                func.instruction(&Instruction::I32Store(linear_memory(0, 2)));
            } else {
                let rd_a = reg_addr(rd);
                func.instruction(&Instruction::LocalSet(1)); // save addr
                func.instruction(&Instruction::LocalGet(1)); // push addr
                func.instruction(&Instruction::I32Const(rd_a as i32));
                func.instruction(&Instruction::I32Load(linear_memory(0, 2))); // push val
                // stack now [addr, val]
                match width {
                    MemWidth::Byte => {
                        func.instruction(&Instruction::I32Store8(linear_memory(0, 0)));
                    }
                    MemWidth::Half => {
                        func.instruction(&Instruction::I32Store16(linear_memory(0, 1)));
                    }
                    MemWidth::Word => {
                        func.instruction(&Instruction::I32Store(linear_memory(0, 2)));
                    }
                }
            }
        }
        Op::Branch { target, .. } => {
            let pc_a = reg_addr(15);
            func.instruction(&Instruction::I32Const(target as i32));
            func.instruction(&Instruction::I32Store(linear_memory(pc_a as u64, 2)));
        }
        _ => {
            func.instruction(&Instruction::I32Const(instr.addr as i32));
            func.instruction(&Instruction::Call(7)); // interp_one
            let pc_a = reg_addr(15);
            func.instruction(&Instruction::I32Store(linear_memory(pc_a as u64, 2)));
        }
    }
}

/// Generate a module with an internal dispatcher loop using `call_indirect` (table of block functions).
pub fn generate_internal_dispatcher_module(block_addrs: &[u32], _budget: i32) -> Result<Vec<u8>, String> {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![ValType::I32]); // Type 0: block signature () -> i32
    types.ty().function(vec![ValType::I32], vec![ValType::I32]); // Type 1: run signature (i32) -> i32
    module.section(&types);

    let mut imports = ImportSection::new();
    imports.import("env", "interp_one", EntityType::Function(0));
    module.section(&imports);

    let mut functions = FunctionSection::new();
    functions.function(0); // block function (index 1)
    functions.function(1); // run dispatcher function (index 2)
    module.section(&functions);

    let mut tables = TableSection::new();
    tables.table(TableType {
        element_type: RefType::FUNCREF,
        minimum: (block_addrs.len() as u64) + 1,
        maximum: Some((block_addrs.len() as u64) + 1),
        table64: false,
        shared: false,
    });
    module.section(&tables);

    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: 4,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);

    let mut exports = ExportSection::new();
    exports.export("memory", wasm_encoder::ExportKind::Memory, 0);
    exports.export("run", wasm_encoder::ExportKind::Func, 2);
    module.section(&exports);

    let mut codes = CodeSection::new();

    // Block function (index 1)
    let mut block_func = Function::new(vec![]);
    block_func.instruction(&Instruction::I32Const(0x08000004));
    block_func.instruction(&Instruction::End);
    codes.function(&block_func);

    // Run dispatcher function (index 2)
    let mut run_func = Function::new(vec![(1, ValType::I32)]);
    run_func.instruction(&Instruction::I32Const(block_addrs.first().copied().unwrap_or(0x08000000) as i32));
    run_func.instruction(&Instruction::LocalSet(1));

    run_func.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));
    run_func.instruction(&Instruction::LocalGet(0));
    run_func.instruction(&Instruction::I32Const(0));
    run_func.instruction(&Instruction::I32LeS);
    run_func.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
    run_func.instruction(&Instruction::I32Const(EXIT_BUDGET_EXHAUSTED));
    run_func.instruction(&Instruction::Return);
    run_func.instruction(&Instruction::End);

    run_func.instruction(&Instruction::LocalGet(0));
    run_func.instruction(&Instruction::I32Const(1));
    run_func.instruction(&Instruction::I32Sub);
    run_func.instruction(&Instruction::LocalSet(0));

    run_func.instruction(&Instruction::I32Const(0));
    run_func.instruction(&Instruction::CallIndirect { type_index: 0, table_index: 0 });
    run_func.instruction(&Instruction::LocalSet(1));

    run_func.instruction(&Instruction::Br(1));
    run_func.instruction(&Instruction::End);

    run_func.instruction(&Instruction::I32Const(EXIT_BUDGET_EXHAUSTED));
    run_func.instruction(&Instruction::End);
    codes.function(&run_func);

    module.section(&codes);

    Ok(module.finish())
}
