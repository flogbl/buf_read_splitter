use sysinfo::System;

pub struct Cpu {
    system: System,
}

impl Cpu {
    pub fn new() -> Self {
        let system = System::new_all();
        Self { system }
    }
    pub fn cpu_time(&mut self) -> u64 {
        self.system.refresh_all();
        let pid = sysinfo::get_current_pid().unwrap();
        let process = self.system.process(pid).unwrap();
        process.accumulated_cpu_time()
    }
}
