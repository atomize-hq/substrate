use crate::WorldCleanupArgs;
use anyhow::Result;

pub fn run(args: &WorldCleanupArgs) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        linux::run(args)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let purge_hint = if args.purge { " --purge" } else { "" };
        println!(
            "substrate world cleanup currently targets Linux hosts. On macOS run:\n  limactl shell substrate sudo substrate world cleanup{}\nWindows/WSL:\n  wsl -d substrate-wsl -- sudo substrate world cleanup{}",
            purge_hint, purge_hint
        );
        println!("These commands inspect/delete guest namespaces and nft tables from inside the provisioned environment.");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use anyhow::{anyhow, Context};
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const NETNS_PREFIX: &str = "substrate-";
    const WORLD_PREFIX: &str = "wld_";

    pub(super) fn run(args: &WorldCleanupArgs) -> Result<()> {
        let plan = scan();
        if let Some(err) = plan.netns_error {
            eprintln!("[cleanup] warn: {}", err);
        }
        if let Some(err) = plan.cgroup_error {
            eprintln!("[cleanup] warn: {}", err);
        }

        if plan.worlds.is_empty() {
            println!("No substrate namespaces or cgroups detected.");
        } else {
            for world in &plan.worlds {
                describe_world(world);
                if args.purge {
                    if let Err(e) = purge_world(world) {
                        eprintln!("[cleanup] warn: {}", e);
                        print_manual_instructions(world);
                    }
                } else {
                    print_manual_instructions(world);
                }
                println!();
            }
        }

        if let Some(err) = plan.nft_scan_error {
            eprintln!("[cleanup] warn: {}", err);
            println!("Manual nft inspection: sudo nft list tables | grep substrate_");
        } else if !plan.nft_tables.is_empty() {
            println!("Host nft tables with substrate prefix:");
            for table in &plan.nft_tables {
                println!("  - inet {}", table);
            }
            if args.purge {
                for table in &plan.nft_tables {
                    run_step(
                        &format!("delete host nft table {}", table),
                        delete_host_table(table),
                        Some(format!("sudo nft delete table inet {}", table)),
                    );
                }
            } else {
                println!("To remove host tables, run: sudo nft delete table inet <name>");
            }
        }

        Ok(())
    }

    #[derive(Default)]
    struct ScanPlan {
        worlds: Vec<WorldResidue>,
        nft_tables: Vec<String>,
        netns_error: Option<String>,
        cgroup_error: Option<String>,
        nft_scan_error: Option<String>,
    }

    fn scan() -> ScanPlan {
        let mut plan = ScanPlan::default();
        let mut worlds: BTreeMap<String, WorldResidue> = BTreeMap::new();

        match read_netns() {
            Ok(entries) => {
                for (world_id, info) in entries {
                    worlds
                        .entry(world_id.clone())
                        .or_insert_with(|| WorldResidue::new(&world_id))
                        .netns = Some(info);
                }
            }
            Err(e) => plan.netns_error = Some(format!("{e}")),
        }

        match read_cgroups() {
            Ok(entries) => {
                for (world_id, info) in entries {
                    worlds
                        .entry(world_id.clone())
                        .or_insert_with(|| WorldResidue::new(&world_id))
                        .cgroup = Some(info);
                }
            }
            Err(e) => plan.cgroup_error = Some(format!("{e}")),
        }

        match read_host_tables() {
            Ok(tables) => plan.nft_tables = tables,
            Err(e) => plan.nft_scan_error = Some(format!("{e}")),
        }

        plan.worlds = worlds.into_values().collect();
        plan.worlds.sort_by(|a, b| a.world_id.cmp(&b.world_id));
        plan
    }

    #[derive(Debug)]
    struct NetnsInfo {
        name: String,
        pids: Vec<i32>,
        pids_error: Option<String>,
    }

    #[derive(Debug)]
    struct CgroupInfo {
        path: PathBuf,
        procs: Vec<i32>,
        procs_error: Option<String>,
    }

    #[derive(Debug)]
    struct WorldResidue {
        world_id: String,
        netns: Option<NetnsInfo>,
        cgroup: Option<CgroupInfo>,
    }

    impl WorldResidue {
        fn new(world_id: &str) -> Self {
            Self {
                world_id: world_id.to_string(),
                netns: None,
                cgroup: None,
            }
        }

        fn is_idle(&self) -> bool {
            let netns_idle = self
                .netns
                .as_ref()
                .map(|n| n.pids_error.is_none() && n.pids.is_empty())
                .unwrap_or(true);
            let cgroup_idle = self
                .cgroup
                .as_ref()
                .map(|c| c.procs_error.is_none() && c.procs.is_empty())
                .unwrap_or(true);
            netns_idle && cgroup_idle
        }
    }

    #[derive(Deserialize)]
    struct NftList {
        nftables: Vec<NftEntry>,
    }

    #[derive(Deserialize)]
    struct NftEntry {
        #[serde(default)]
        table: Option<NftTable>,
    }

    #[derive(Deserialize)]
    struct NftTable {
        family: String,
        name: String,
    }

    fn read_netns() -> Result<Vec<(String, NetnsInfo)>> {
        let output = Command::new("ip")
            .args(["netns", "list"])
            .output()
            .context("failed to run ip netns list")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ip netns list failed: {}", stderr.trim()));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();
        for line in stdout.lines() {
            let name = line.split_whitespace().next().unwrap_or_default();
            if let Some(world_id) = name.strip_prefix(NETNS_PREFIX) {
                let (pids, pids_error) = read_netns_pids(name);
                entries.push((
                    world_id.to_string(),
                    NetnsInfo {
                        name: name.to_string(),
                        pids,
                        pids_error,
                    },
                ));
            }
        }
        Ok(entries)
    }

    fn read_netns_pids(ns: &str) -> (Vec<i32>, Option<String>) {
        match Command::new("ip").args(["netns", "pids", ns]).output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut pids = Vec::new();
                for line in stdout.lines() {
                    if let Ok(pid) = line.trim().parse::<i32>() {
                        pids.push(pid);
                    }
                }
                (pids, None)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                (Vec::new(), Some(stderr.trim().to_string()))
            }
            Err(e) => (Vec::new(), Some(e.to_string())),
        }
    }

    fn read_cgroups() -> Result<Vec<(String, CgroupInfo)>> {
        let base = Path::new("/sys/fs/cgroup/substrate");
        if !base.exists() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for entry in fs::read_dir(base).context("failed to read /sys/fs/cgroup/substrate")? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let Some(name_str) = name.to_str() else {
                continue;
            };
            if !name_str.starts_with(WORLD_PREFIX) {
                continue;
            }
            let path = entry.path();
            let (procs, procs_error) = read_pid_file(&path.join("cgroup.procs"));
            entries.push((
                name_str.to_string(),
                CgroupInfo {
                    path,
                    procs,
                    procs_error,
                },
            ));
        }
        Ok(entries)
    }

    fn read_pid_file(path: &Path) -> (Vec<i32>, Option<String>) {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let mut pids = Vec::new();
                for line in contents.lines() {
                    if let Ok(pid) = line.trim().parse::<i32>() {
                        pids.push(pid);
                    }
                }
                (pids, None)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Vec::new(), None),
            Err(e) => (Vec::new(), Some(e.to_string())),
        }
    }

    fn read_host_tables() -> Result<Vec<String>> {
        let output = Command::new("nft")
            .args(["-j", "list", "tables"])
            .output()
            .context("failed to run nft list tables")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "nft list tables failed (need sudo?): {}",
                stderr.trim()
            ));
        }
        let parsed: NftList =
            serde_json::from_slice(&output.stdout).context("failed to parse nft list output")?;
        let mut names = Vec::new();
        for entry in parsed.nftables {
            if let Some(table) = entry.table {
                if table.family == "inet" && table.name.starts_with("substrate_") {
                    names.push(table.name);
                }
            }
        }
        Ok(names)
    }

    fn describe_world(world: &WorldResidue) {
        println!("World {}:", world.world_id);
        match &world.netns {
            Some(ns) => {
                if let Some(err) = &ns.pids_error {
                    println!("  - netns {} (pid scan failed: {})", ns.name, err);
                } else if ns.pids.is_empty() {
                    println!("  - netns {} (idle)", ns.name);
                } else {
                    println!(
                        "  - netns {} ({} pid{})",
                        ns.name,
                        ns.pids.len(),
                        if ns.pids.len() == 1 { "" } else { "s" }
                    );
                }
            }
            None => println!("  - netns: not found"),
        }

        match &world.cgroup {
            Some(cg) => {
                if let Some(err) = &cg.procs_error {
                    println!(
                        "  - cgroup {} (pid scan failed: {})",
                        cg.path.display(),
                        err
                    );
                } else if cg.procs.is_empty() {
                    println!("  - cgroup {} (idle)", cg.path.display());
                } else {
                    println!(
                        "  - cgroup {} ({} pid{})",
                        cg.path.display(),
                        cg.procs.len(),
                        if cg.procs.len() == 1 { "" } else { "s" }
                    );
                }
            }
            None => println!("  - cgroup: not found"),
        }

        println!(
            "  - status: {}",
            if world.is_idle() {
                "idle"
            } else {
                "ACTIVE (skip purge while processes exist)"
            }
        );
    }

    fn print_manual_instructions(world: &WorldResidue) {
        println!("  - Manual cleanup commands:");
        if let Some(ns) = &world.netns {
            println!(
                "    sudo ip netns exec {} nft delete table inet substrate_{}",
                ns.name, world.world_id
            );
            println!("    sudo ip netns delete {}", ns.name);
        } else {
            println!(
                "    sudo nft delete table inet substrate_{}",
                world.world_id
            );
        }
        let cg_path = world
            .cgroup
            .as_ref()
            .map(|c| c.path.display().to_string())
            .unwrap_or_else(|| format!("/sys/fs/cgroup/substrate/{}", world.world_id));
        println!("    sudo rm -rf {}", cg_path);
    }

    fn purge_world(world: &WorldResidue) -> Result<()> {
        if !world.is_idle() {
            println!("  - Skipping purge; namespace/cgroup still have processes");
            return Ok(());
        }
        let table_name = format!("substrate_{}", world.world_id);
        if let Some(ns) = &world.netns {
            run_step(
                &format!("remove nft table via {}", ns.name),
                delete_nft_in_netns(&ns.name, &table_name),
                Some(format!(
                    "sudo ip netns exec {} nft delete table inet {}",
                    ns.name, table_name
                )),
            );
            run_step(
                &format!("delete netns {}", ns.name),
                delete_netns(&ns.name),
                Some(format!("sudo ip netns delete {}", ns.name)),
            );
        } else {
            run_step(
                "remove host nft table",
                delete_host_table(&table_name),
                Some(format!("sudo nft delete table inet {}", table_name)),
            );
        }
        if let Some(cg) = &world.cgroup {
            run_step(
                &format!("remove cgroup {}", cg.path.display()),
                remove_cgroup_dir(&cg.path),
                Some(format!("sudo rm -rf {}", cg.path.display())),
            );
        } else {
            let path = PathBuf::from(format!("/sys/fs/cgroup/substrate/{}", world.world_id));
            run_step(
                &format!("remove cgroup {}", path.display()),
                remove_cgroup_dir(&path),
                Some(format!("sudo rm -rf {}", path.display())),
            );
        }
        Ok(())
    }

    fn run_step(label: &str, result: Result<()>, manual: Option<String>) {
        match result {
            Ok(_) => println!("  - {}: ok", label),
            Err(e) => {
                eprintln!("[cleanup] warn: {} failed: {}", label, e);
                if let Some(cmd) = manual {
                    println!("    manual: {}", cmd);
                }
            }
        }
    }

    fn delete_nft_in_netns(ns: &str, table: &str) -> Result<()> {
        let output = Command::new("ip")
            .args(["netns", "exec", ns, "nft", "delete", "table", "inet", table])
            .output()
            .context("failed to run ip netns exec nft delete")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "nft delete table inet {} inside {}: {}",
                table,
                ns,
                stderr.trim()
            ));
        }
        Ok(())
    }

    fn delete_netns(ns: &str) -> Result<()> {
        let output = Command::new("ip")
            .args(["netns", "delete", ns])
            .output()
            .context("failed to run ip netns delete")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("ip netns delete {}: {}", ns, stderr.trim()));
        }
        Ok(())
    }

    fn delete_host_table(table: &str) -> Result<()> {
        let output = Command::new("nft")
            .args(["delete", "table", "inet", table])
            .output()
            .context("failed to run nft delete table")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "nft delete table inet {} failed: {}",
                table,
                stderr.trim()
            ));
        }
        Ok(())
    }

    fn remove_cgroup_dir(path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        match fs::remove_dir_all(path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
        .with_context(|| format!("failed to remove {}", path.display()))?;
        Ok(())
    }
}
