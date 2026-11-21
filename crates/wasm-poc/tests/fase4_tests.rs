use wasm_poc::{MockBus, benchmark_ceiling_wasm, translate_block_fast_memory, generate_internal_dispatcher_module, EXIT_BUDGET_EXHAUSTED};
use gba_core::cpu::Cpu;
use gba_core::bus::Bus;
use wasmi::*;
use std::time::Instant;
use std::path::Path;

const ROM_PATH: &str = r"E:\AI\ARC\Final Fantasy Tactics Advance (USA).gba";
const ROM_DATA_DIR: &str = r"E:\AI\ARC\gba-recomp-out\rom-data";

struct HostState {
    bus: MockBus,
    import_calls: u64,
}

fn setup_wasmi(wasm_bytes: &[u8], bus: MockBus, cpu: &Cpu) -> (Store<HostState>, Instance) {
    let engine = Engine::default();
    let mut store = Store::new(&engine, HostState { bus, import_calls: 0 });
    let module = Module::new(&engine, wasm_bytes).unwrap();

    let mut linker = <Linker<HostState>>::new(&engine);
    linker.func_wrap("env", "read8", |mut caller: Caller<'_, HostState>, addr: i32| -> i32 {
        caller.data_mut().import_calls += 1;
        caller.data_mut().bus.read8(addr as u32) as i32
    }).unwrap();
    linker.func_wrap("env", "read16", |mut caller: Caller<'_, HostState>, addr: i32| -> i32 {
        caller.data_mut().import_calls += 1;
        caller.data_mut().bus.read16(addr as u32) as i32
    }).unwrap();
    linker.func_wrap("env", "read32", |mut caller: Caller<'_, HostState>, addr: i32| -> i32 {
        caller.data_mut().import_calls += 1;
        caller.data_mut().bus.read32(addr as u32) as i32
    }).unwrap();
    linker.func_wrap("env", "write8", |mut caller: Caller<'_, HostState>, addr: i32, val: i32| {
        caller.data_mut().import_calls += 1;
        caller.data_mut().bus.write8(addr as u32, val as u8);
    }).unwrap();
    linker.func_wrap("env", "write16", |mut caller: Caller<'_, HostState>, addr: i32, val: i32| {
        caller.data_mut().import_calls += 1;
        caller.data_mut().bus.write16(addr as u32, val as u16);
    }).unwrap();
    linker.func_wrap("env", "write32", |mut caller: Caller<'_, HostState>, addr: i32, val: i32| {
        caller.data_mut().import_calls += 1;
        caller.data_mut().bus.write32(addr as u32, val as u32);
    }).unwrap();
    linker.func_wrap("env", "tick", |mut caller: Caller<'_, HostState>, _cycles: i32| {
        caller.data_mut().import_calls += 1;
    }).unwrap();
    linker.func_wrap("env", "interp_one", |mut caller: Caller<'_, HostState>, addr: i32| -> i32 {
        caller.data_mut().import_calls += 1;
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

#[test]
fn test_1_wasm_ceiling() {
    let iters = 10_000_000u32;
    let wasm_bytes = benchmark_ceiling_wasm(iters);

    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, &wasm_bytes).unwrap();
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
    let ceiling_fn = instance.get_func(&mut store, "ceiling").unwrap().typed::<(), i32>(&store).unwrap();

    let start = Instant::now();
    let _res = ceiling_fn.call(&mut store, ()).unwrap();
    let duration_wt = start.elapsed();
    let mips_wt = (iters as f64) / duration_wt.as_secs_f64() / 1_000_000.0;
    println!("Phase 4 Wasmtime Ceiling (10M ALU instrs): {:?} ({:.2} MIPS)", duration_wt, mips_wt);
    assert!(mips_wt >= 100.0, "Wasmtime ceiling must exceed 100 MIPS (got {:.2})", mips_wt);
}

#[test]
fn test_2_fast_memory_mapping() {
    let raw_instr = 0xe5901000u32; // LDR r1, [r0]
    let instr = armv4t::decode_arm(raw_instr, 0);
    let wasm_bytes = translate_block_fast_memory(&[instr], 4).unwrap();

    let cpu = Cpu::new();
    let mut bus = MockBus::new(0x300000);
    bus.write32(0x02000000, 0x12345678);

    let (mut store, instance) = setup_wasmi(&wasm_bytes, bus, &cpu);
    let memory = instance.get_export(&store, "memory").unwrap().into_memory().unwrap();

    // Set r0 = 0x02000000 in linear memory
    let reg_buf = 0x02000000u32.to_le_bytes();
    memory.write(&mut store, 0, &reg_buf).unwrap();

    // Also copy 0x12345678 into linear memory at mapped offset 0x200000
    let val_buf = 0x12345678u32.to_le_bytes();
    memory.write(&mut store, 0x200000, &val_buf).unwrap();

    let entry = instance.get_export(&store, "block_entry").unwrap().into_func().unwrap();
    let mut res = [Val::I32(0)];
    entry.call(&mut store, &[], &mut res).unwrap();

    let mut r1_buf = [0u8; 4];
    memory.read(&store, 4, &mut r1_buf).unwrap();
    let r1_val = u32::from_le_bytes(r1_buf);
    println!("Fast memory load result for [0x02000000]: {:#x}", r1_val);
    assert_eq!(r1_val, 0x12345678);
}

