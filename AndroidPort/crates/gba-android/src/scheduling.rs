//! Keep serial emulation on performance cores when Android exposes capacity.
//! No frequency/thermal changes, root, model names or fixed CPU numbers.
#[cfg(any(target_os="android", test))]
fn performance_cpus(capacities:&[(usize,u32)])->Vec<usize>{
    let Some(minimum)=capacities.iter().map(|(_,capacity)|*capacity).min() else{return Vec::new();};
    // Include both performance and prime clusters. A homogeneous or restricted
    // CPU set remains entirely under the normal scheduler's control.
    capacities.iter().filter(|(_,capacity)|*capacity>minimum).map(|(cpu,_)|*cpu).collect()
}

#[cfg(target_os="android")]
pub fn prefer_performance_cpus()->bool {
    let mut allowed=unsafe{std::mem::zeroed::<libc::cpu_set_t>()};
    if unsafe{libc::sched_getaffinity(0,std::mem::size_of_val(&allowed),&mut allowed)}!=0{return false;}
    let mut capacities=Vec::new();
    for cpu in 0..libc::CPU_SETSIZE as usize {
        if !unsafe{libc::CPU_ISSET(cpu,&allowed)}{continue;}
        let Some(capacity)=std::fs::read_to_string(format!("/sys/devices/system/cpu/cpu{cpu}/cpu_capacity"))
            .ok().and_then(|s|s.trim().parse::<u32>().ok()).filter(|v|*v>0) else{return false;};
        capacities.push((cpu,capacity));
    }
    let fast=performance_cpus(&capacities);
    if fast.is_empty(){return false;}
    let mut selected=unsafe{std::mem::zeroed::<libc::cpu_set_t>()};
    unsafe {
        libc::CPU_ZERO(&mut selected);
        for cpu in fast {libc::CPU_SET(cpu,&mut selected);}
        libc::sched_setaffinity(0,std::mem::size_of_val(&selected),&selected)==0
    }
}
#[cfg(not(target_os="android"))]
pub fn prefer_performance_cpus()->bool {false}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capacity_policy_supports_two_and_three_clusters_and_safe_fallbacks(){
        assert!(performance_cpus(&[]).is_empty());
        assert!(performance_cpus(&[(0,544),(1,544)]).is_empty());
        assert_eq!(performance_cpus(&[(0,544),(1,544),(6,1024),(7,1024)]),[6,7]);
        assert_eq!(performance_cpus(&[(0,400),(4,760),(5,760),(6,760),(7,1024)]),[4,5,6,7]);
        assert_eq!(performance_cpus(&[(2,400),(3,800)]),[3]);
    }
}
