//! Functional guest-clock checks; no host-time or performance assertions.
use crate::Machine;
#[test]
fn atlas_percentages_advance_complete_guest_frames_without_clock_changes() {
    let rom=0xeaff_fffeu32.to_le_bytes().repeat(256); // synthetic ARM B .
    for percent in [100,120,140,200,400,800] {
        let mut scheduled=Machine::new(rom.clone());
        let mut reference=Machine::new(rom.clone());
        let mut clock=input_config::fast_forward::FrameClock::default();
        let mut total=0;
        for _ in 0..5 {
            let count=clock.next_batch(percent);
            for _ in 0..count {scheduled.run_frame(1_000_000);assert!(scheduled.bus.frame_ready);}
            total+=count;
        }
        assert_eq!(total,percent/20);
        assert_eq!(scheduled.bus.frames,u64::from(total));
        for _ in 0..total {reference.run_frame(1_000_000);}
        assert!(scheduled.bus.clock>0);
        assert_eq!(scheduled.save_state().unwrap(),reference.save_state().unwrap(),"guest state/clock at {percent}%");
        clock.reset();
        let effective=input_config::fast_forward::effective_percent(percent,false,false);
        assert_eq!(clock.next_batch(effective),1);
        scheduled.run_frame(1_000_000);reference.run_frame(1_000_000);
        assert_eq!(scheduled.save_state().unwrap(),reference.save_state().unwrap());
    }
}
