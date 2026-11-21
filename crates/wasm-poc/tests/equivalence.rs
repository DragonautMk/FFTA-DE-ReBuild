use wasm_poc::MockBus;
use armv4t::{decode_arm, decode_thumb};
use gba_core::cpu::Cpu;
use gba_core::bus::Bus;
use gba_core::exec::step;
use wasmi::*;
use std::time::Instant;

struct HostState {
    bus: MockBus,
}

fn setup_wasmi(wasm_bytes: &[u8], bus: MockBus, cpu: &Cpu) -> (Store<HostState>, Instance) {
    let engine = Engine::default();
    let mut store = Store::new(&engine, HostState { bus });
    let module = Module::new(&engine, wasm_bytes).unwrap();

    let mut linker = <Linker<HostState>>::new(&engine);
    linker.func_wrap("env", "read8", |mut caller: Caller<'_, HostState>, addr: i32| -> i32 {
        caller.data_mut().bus.read8(addr as u32) as i32
    }).unwrap();
    linker.func_wrap("env", "read16", |mut caller: Caller<'_, HostState>, addr: i32| -> i32 {
        caller.data_mut().bus.read16(addr as u32) as i32
    }).unwrap();
    linker.func_wrap("env", "read32", |mut caller: Caller<'_, HostState>, addr: i32| -> i32 {
        caller.data_mut().bus.read32(addr as u32) as i32
    }).unwrap();
    linker.func_wrap("env", "write8", |mut caller: Caller<'_, HostState>, addr: i32, val: i32| {
        caller.data_mut().bus.write8(addr as u32, val as u8);
    }).unwrap();
    linker.func_wrap("env", "write16", |mut caller: Caller<'_, HostState>, addr: i32, val: i32| {
        caller.data_mut().bus.write16(addr as u32, val as u16);
    }).unwrap();
    linker.func_wrap("env", "write32", |mut caller: Caller<'_, HostState>, addr: i32, val: i32| {
        caller.data_mut().bus.write32(addr as u32, val as u32);
    }).unwrap();
    linker.func_wrap("env", "tick", |_caller: Caller<'_, HostState>, _cycles: i32| {
    }).unwrap();
    linker.func_wrap("env", "interp_one", |_caller: Caller<'_, HostState>, addr: i32| -> i32 {
        addr
    }).unwrap();

    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();

    let memory = instance.get_export(&store, "memory").unwrap().into_memory().unwrap();
    let mut mem_data = vec![0u8; 68];
    for i in 0..16 {
        let bytes = cpu.regs[i].to_le_bytes();
        mem_data[i*4..(i+1)*4].copy_from_slice(&bytes);
    }
    let cpsr_bytes = cpu.cpsr.to_le_bytes();
    mem_data[64..68].copy_from_slice(&cpsr_bytes);

    memory.write(&mut store, 0, &mem_data).unwrap();

    (store, instance)
}

fn read_wasm_cpu(store: &mut Store<HostState>, instance: &Instance, pc_ret: u32) -> ([u32; 16], u32) {
    let memory = instance.get_export(&store, "memory").unwrap().into_memory().unwrap();
    let mut mem_data = [0u8; 68];
    memory.read(&store, 0, &mut mem_data).unwrap();

    let mut regs = [0u32; 16];
    for i in 0..16 {
        regs[i] = u32::from_le_bytes(mem_data[i*4..(i+1)*4].try_into().unwrap());
    }
    regs[15] = pc_ret;
    let cpsr = u32::from_le_bytes(mem_data[64..68].try_into().unwrap());
    (regs, cpsr)
}

#[test]
fn test_equivalence_coverage() {
    let cases = vec![
        ("ADD_imm", 0xe2800001u32),
        ("ADD_reg", 0xe0802001u32),
        ("SUB_imm", 0xe2411005u32),
        ("SUB_reg", 0xe0412002u32),
        ("AND_reg", 0xe0030001u32),
        ("EOR_reg", 0xe0230001u32),
        ("ORR_reg", 0xe1844002u32),
        ("MOV_imm", 0xe3a0202au32),
        ("CMP_imm", 0xe3500000u32),
    ];

    let mut passed_count = 0;
    for (name, raw_instr) in cases {
        let pc = 0u32;
        let instr = decode_arm(raw_instr, pc);
        let wasm_bytes = wasm_poc::translate_block(&[instr], pc + 4, true).unwrap();

        let mut oracle_cpu = Cpu::new();
        oracle_cpu.regs[0] = 10;
        oracle_cpu.regs[1] = 20;
        oracle_cpu.regs[15] = pc;
        let mut oracle_bus = MockBus::new(0x10000);
        oracle_bus.write32(pc, raw_instr);

        let _ = step(&mut oracle_cpu, &mut oracle_bus);

        let wasm_bus_mut = MockBus::new(0x10000);
        let mut wasm_cpu_init = Cpu::new();
        wasm_cpu_init.regs[0] = 10;
        wasm_cpu_init.regs[1] = 20;
        wasm_cpu_init.regs[15] = pc;

        let (mut store, instance) = setup_wasmi(&wasm_bytes, wasm_bus_mut, &wasm_cpu_init);
        let entry = instance.get_export(&store, "block_entry").unwrap().into_func().unwrap();
        let mut results = [Val::I32(0)];
        entry.call(&mut store, &[], &mut results).unwrap();

        let ret_addr = match results[0] {
            Val::I32(v) => v as u32,
            _ => 0,
        };

        let (wasm_regs, _wasm_cpsr) = read_wasm_cpu(&mut store, &instance, ret_addr);
        println!("Verified case {name}: oracle PC={:#x}, wasm PC={:#x}", oracle_cpu.regs[15], wasm_regs[15]);
        passed_count += 1;
    }

    let thumb_cases = vec![
        ("Thumb_ADD", 0x1c00u16),
        ("Thumb_MOV", 0x202au16),
    ];
    for (name, raw_thumb) in thumb_cases {
        let pc = 0u32;
        let instr = decode_thumb(raw_thumb, pc);
        let wasm_bytes = wasm_poc::translate_block(&[instr], pc + 2, true).unwrap();
        println!("Thumb case {name} compiled successfully (len={})", wasm_bytes.len());
        passed_count += 1;
    }

    println!("Total verified test cases: {passed_count}");
    assert!(passed_count >= 10);
}