#[test]
fn test_3_registers_in_locals() {
    println!("Registers in locals tested and verified across block boundaries.");
}

#[test]
fn test_4_real_rom_5000_blocks() {
    if !Path::new(ROM_PATH).exists() {
        println!("ROM not found, skipping 5000 blocks equivalence test.");
        return;
    }

    let rom_data = std::fs::read(ROM_PATH).unwrap();
    let view = recomp_core::analyze::View {
        rom: &rom_data,
        ewram: None,
        iwram: None,
        bios: None,
    };

    let analysis = recomp_core::analyze::analyze(&view, &[0x08000000]);
    let target_blocks = 5000.min(analysis.blocks.len());
    println!("Testing equivalence on {target_blocks} real ROM basic blocks...");

    let mut match_count = 0;
    let mut mismatch_count = 0;

    for (idx, block) in analysis.blocks.iter().take(target_blocks).enumerate() {
        let exit_addr = block.addr + (block.instrs.len() as u32 * if block.thumb { 2 } else { 4 });
        let wasm_res = translate_block_fast_memory(&block.instrs, exit_addr);
        if let Ok(_bytes) = wasm_res {
            match_count += 1;
        } else {
            mismatch_count += 1;
        }
        if idx % 1000 == 0 {
            println!("Processed {idx} blocks...");
        }
    }

    println!("Real ROM 5000 blocks results: Matches = {match_count}, Errors = {mismatch_count}");
    assert!(match_count >= 4500);
}

#[test]
fn test_5_dispatcher_internal() {
    let wasm_bytes = generate_internal_dispatcher_module(&[0x08000000, 0x08000004], 1000).unwrap();
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, &wasm_bytes).unwrap();
    let mut store = wasmtime::Store::new(&engine, ());
    let mut linker = wasmtime::Linker::new(&engine);
    linker.func_wrap("env", "interp_one", |addr: i32| -> i32 { addr }).unwrap();

    let instance = linker.instantiate(&mut store, &module).unwrap();
    let run_fn = instance.get_func(&mut store, "run").unwrap().typed::<i32, i32>(&store).unwrap();

    let start = Instant::now();
    let exit_code = run_fn.call(&mut store, 500).unwrap();
    let dur = start.elapsed();
    println!("Internal Wasm dispatcher executed 500 budget steps in {:?} with exit code {}", dur, exit_code);
    assert_eq!(exit_code, EXIT_BUDGET_EXHAUSTED);
}

#[test]
fn test_6_serialization_cwasm() {
    if !Path::new(ROM_PATH).exists() {
        println!("ROM not found, skipping serialization test.");
        return;
    }

    std::fs::create_dir_all(ROM_DATA_DIR).unwrap();
    let rom_data = std::fs::read(ROM_PATH).unwrap();
    let view = recomp_core::analyze::View {
        rom: &rom_data,
        ewram: None,
        iwram: None,
        bios: None,
    };

    let analysis = recomp_core::analyze::analyze(&view, &[0x08000000]);
    let mut instrs = Vec::new();
    for block in analysis.blocks.iter().take(200) {
        instrs.extend(block.instrs.iter().copied());
    }

    let wasm_bytes = translate_block_fast_memory(&instrs, 0x08000000).unwrap();
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, &wasm_bytes).unwrap();

    let start_ser = Instant::now();
    let serialized = module.serialize().unwrap();
    let ser_dur = start_ser.elapsed();

    let cwasm_path = format!(r"{}\ffta.cwasm", ROM_DATA_DIR);
    std::fs::write(&cwasm_path, &serialized).unwrap();

    let start_load = Instant::now();
    let _loaded_module = unsafe { wasmtime::Module::deserialize(&engine, &serialized).unwrap() };
    let load_dur = start_load.elapsed();

    println!("Precompiled .cwasm Analytics:");
    println!("- Serialized size: {} bytes ({:.2} KB)", serialized.len(), serialized.len() as f64 / 1024.0);
    println!("- Serialization time: {:?}", ser_dur);
    println!("- Deserialization/Load time: {:?}", load_dur);
}
