#[cfg(pprof)]
use std::{fs::File, os::raw::c_int, path::Path};

#[cfg(pprof)]
use criterion::profiler::Profiler;

#[cfg(pprof)]
use pprof::ProfilerGuard;

/// This contains the ProfilerGuard (which starts profiling when created).
#[cfg(pprof)]
pub struct FlamegraphProfiler<'a> {
    frequency: c_int,
    active_profiler: Option<ProfilerGuard<'a>>,
}

#[cfg(pprof)]
impl<'a> FlamegraphProfiler<'a> {
    pub fn new(frequency: c_int) -> Self {
        FlamegraphProfiler {
            frequency,
            active_profiler: None,
        }
    }
}

#[cfg(pprof)]
impl<'a> Profiler for FlamegraphProfiler<'a> {
    
    fn start_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        self.active_profiler = Some(ProfilerGuard::new(self.frequency).unwrap());
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
        std::fs::create_dir_all(benchmark_dir).unwrap();
        let flamegraph_path = benchmark_dir.join("flamegraph.svg");
        let flamegraph_file = File::create(&flamegraph_path)
            .expect("File system error while creating flamegraph.svg");
        if let Some(profiler) = self.active_profiler.take() {
            profiler
                .report()
                .build()
                .unwrap()
                .flamegraph(flamegraph_file)
                .expect("Error writing flamegraph");
        }
    }
}