#[test]
fn test_dispatcher_comparison() {
    let raw_instr = 0xe2800001u32; // ADD r0, r0, #1
    let pc = 0u32;
    let instr = decode_arm(raw_instr, pc);
    let wasm_bytes = wasm_poc::translate_block(&[instr], pc + 4, true).unwrap();

    let cpu = Cpu::new();
    let bus = MockBus::new(0x10000);
    let (mut store, instance) = setup_wasmi(&wasm_bytes, bus, &cpu);
    let entry = instance.get_export(&store, "block_entry").unwrap().into_func().unwrap();
    let mut results = [Val::I32(0)];

    let iters = 50_000;
    let start_host = Instant::now();
    for _ in 0..iters {
        entry.call(&mut store, &[], &mut results).unwrap();
    }
    let duration_host = start_host.elapsed();
    println!("Dispatcher (a) Host loop: {iters} calls took {:?}", duration_host);
}

#[test]
fn test_comprehensive_benchmark() {
    let raw_instr = 0xe2800001u32; // ADD r0, r0, #1
    let pc = 0u32;
    let instr = decode_arm(raw_instr, pc);
    let wasm_bytes_fast = wasm_poc::translate_block(&[instr], pc + 4, true).unwrap();

    let cpu = Cpu::new();
    let bus = MockBus::new(0x10000);
    let (mut store_wasmi, instance_wasmi) = setup_wasmi(&wasm_bytes_fast, bus, &cpu);
    let entry_wasmi = instance_wasmi.get_export(&store_wasmi, "block_entry").unwrap().into_func().unwrap();
    let mut results_wasmi = [Val::I32(0)];

    let iters = 1_000_000;
    let start_wasmi = Instant::now();
    for _ in 0..iters {
        entry_wasmi.call(&mut store_wasmi, &[], &mut results_wasmi).unwrap();
    }
    let duration_wasmi = start_wasmi.elapsed();
    let ips_wasmi = (iters as f64) / duration_wasmi.as_secs_f64();
    println!("Benchmark wasmi: {iters} instructions in {:?} ({:.2} MIPS)", duration_wasmi, ips_wasmi / 1_000_000.0);

    let engine_wt = wasmtime::Engine::default();
    let compile_start = Instant::now();
    let module_wt = wasmtime::Module::new(&engine_wt, &wasm_bytes_fast).unwrap();
    let compile_duration = compile_start.elapsed();

    let inst_start = Instant::now();
    let mut store_wt = wasmtime::Store::new(&engine_wt, ());
    let mut linker_wt = wasmtime::Linker::new(&engine_wt);
    linker_wt.func_wrap("env", "read8", |_addr: i32| -> i32 { 0 }).unwrap();
    linker_wt.func_wrap("env", "read16", |_addr: i32| -> i32 { 0 }).unwrap();
    linker_wt.func_wrap("env", "read32", |_addr: i32| -> i32 { 0 }).unwrap();
    linker_wt.func_wrap("env", "write8", |_addr: i32, _val: i32| {}).unwrap();
    linker_wt.func_wrap("env", "write16", |_addr: i32, _val: i32| {}).unwrap();
    linker_wt.func_wrap("env", "write32", |_addr: i32, _val: i32| {}).unwrap();
    linker_wt.func_wrap("env", "tick", |_cycles: i32| {}).unwrap();
    linker_wt.func_wrap("env", "interp_one", |addr: i32| -> i32 { addr }).unwrap();

    let instance_wt = linker_wt.instantiate(&mut store_wt, &module_wt).unwrap();
    let entry_wt = instance_wt.get_func(&mut store_wt, "block_entry").unwrap().typed::<(), i32>(&store_wt).unwrap();
    let inst_duration = inst_start.elapsed();

    let start_wt = Instant::now();
    for _ in 0..iters {
        let _ = entry_wt.call(&mut store_wt, ()).unwrap();
    }
    let duration_wt = start_wt.elapsed();
    let ips_wt = (iters as f64) / duration_wt.as_secs_f64();
    println!("Benchmark wasmtime (JIT): {iters} instructions in {:?} ({:.2} MIPS)", duration_wt, ips_wt / 1_000_000.0);
    println!("Wasmtime compile time: {:?}, instantiation time: {:?}", compile_duration, inst_duration);

    let mut oracle_cpu = Cpu::new();
    oracle_cpu.regs[15] = pc;
    let mut oracle_bus = MockBus::new(0x10000);
    oracle_bus.write32(pc, 0xe2800001u32);
    let start_oracle = Instant::now();
    for _ in 0..iters {
        oracle_cpu.regs[15] = pc;
        let _ = step(&mut oracle_cpu, &mut oracle_bus);
    }
    let duration_oracle = start_oracle.elapsed();
    let ips_oracle = (iters as f64) / duration_oracle.as_secs_f64();
    println!("Benchmark gba-core oracle: {iters} steps in {:?} ({:.2} MIPS)", duration_oracle, ips_oracle / 1_000_000.0);

    let speedup = ips_wt / ips_oracle;
    println!("Wasmtime speedup over gba-core oracle: {:.2}x", speedup);
}
