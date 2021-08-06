use std::result::Result;

use probes::{cpu, disk_usage, load, memory};
use snafu::ResultExt;
use systemstat::{Platform, System};

use crate::error::*;
use crate::structs::{CpuStat, CpuStatPercentages, DiskUsage, LoadAverage, MemStat};

pub fn cpu_stats() -> Result<String, StatError> {
    let cpu_stats = cpu::proc::read().context(ReadCpuStat)?;
    let s = cpu_stats.stat;
    let cpu = CpuStat {
        user: s.user,
        system: s.system,
        nice: s.nice,
        idle: s.idle,
    };
    let json_cpu = serde_json::to_string(&cpu).context(SerdeSerialize)?;

    Ok(json_cpu)
}

pub fn cpu_stats_percent() -> Result<String, StatError> {
    let cpu_stats = cpu::proc::read().context(ReadCpuStat)?;
    let s = cpu_stats.stat.in_percentages();
    let cpu = CpuStatPercentages {
        user: s.user,
        system: s.system,
        nice: s.nice,
        idle: s.idle,
    };
    let json_cpu = serde_json::to_string(&cpu).context(SerdeSerialize)?;

    Ok(json_cpu)
}

pub fn disk_usage() -> Result<String, StatError> {
    let disks = disk_usage::read().context(ReadDiskUsage)?;
    let mut disk_usages = Vec::new();
    for d in disks {
        let disk = DiskUsage {
            filesystem: d.filesystem,
            one_k_blocks: d.one_k_blocks,
            one_k_blocks_used: d.one_k_blocks_used,
            one_k_blocks_free: d.one_k_blocks_free,
            used_percentage: d.used_percentage,
            mountpoint: d.mountpoint,
        };
        disk_usages.push(disk);
    }
    let json_disks = serde_json::to_string(&disk_usages).context(SerdeSerialize)?;

    Ok(json_disks)
}

pub fn load_average() -> Result<String, StatError> {
    let l = load::read().context(ReadLoadAvg)?;
    let load_avg = LoadAverage {
        one: l.one,
        five: l.five,
        fifteen: l.fifteen,
    };
    let json_load_avg = serde_json::to_string(&load_avg).context(SerdeSerialize)?;

    Ok(json_load_avg)
}

pub fn mem_stats() -> Result<String, StatError> {
    let m = memory::read().context(ReadMemStat)?;
    let mem = MemStat {
        total: m.total(),
        free: m.free(),
        used: m.used(),
    };
    let json_mem = serde_json::to_string(&mem).context(SerdeSerialize)?;

    Ok(json_mem)
}

pub fn uptime() -> Result<String, StatError> {
    let sys = System::new();
    let uptime = sys.uptime().context(ReadUptime)?;
    let json_uptime = serde_json::to_string(&uptime).context(SerdeSerialize)?;

    Ok(json_uptime)
}
