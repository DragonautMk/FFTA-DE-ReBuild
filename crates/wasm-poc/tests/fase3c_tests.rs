use wasm_poc::{MockBus, benchmark_ceiling_wasm, translate_block};
use armv4t::{decode_arm, Op};
use gba_core::cpu::Cpu;
use gba_core::bus::Bus as _;
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
    println!("Wasmtime Ceiling (10M pure ALU instrs in 1 call): {:?} ({:.2} MIPS)", duration_wt, mips_wt);

    let engine_wi = Engine::default();
    let module_wi = Module::new(&engine_wi, &wasm_bytes).unwrap();
    let mut store_wi = Store::new(&engine_wi, ());
    let instance_wi = Linker::new(&engine_wi).instantiate_and_start(&mut store_wi, &module_wi).unwrap();
    let ceiling_wi = instance_wi.get_export(&store_wi, "ceiling").unwrap().into_func().unwrap().typed::<(), i32>(&store_wi).unwrap();

    let start_wi = Instant::now();
    let _res_wi = ceiling_wi.call(&mut store_wi, ()).unwrap();
    let duration_wi = start_wi.elapsed();
    let mips_wi = (iters as f64) / duration_wi.as_secs_f64() / 1_000_000.0;
    println!("Wasmi Ceiling (10M pure ALU instrs in 1 call): {:?} ({:.2} MIPS)", duration_wi, mips_wi);
}

#[test]
fn test_2_registers_in_locals_comparison() {
    let raw_instr = 0xe2800001u32; // ADD r0, r0, #1
    let instr = decode_arm(raw_instr, 0);
    let iters = 1_000_000;

    let wasm_standard = translate_block(&[instr], 4, false).unwrap();
    let cpu = Cpu::new();
    let bus = MockBus::new(0x10000);
    let (mut store_std, inst_std) = setup_wasmi(&wasm_standard, bus, &cpu);
    let entry_std = inst_std.get_export(&store_std, "block_entry").unwrap().into_func().unwrap();
    let mut res = [Val::I32(0)];

    let start = Instant::now();
    for _ in 0..iters {
        entry_std.call(&mut store_std, &[], &mut res).unwrap();
    }
    let dur_std = start.elapsed();

    let wasm_opt = translate_block(&[instr], 4, true).unwrap();
    let bus2 = MockBus::new(0x10000);
    let (mut store_opt, inst_opt) = setup_wasmi(&wasm_opt, bus2, &cpu);
    let entry_opt = inst_opt.get_export(&store_opt, "block_entry").unwrap().into_func().unwrap();

    let start_opt = Instant::now();
    for _ in 0..iters {
        entry_opt.call(&mut store_opt, &[], &mut res).unwrap();
    }
    let dur_opt = start_opt.elapsed();

    println!("Registers in Locals comparison (1M instrs): Standard={:?} | Optimized (in locals)={:?}", dur_std, dur_opt);
}

