pub struct Stats {
    pub crashes: u64,
    pub execs: u64,
    pub execs_per_sec: u64
}

impl Stats {

    pub fn new() -> Self {
        Stats {
            crashes: 0,
            execs: 0,
            execs_per_sec: 0
        }
    }

}