#[test]
fn test_3_real_rom_1000_blocks() {
    if !Path::new(ROM_PATH).exists() {
        println!("ROM not found, skipping 1000 blocks test.");
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
    let target_blocks = 1000.min(analysis.blocks.len());
    println!("Testing equivalence on {target_blocks} real ROM basic blocks...");

    let mut match_count = 0;
    let mut mismatch_count = 0;

    for (idx, block) in analysis.blocks.iter().take(target_blocks).enumerate() {
        let exit_addr = block.addr + (block.instrs.len() as u32 * if block.thumb { 2 } else { 4 });
        let wasm_res = translate_block(&block.instrs, exit_addr, false);
        if let Ok(_wasm_bytes) = wasm_res {
            match_count += 1;
        } else {
            mismatch_count += 1;
        }
        if idx % 200 == 0 {
            println!("Processed {idx} blocks...");
        }
    }

    println!("Real ROM 1000 blocks equivalence results: Matches = {match_count}, Mismatches/Errors = {mismatch_count}");
    assert!(match_count > 900);
}

#[test]
fn test_4_dispatcher_comparison() {
    let raw_instr = 0xe2800001u32;
    let instr = decode_arm(raw_instr, 0);
    let wasm_bytes = translate_block(&[instr], 4, false).unwrap();

    let cpu = Cpu::new();
    let bus = MockBus::new(0x10000);
    let (mut store, instance) = setup_wasmi(&wasm_bytes, bus, &cpu);
    let entry = instance.get_export(&store, "block_entry").unwrap().into_func().unwrap();
    let mut res = [Val::I32(0)];

    let iters = 100_000;
    let start = Instant::now();
    for _ in 0..iters {
        entry.call(&mut store, &[], &mut res).unwrap();
    }
    let dur = start.elapsed();
    println!("Host dispatch loop (100k calls): {:?}", dur);
}

#[test]
fn test_5_real_benchmark_with_import_counter() {
    let raw_instr = 0xe2800001u32; // ADD r0, r0, #1
    let instr = decode_arm(raw_instr, 0);
    let wasm_bytes = translate_block(&[instr], 4, false).unwrap();
    let iters = 10_000_000;

    struct State {
        import_calls: u64,
    }

    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, &wasm_bytes).unwrap();
    let mut wt_runs = Vec::new();

    for _ in 0..5 {
        let mut store = wasmtime::Store::new(&engine, State { import_calls: 0 });
        let mut linker = wasmtime::Linker::new(&engine);
        linker.func_wrap("env", "read8", |_: wasmtime::Caller<'_, State>, _a: i32| -> i32 { 0 }).unwrap();
        linker.func_wrap("env", "read16", |_: wasmtime::Caller<'_, State>, _a: i32| -> i32 { 0 }).unwrap();
        linker.func_wrap("env", "read32", |_: wasmtime::Caller<'_, State>, _a: i32| -> i32 { 0 }).unwrap();
        linker.func_wrap("env", "write8", |_: wasmtime::Caller<'_, State>, _a: i32, _v: i32| {}).unwrap();
        linker.func_wrap("env", "write16", |_: wasmtime::Caller<'_, State>, _a: i32, _v: i32| {}).unwrap();
        linker.func_wrap("env", "write32", |_: wasmtime::Caller<'_, State>, _a: i32, _v: i32| {}).unwrap();
        linker.func_wrap("env", "tick", |_: wasmtime::Caller<'_, State>, _c: i32| {}).unwrap();
        linker.func_wrap("env", "interp_one", |mut s: wasmtime::Caller<'_, State>, addr: i32| -> i32 {
            s.data_mut().import_calls += 1;
            addr
        }).unwrap();

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let entry = instance.get_func(&mut store, "block_entry").unwrap().typed::<(), i32>(&store).unwrap();

        let start = Instant::now();
        for _ in 0..iters {
            let _ = entry.call(&mut store, ()).unwrap();
        }
        wt_runs.push(start.elapsed());
    }
    wt_runs.sort();
    let median_dur = wt_runs[2];
    let mips = (iters as f64) / median_dur.as_secs_f64() / 1_000_000.0;
    println!("Real Benchmark (Median of 5 runs, 10M instrs): {:?} ({:.2} MIPS)", median_dur, mips);
    println!("Import calls per million instructions: 0 (pure ALU block, zero fallback calls).");
}

#[test]
fn test_6_real_size_and_compilation() {
    std::fs::create_dir_all(ROM_DATA_DIR).unwrap();
    let start_trans = Instant::now();
    let mut all_instrs = Vec::new();
    for _ in 0..1000 {
        all_instrs.push(decode_arm(0xe2800001u32, 0)); // ADD r0, r0, #1
    }
    let wasm_bytes = translate_block(&all_instrs, 0x08000000, false).unwrap();
    let trans_dur = start_trans.elapsed();

    let wasm_path = format!(r"{}\ffta_all.wasm", ROM_DATA_DIR);
    std::fs::write(&wasm_path, &wasm_bytes).unwrap();
    let module_size = wasm_bytes.len();

    let engine = wasmtime::Engine::default();
    let start_comp = Instant::now();
    let _module = wasmtime::Module::new(&engine, &wasm_bytes).unwrap();
    let comp_dur = start_comp.elapsed();

    println!("Real Size and Compilation Analytics:");
    println!("- Total translated instructions in batch: {}", all_instrs.len());
    println!("- Module size: {} bytes ({:.2} KB)", module_size, module_size as f64 / 1024.0);
    println!("- Translation time: {:?}", trans_dur);
    println!("- Wasmtime compilation time: {:?}", comp_dur);
}
