// Included only inside member_runtime's private cfg(test) module. No public seam.
mod native0156 {
    use super::*;
    use std::io::Read;
    use std::os::unix::process::CommandExt;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    const WORKER: &str = "member_runtime::tests::native0156::codex0156_native_isolated_worker";
    const GATEWAY_WORKER: &str =
        "providers::openai::native0156_fixture::codex0156_actual_gateway_fixture_worker";

    fn write_json(path: impl AsRef<Path>, value: &serde_json::Value) {
        let path = path.as_ref();
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(value).unwrap()).unwrap();
        fs::rename(temporary, path).unwrap();
    }

    fn digest(path: &Path) -> String {
        let mut file = fs::File::open(path).unwrap();
        let mut digest = Sha256::new();
        let mut chunk = [0; 65536];
        loop {
            let count = file.read(&mut chunk).unwrap();
            if count == 0 {
                break;
            }
            digest.update(&chunk[..count]);
        }
        format!("{:x}", digest.finalize())
    }

    fn check_pair(package: &Path) -> serde_json::Value {
        let pins = config_projection::Codex0156ReleaseBundleV1::pinned();
        let mut checked = Vec::new();
        for (name, pin) in [
            ("codex", pins.main),
            ("codex-code-mode-host", pins.code_mode_host),
        ] {
            let path = package.join(name);
            let metadata = fs::metadata(&path).expect("retained official artifact");
            assert_eq!(metadata.len(), pin.extracted_byte_length, "{name}");
            assert_eq!(digest(&path), pin.extracted_executable_sha256, "{name}");
            assert_ne!(
                metadata.mode() & 0o111,
                0,
                "executable private copy required"
            );
            let elf = config_projection::LinuxArtifactSourceV1::validate_e3_static_elf_v1(
                &fs::File::open(&path).unwrap(),
            )
            .expect("production static ELF validator");
            checked.push(json!({"path":path,"sha256":pin.extracted_executable_sha256,
                "length":metadata.len(),"device":metadata.dev(),"inode":metadata.ino(),
                "elf":format!("{elf:?}")}));
        }
        json!(checked)
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Process {
        pid: u32,
        start: String,
        ppid: u32,
        state: String,
        exe: String,
        argv: Vec<String>,
    }

    fn process(pid: u32) -> Option<Process> {
        let directory = PathBuf::from(format!("/proc/{pid}"));
        let stat = fs::read_to_string(directory.join("stat")).ok()?;
        let fields = stat
            .rsplit_once(") ")?
            .1
            .split_whitespace()
            .collect::<Vec<_>>();
        Some(Process {
            pid,
            start: fields.get(19)?.to_string(),
            ppid: fields.get(1)?.parse().ok()?,
            state: fields.first()?.to_string(),
            exe: fs::read_link(directory.join("exe"))
                .ok()?
                .display()
                .to_string(),
            argv: fs::read(directory.join("cmdline"))
                .ok()?
                .split(|b| *b == 0)
                .filter(|a| !a.is_empty())
                .map(|a| String::from_utf8_lossy(a).into_owned())
                .collect(),
        })
    }

    #[derive(Debug, Serialize)]
    struct ProcIdentity {
        pid: u32,
        start: String,
        state: String,
    }

    fn process_identity(pid: u32) -> Option<ProcIdentity> {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let fields = stat
            .rsplit_once(") ")?
            .1
            .split_whitespace()
            .collect::<Vec<_>>();
        Some(ProcIdentity {
            pid,
            start: fields.get(19)?.to_string(),
            state: fields.first()?.to_string(),
        })
    }

    fn is_alive(observed: &Process) -> bool {
        // Identity survives exec and includes zombies until actually reaped.
        // exe/cmdline read failures never establish process absence.
        process_identity(observed.pid).is_some_and(|now| now.start == observed.start)
    }

    fn qualify_native0156_process_identity(evidence_root: &Path) -> serde_json::Value {
        struct OwnedChild(Child);
        impl Drop for OwnedChild {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let root = tempfile::tempdir_in("/tmp").unwrap();
        let mut child = OwnedChild(
            Command::new("/bin/sh")
                .args([
                    "-c",
                    "read gate; exec /usr/bin/python3 -c 'import time; time.sleep(30)'",
                ])
                .current_dir(root.path())
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let shell = fs::canonicalize("/bin/sh").unwrap().display().to_string();
        let deadline = Instant::now() + Duration::from_secs(5);
        let before = loop {
            if let Some(p) = process(child.0.id()) {
                if p.exe == shell {
                    break p;
                }
            }
            assert!(
                Instant::now() < deadline,
                "controlled shell did not become observable"
            );
            std::thread::sleep(Duration::from_millis(2));
        };
        child
            .0
            .stdin
            .take()
            .unwrap()
            .write_all(b"continue\n")
            .unwrap();
        let python = fs::canonicalize("/usr/bin/python3")
            .unwrap()
            .display()
            .to_string();
        let after = loop {
            if let Some(p) = process(child.0.id()) {
                if p.exe == python {
                    break p;
                }
            }
            assert!(
                Instant::now() < deadline,
                "controlled exec did not become observable"
            );
            std::thread::sleep(Duration::from_millis(2));
        };
        assert_eq!((before.pid, &before.start), (after.pid, &after.start));
        assert_ne!(before.exe, after.exe);
        assert!(
            is_alive(&before),
            "exec changes metadata while retaining PID/start identity"
        );
        child.0.kill().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let zombie = loop {
            let identity = process_identity(before.pid).expect("child is not reaped before wait");
            if identity.state == "Z" {
                break identity;
            }
            assert!(Instant::now() < deadline, "killed child did not exit");
            std::thread::sleep(Duration::from_millis(2));
        };
        assert_eq!(zombie.start, before.start);
        assert!(
            is_alive(&before),
            "an unreaped zombie must not be reported absent"
        );
        assert!(!child.0.wait().unwrap().success());
        assert!(!is_alive(&before), "wait must reap the actual identity");
        let proof = json!({"invocations":1,"passed":true,"shell_before_exec":before,
            "python_after_exec":after,"unreaped_zombie":zombie,
            "identity_after_wait":process_identity(before.pid),
            "placement":"unconditional isolated worker prerequisite before observer/gateway startup"});
        write_json(
            evidence_root.join("process-identity-regression.json"),
            &proof,
        );
        eprintln!("isolated process identity regression passed: {proof}");
        proof
    }

    struct Observer {
        stop: Arc<AtomicBool>,
        thread: Option<std::thread::JoinHandle<Vec<Process>>>,
    }
    impl Observer {
        fn start(root: PathBuf) -> Self {
            let stop = Arc::new(AtomicBool::new(false));
            let stopping = stop.clone();
            let owner = std::process::id();
            let thread = std::thread::spawn(move || {
                let mut seen = BTreeMap::new();
                while !stopping.load(Ordering::Relaxed) {
                    // Discover children of every thread in the attempt's known process
                    // tree. Full-host scans can miss millisecond UAA capability probes.
                    // Previously seen identities remain tracked after reparenting.
                    let mut pending = vec![owner];
                    pending.extend(
                        seen.values()
                            .filter(|p| is_alive(p))
                            .map(|p: &Process| p.pid),
                    );
                    let mut owned = BTreeSet::new();
                    let mut all = Vec::new();
                    while let Some(pid) = pending.pop() {
                        if !owned.insert(pid) {
                            continue;
                        }
                        if let Ok(tasks) = fs::read_dir(format!("/proc/{pid}/task")) {
                            for task in tasks.filter_map(|e| e.ok()) {
                                if let Ok(children) =
                                    fs::read_to_string(task.path().join("children"))
                                {
                                    pending.extend(
                                        children
                                            .split_whitespace()
                                            .filter_map(|p| p.parse::<u32>().ok()),
                                    );
                                }
                            }
                        }
                        if pid != owner {
                            if let Some(p) = process(pid) {
                                all.push(p);
                            }
                        }
                    }
                    let mut changed = false;
                    for p in all {
                        if p.pid != owner
                            && (owned.contains(&p.pid)
                                || seen.contains_key(&(p.pid, p.start.clone())))
                        {
                            let key = (p.pid, p.start.clone());
                            if seen.get(&key) == Some(&p) {
                                continue;
                            }
                            changed = true;
                            let mut transitions = fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(root.join("observed-process-transitions.jsonl"))
                                .unwrap();
                            writeln!(transitions, "{}", serde_json::to_string(&p).unwrap())
                                .unwrap();
                            if p.argv.iter().any(|a| a == "exec")
                                && p.exe.ends_with("/package/bin/codex")
                            {
                                if let Ok(environ) = fs::read(format!("/proc/{}/environ", p.pid)) {
                                    if let Some(home) = environ
                                        .split(|b| *b == 0)
                                        .find_map(|row| row.strip_prefix(b"CODEX_HOME="))
                                    {
                                        let home = PathBuf::from(
                                            String::from_utf8_lossy(home).into_owned(),
                                        );
                                        if let Ok(config) =
                                            fs::read_to_string(home.join("config.toml"))
                                        {
                                            let native_environment = environ
                                                .split(|b| *b == 0)
                                                .filter_map(|row| {
                                                    let row = String::from_utf8_lossy(row);
                                                    let (key, value) = row.split_once('=')?;
                                                    [
                                                        "HOME",
                                                        "CODEX_HOME",
                                                        "TMPDIR",
                                                        "PATH",
                                                        "XDG_CONFIG_HOME",
                                                        "XDG_CACHE_HOME",
                                                    ]
                                                    .contains(&key)
                                                    .then(|| (key.to_string(), value.to_string()))
                                                })
                                                .collect::<BTreeMap<_, _>>();
                                            write_json(
                                                root.join(format!(
                                                    "observed-native-startup-{}-{}.json",
                                                    p.pid, p.start
                                                )),
                                                &json!({"process":p,"codex_home":home,"config":config,"environment":native_environment,
                                                    "actual_proc_cwd":fs::read_link(format!("/proc/{}/cwd",p.pid)).ok(),
                                                }),
                                            );
                                        }
                                    }
                                }
                            }
                            if p.exe.ends_with("/package/bin/codex-code-mode-host") {
                                let identity = root.join(format!(
                                    "observed-helper-startup-{}-{}.json",
                                    p.pid, p.start
                                ));
                                if !identity.exists() {
                                    write_json(identity, &json!(p));
                                }
                            }
                            seen.insert((p.pid, p.start.clone()), p);
                        }
                    }
                    if changed || !root.join("observed-processes.json").exists() {
                        write_json(
                            root.join("observed-processes.json"),
                            &json!(seen.values().collect::<Vec<_>>()),
                        );
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
                seen.into_values().collect()
            });
            Self {
                stop,
                thread: Some(thread),
            }
        }
        fn finish(mut self) -> Vec<Process> {
            self.stop.store(true, Ordering::Relaxed);
            self.thread.take().unwrap().join().unwrap()
        }
    }
    impl Drop for Observer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    struct Gateway {
        child: Child,
        root: PathBuf,
    }
    impl Drop for Gateway {
        fn drop(&mut self) {
            let _ = fs::write(self.root.join("stop"), "stop");
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline {
                if let Some(status) = self.child.try_wait().ok().flatten() {
                    write_json(
                        self.root.join("worker-shutdown.json"),
                        &json!({"status":status.to_string(),"success":status.success(),"rescue":false}),
                    );
                    return;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = fs::write(
                self.root.join("fixture-rescue-required"),
                "gateway worker failed to stop",
            );
        }
    }

    async fn ready(path: &Path) -> serde_json::Value {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Ok(bytes) = fs::read(path) {
                if let Ok(value) = serde_json::from_slice(&bytes) {
                    return value;
                }
            }
            assert!(Instant::now() < deadline, "missing {}", path.display());
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    fn preserve_observations(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).unwrap();
        for entry in fs::read_dir(source).unwrap().filter_map(|entry| entry.ok()) {
            // Official binaries remain available in retained package evidence;
            // copying their identities is enough. Never follow native symlinks.
            if entry.file_name() == "package" {
                continue;
            }
            let target = destination.join(entry.file_name());
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                preserve_observations(&entry.path(), &target);
            } else if kind.is_file() {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    fn isolated(lane: &str) {
        let evidence_root = PathBuf::from(
            std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_ATTEMPT_DIR")
                .expect("attempt directory required"),
        );
        fs::create_dir_all(&evidence_root).unwrap();
        let runtime_owner = tempfile::Builder::new()
            .prefix("native0156-")
            .tempdir_in("/tmp")
            .expect("private runtime root outside Git ancestry");
        let root = runtime_owner.path().to_path_buf();
        write_json(
            evidence_root.join("runtime-placement.json"),
            &json!({
                "private_runtime_root":root,
                "evidence_root":evidence_root,
                "purpose":"disposable native resources, not a new orchestration record root",
            }),
        );
        for name in ["home", "xdg-config", "xdg-cache", "tmp"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        let output = fs::File::create(root.join("manager-worker.output")).unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", WORKER, "--ignored", "--nocapture"])
            .current_dir(&root)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", root.join("home"))
            .env("XDG_CONFIG_HOME", root.join("xdg-config"))
            .env("XDG_CACHE_HOME", root.join("xdg-cache"))
            .env("TMPDIR", root.join("tmp"))
            .env("LANG", "C.UTF-8")
            .env("SUBSTRATE_NATIVE_CODEX_0156_ATTEMPT_DIR", &root)
            .env("SUBSTRATE_NATIVE_CODEX_0156_LANE", lane)
            .env(
                "SUBSTRATE_NATIVE_CODEX_0156_GATEWAY_FIXTURE_BIN",
                std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_GATEWAY_FIXTURE_BIN")
                    .expect("gateway test artifact required"),
            )
            .env(
                "SUBSTRATE_NATIVE_CODEX_0156_PACKAGE_DIR",
                std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_PACKAGE_DIR")
                    .expect("retained package required"),
            )
            .stdin(Stdio::null())
            .stdout(output.try_clone().unwrap())
            .stderr(output)
            .process_group(0);
        write_json(
            root.join("sanitized-startup.json"),
            &json!({
                "environment":command.get_envs().map(|(k,v)|(k.to_string_lossy().to_string(),v.map(|x|x.to_string_lossy().to_string()))).collect::<BTreeMap<_,_>>(),
                "system_inputs":"stock system configuration and trust; no host policy edits",
                "native_auth_seed":"inert auth.json {}; tokenless object possible, no credentials",
            }),
        );
        let mut child = command.spawn().unwrap();
        let identity = process(child.id()).expect("isolated worker identity");
        let deadline = Instant::now() + Duration::from_secs(210);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if Instant::now() >= deadline {
                break None;
            }
            std::thread::sleep(Duration::from_millis(30));
        };
        if !status.is_some_and(|s| s.success()) {
            // Containment only. Never count a rescue as product cleanup success.
            unsafe {
                libc::kill(-(identity.pid as i32), libc::SIGKILL);
            }
            let _ = child.wait();
            write_json(
                root.join("FAILED-containment.json"),
                &json!({"worker":identity,"status":status.map(|s|s.to_string()),"rescue":"attempt process group SIGKILL; cleanup unproved/failed"}),
            );
            preserve_observations(&root, &evidence_root);
            panic!(
                "native qualification failed: {}",
                fs::read_to_string(root.join("manager-worker.output")).unwrap()
            );
        }
        preserve_observations(&root, &evidence_root);
        let receipt: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join("qualification-passed.json")).expect("nonempty success receipt"),
        )
        .unwrap();
        assert_eq!(receipt["lane"], lane);
        assert_eq!(receipt["identity_regression"]["invocations"], 1);
        assert_eq!(receipt["identity_regression"]["passed"], true);
        assert!(receipt["identity_regression"]["identity_after_wait"].is_null());
        eprintln!("native0156 qualification receipt: {receipt}");
    }

    #[test]
    #[ignore = "official retained 0.156.1 pair and actual gateway fixture required"]
    fn codex0156_native_initial_tool_and_two_exact_history_resumes() {
        isolated("history");
    }

    #[test]
    #[ignore = "official retained 0.156.1 pair and actual gateway fixture required"]
    fn codex0156_native_cancel_timeout_disconnect_and_ambiguous_response() {
        isolated("faults");
    }

    async fn frames(
        manager: &MemberRuntimeManager,
        response: Response,
        span: &str,
        root: &Path,
        name: &str,
    ) -> Vec<serde_json::Value> {
        let mut reading = Box::pin(hyper::body::to_bytes(response.into_body()));
        let bytes = match tokio::time::timeout(Duration::from_secs(45), &mut reading).await {
            Ok(result) => result.expect("manager stream"),
            Err(_) => {
                let cancelled = manager.cancel(span, "TERM").unwrap();
                let drained = tokio::time::timeout(Duration::from_secs(5), reading).await;
                write_json(
                    root.join(format!("{name}-deadline.json")),
                    &json!({"existing_manager_cancel":cancelled,"drained":drained.is_ok(),"production_timeout_claim":false}),
                );
                panic!("test turn deadline reached; existing manager cancel invoked");
            }
        };
        fs::write(root.join(format!("{name}-frames.jsonl")), &bytes).unwrap();
        String::from_utf8(bytes.to_vec())
            .unwrap()
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn assert_turn(frames: &[serde_json::Value], answer: &str, root: &Path) {
        assert!(
            frames.iter().any(|f| f["type"] == "exit" && f["exit"] == 0),
            "{frames:?}"
        );
        assert!(
            !frames.iter().any(|f| f["type"] == "error"
                || f["event"]["channel"] == "error"
                || f["event"]["kind"] == "alert"),
            "unexplained happy-path error: {frames:?}"
        );
        assert!(
            serde_json::to_string(frames).unwrap().contains(answer),
            "missing assistant text {answer}: {frames:?}"
        );
        let files = fs::read_dir(root.join("tmp"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("codex_last_message_")
            })
            .map(|e| (e.path(), fs::read(e.path()).unwrap()))
            .collect::<Vec<_>>();
        assert!(
            files.iter().any(|(_, bytes)| bytes == answer.as_bytes()),
            "actual last-message bytes missing/mismatched: {files:?}"
        );
    }

    fn fault_scenario(
        root: &Path,
        workspace: &Path,
        nonce: &str,
        label: &str,
        sleep: u32,
    ) -> PathBuf {
        let tool_file = workspace.join(format!("fault-{label}-tool.json"));
        let marker_file = workspace.join(format!("fault-{label}-markers"));
        let python = format!("import os,time,pathlib,json; p=pathlib.Path({path:?}); s=pathlib.Path('/proc/self/stat').read_text().rsplit(') ',1)[1].split()[19]; p.write_text(json.dumps(dict(pid=os.getpid(),start=s))); pathlib.Path({marker:?}).open('a').write({line:?}); time.sleep({sleep}); print({result:?})",
            path=tool_file.display().to_string(), marker=marker_file.display().to_string(), line=format!("{nonce}:{label}\n"), result=format!("{nonce}:RESULT:{label}"));
        let command = format!("/usr/bin/python3 -c '{}'", python.replace('\'', "'\"'\"'"));
        write_json(
            root.join("gateway/scenario.json"),
            &json!({"mode":if label == "ambiguous" {"ambiguous"} else {"blocked"},
            "nonce":nonce,"fault":label,"command":command}),
        );
        tool_file
    }

    async fn actual_tool(path: &Path) -> Process {
        let reported = ready(path).await;
        let actual = process(reported["pid"].as_u64().unwrap() as u32)
            .expect("actual running tool /proc identity");
        assert_eq!(actual.start, reported["start"].as_str().unwrap());
        actual
    }

    async fn assert_fault_quiescent(root: &Path, bin: &Path, label: &str, tool: &Process) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let observed: Vec<Process> =
                serde_json::from_slice(&fs::read(root.join("observed-processes.json")).unwrap())
                    .unwrap();
            let work = observed
                .iter()
                .filter(|p| p.exe.starts_with(&bin.display().to_string()))
                .collect::<Vec<_>>();
            let survivors = work.iter().filter(|p| is_alive(p)).collect::<Vec<_>>();
            if (survivors.is_empty() && !is_alive(tool)) || Instant::now() >= deadline {
                write_json(
                    root.join(format!("{label}-cleanup.json")),
                    &json!({"tool":tool,
                    "observed_native_and_helpers":work,"survivors":survivors,"tool_alive":is_alive(tool),
                    "tool_identity_after":process_identity(tool.pid),"survivor_states":survivors.iter().map(|p| process_identity(p.pid)).collect::<Vec<_>>(),
                    "scope":"observed PID/start identities must be absent, including zombies; direct wait is not whole-descendant proof"}),
                );
                assert!(
                    !is_alive(tool) && survivors.is_empty(),
                    "{label} cleanup failed: {survivors:?}; tool {tool:?}"
                );
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn wait_turn_free(active: &ActiveMemberRuntime) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while active.active_turn_span_id.lock().unwrap().is_some() {
            assert!(
                Instant::now() < deadline,
                "background manager turn failed to drain"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    #[tokio::test]
    #[ignore = "sanitized self-test worker; parent owns explicit prerequisites"]
    async fn codex0156_native_isolated_worker() {
        let root =
            PathBuf::from(std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_ATTEMPT_DIR").unwrap());
        let lane = std::env::var("SUBSTRATE_NATIVE_CODEX_0156_LANE").unwrap();
        let identity_regression = qualify_native0156_process_identity(&root);
        let package =
            PathBuf::from(std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_PACKAGE_DIR").unwrap());
        let source_pair_before = check_pair(&package);
        write_json(
            root.join("retained-source-pair-before.json"),
            &source_pair_before,
        );
        let bin = root.join("package/bin");
        fs::create_dir_all(&bin).unwrap();
        for name in ["codex", "codex-code-mode-host"] {
            fs::copy(package.join(name), bin.join(name)).unwrap();
            fs::set_permissions(bin.join(name), fs::Permissions::from_mode(0o700)).unwrap();
        }
        let pair_before = check_pair(&bin);
        write_json(root.join("verified-subject-before.json"), &pair_before);
        let observer = Observer::start(root.clone());
        let gateway_root = root.join("gateway");
        fs::create_dir(&gateway_root).unwrap();
        let log = fs::File::create(gateway_root.join("worker.output")).unwrap();
        let gateway = Gateway {
            child: Command::new(
                std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_GATEWAY_FIXTURE_BIN").unwrap(),
            )
            .args(["--exact", GATEWAY_WORKER, "--ignored", "--nocapture"])
            .env("SUBSTRATE_NATIVE_CODEX_0156_ATTEMPT_DIR", &gateway_root)
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .unwrap(),
            root: gateway_root.clone(),
        };
        let gateway_ready = ready(&gateway_root.join("gateway-ready.json")).await;
        // All native paths, including actual cwd, stay outside Git ancestry.
        let workspace = root.join("workspace");
        fs::create_dir(&workspace).unwrap();
        write_json(
            root.join("workspace-placement.json"),
            &json!({
                "actual_process_cwd":workspace,
                "same_workspace_all_turns":true,
                "reason":"native01/02 discovered host project hooks via repository ancestry",
            }),
        );
        let seed = root.join("seed");
        fs::create_dir(&seed).unwrap();
        fs::write(seed.join("auth.json"), "{}").unwrap();
        fs::write(seed.join("config.toml"), format!("model = \"gpt-6-astra\"\nmodel_provider = \"substrate-native-protocol-fixture\"\n[model_providers.substrate-native-protocol-fixture]\nbase_url = {:?}\n",gateway_ready["url"].as_str().unwrap())).unwrap();
        let nonce = uuid::Uuid::new_v4().simple().to_string();
        let commands=(1..=3).map(|turn| {
            let python = format!("import os,time,pathlib,json; p=pathlib.Path({workspace:?}); n={nonce:?}; t={turn}; s=pathlib.Path('/proc/self/stat').read_text().rsplit(') ',1)[1].split()[19]; (p/'markers').open('a').write(n+':T'+str(t)+'\\n'); (p/('tool-'+str(t)+'.json')).write_text(json.dumps(dict(pid=os.getpid(),start=s,exe=os.readlink('/proc/self/exe')))); time.sleep(0.35); print(n+':RESULT:T'+str(t))",workspace=workspace.display().to_string());
            (turn.to_string(),format!("/usr/bin/python3 -c '{}'",python.replace('\'',"'\"'\"'")))
        }).collect::<BTreeMap<_,_>>();
        write_json(
            gateway_root.join("scenario.json"),
            &json!({"mode":"history","nonce":nonce,"workspace":workspace,"commands":commands}),
        );
        let cgroup = root.join("test-owned-regular-attach-surrogate");
        fs::create_dir(&cgroup).unwrap();
        fs::write(cgroup.join("cgroup.procs"), "").unwrap();
        let placement = LinuxWorldPlacementContext {
            working_dir: workspace.clone(),
            cgroup_path: cgroup.clone(),
            require_cgroup_attach: true,
        };
        let mut dispatch = sample_authority_managed_dispatch();
        dispatch.participant_id = "ash_member".into();
        dispatch.retained_worker_launch_authority = None;
        dispatch.e2_launch_activation = None;
        dispatch.resolved_runtime.binary_path = bin.join("codex").display().to_string();
        dispatch.initial_prompt = Some(format!(
            "{nonce}:T1 Perform the bounded qualification command."
        ));
        let manager = MemberRuntimeManager::new();
        let env = HashMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.into(),
            seed.display().to_string(),
        )]);
        let response = manager
            .launch(
                "native0156-fixture".into(),
                env,
                "spn_native0156_1".into(),
                MemberRuntimeLaunchAdmissionV1 {
                    ordinary_effect: None,
                    dispatch,
                    acceptance_context: None,
                    policy_snapshot: sample_e2_submit_turn_request()
                        .policy_snapshot_carrier
                        .unwrap()
                        .policy_snapshot()
                        .unwrap(),
                },
                sample_world_binding(),
                placement,
            )
            .await
            .expect("actual manager initial turn");
        if let Some(active) = manager
            .active_members
            .read()
            .unwrap()
            .by_participant_id
            .get("ash_member")
        {
            write_json(
                root.join("manager-runtime-placement.json"),
                &json!({
                    "process_working_dir":active.process_working_dir,
                    "workspace_dir":active.workspace_dir,
                    "launcher":active.binary_path,
                    "world_entry_working_dir":active.env.get("SUBSTRATE_WORLD_ENTRY_WORKING_DIR"),
                    "codex_home":active.env.get("CODEX_HOME"),
                }),
            );
        }
        let first = frames(&manager, response, "spn_native0156_1", &root, "turn-1").await;
        assert_turn(&first, &format!("{nonce}:A1"), &root);
        let mut request = sample_submit_turn_request();
        let active = manager
            .find_submit_target(&request)
            .expect("retained actual manager identity");
        let uuid = active.uaa_session_id().expect("observed native UUID");
        assert!(uuid::Uuid::parse_str(&uuid).is_ok(), "{uuid}");
        let retained_home = active.env.get("CODEX_HOME").unwrap().clone();
        let launcher = active.binary_path.clone();
        write_json(
            root.join("observed-session.json"),
            &json!({"uuid":uuid,"home":retained_home,"launcher":launcher,"workspace":workspace}),
        );
        for turn in 2..=3 {
            request.run_id = format!("run_native0156_{turn}");
            request.prompt =
                format!("{nonce}:T{turn} Perform the next bounded qualification command.");
            let response = manager
                .submit_turn(request.clone())
                .await
                .expect("actual exact UUID resume");
            let result = frames(&manager, response, "unused", &root, &format!("turn-{turn}")).await;
            assert_turn(&result, &format!("{nonce}:A{turn}"), &root);
            assert_eq!(active.uaa_session_id().as_deref(), Some(uuid.as_str()));
            assert_eq!(active.env.get("CODEX_HOME"), Some(&retained_home));
            assert_eq!(active.binary_path, launcher);
        }
        assert_eq!(
            fs::read_to_string(workspace.join("markers")).unwrap(),
            (1..=3)
                .map(|t| format!("{nonce}:T{t}\n"))
                .collect::<String>()
        );
        let observations = root.join("workspace-observations");
        fs::create_dir(&observations).unwrap();
        for entry in fs::read_dir(&workspace)
            .unwrap()
            .filter_map(|entry| entry.ok())
        {
            if entry.path().is_file() {
                fs::copy(entry.path(), observations.join(entry.file_name())).unwrap();
            }
        }
        if lane == "faults" {
            let upstream_count = || {
                fs::read_dir(&gateway_root)
                    .unwrap()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_name().to_string_lossy().starts_with("request-"))
                    .count()
            };
            let before = upstream_count();
            let marker_before = fs::read(workspace.join("markers")).unwrap();
            let target = cgroup.join("cgroup.procs");
            assert!(fs::symlink_metadata(&target).unwrap().file_type().is_file());
            fs::remove_file(&target).unwrap();
            request.run_id = "run_native0156_missing_attach".into();
            let missing_attach = match manager.submit_turn(request.clone()).await {
                Ok(response) => frames(&manager, response, "unused", &root, "missing-attach").await,
                Err(error) => vec![json!({"submission_error":format!("{error:#}")})],
            };
            write_json(
                root.join("missing-attach-result.json"),
                &json!(missing_attach),
            );
            assert!(!missing_attach
                .iter()
                .any(|f| f["type"] == "exit" && f["exit"] == 0));
            assert_eq!(
                upstream_count(),
                before,
                "missing attach must reject before upstream work"
            );
            assert_eq!(fs::read(workspace.join("markers")).unwrap(), marker_before);
            assert!(!target.exists());
            assert!(active.active_turn_span_id.lock().unwrap().is_none());
            assert_eq!(active.uaa_session_id().as_deref(), Some(uuid.as_str()));
            fs::write(&target, "").unwrap();

            let missing_uuid = uuid::Uuid::new_v4().to_string();
            *active.uaa_session_id.lock().unwrap() = Some(missing_uuid.clone());
            request.run_id = "run_native0156_missing_uuid".into();
            let missing = match manager.submit_turn(request.clone()).await {
                Ok(response) => frames(&manager, response, "unused", &root, "missing-uuid").await,
                Err(error) => vec![json!({"submission_error":format!("{error:#}")})],
            };
            let observed_after = active.uaa_session_id();
            *active.uaa_session_id.lock().unwrap() = Some(uuid.clone());
            write_json(
                root.join("missing-uuid-result.json"),
                &json!({"selector":missing_uuid,
                "frames":missing,"observed_session_after":observed_after,"upstream_before":before,"upstream_after":upstream_count()}),
            );
            assert!(
                !missing
                    .iter()
                    .any(|f| f["type"] == "exit" && f["exit"] == 0),
                "missing exact UUID must reject: {missing:?}"
            );
            assert_eq!(
                observed_after.as_deref(),
                Some(missing_uuid.as_str()),
                "missing selector must not surface a replacement session"
            );
            assert_eq!(
                upstream_count(),
                before,
                "missing exact UUID must not create replacement work"
            );
            assert_eq!(fs::read(workspace.join("markers")).unwrap(), marker_before);
            assert!(active.active_turn_span_id.lock().unwrap().is_none());
            assert!(serde_json::to_string(&missing)
                .unwrap()
                .contains("session not found"));
            for label in ["cancel", "deadline", "disconnect"] {
                let tool_file = fault_scenario(&root, &workspace, &nonce, label, 30);
                request.run_id = format!("run_native0156_{label}");
                request.prompt = format!("{nonce}:F:{label} Perform the bounded blocked command.");
                assert!(build_submitted_turn_run_request(
                    active.as_ref(),
                    request.prompt.clone(),
                    &uuid,
                    None
                )
                .unwrap()
                .timeout
                .is_none());
                let response = manager
                    .submit_turn(request.clone())
                    .await
                    .expect("real blocked turn");
                let tool = actual_tool(&tool_file).await;
                assert!(is_alive(&tool));
                let span = active.active_turn_span_id.lock().unwrap().clone().unwrap();
                match label {
                    "deadline" => {
                        let mut reading = Box::pin(hyper::body::to_bytes(response.into_body()));
                        assert!(
                            tokio::time::timeout(Duration::from_millis(250), &mut reading)
                                .await
                                .is_err(),
                            "test deadline must actually expire"
                        );
                        assert!(manager.cancel(&span, "TERM").unwrap());
                        let bytes = tokio::time::timeout(Duration::from_secs(5), reading)
                            .await
                            .expect("deadline cancellation drain")
                            .unwrap();
                        fs::write(root.join("deadline-frames.jsonl"), &bytes).unwrap();
                        assert!(String::from_utf8_lossy(&bytes).contains("\"exit\":143"));
                        write_json(
                            root.join("deadline-result.json"),
                            &json!({"test_deadline_ms":250,"manager_request_timeout":null,"existing_manager_cancel":true,"production_timeout_claim":false}),
                        );
                    }
                    "disconnect" => {
                        drop(response);
                        tokio::time::sleep(Duration::from_millis(250)).await;
                        assert!(
                            is_alive(&tool),
                            "client disconnect must not masquerade as cancellation"
                        );
                        assert_eq!(
                            active.active_turn_span_id.lock().unwrap().as_deref(),
                            Some(span.as_str())
                        );
                        assert!(manager
                            .active_turns_by_span_id
                            .read()
                            .unwrap()
                            .contains_key(&span));
                        assert!(
                            manager.submit_turn(request.clone()).await.is_err(),
                            "background turn must retain exclusion"
                        );
                        assert!(manager.cancel(&span, "TERM").unwrap());
                        write_json(
                            root.join("disconnect-result.json"),
                            &json!({"dropped_client_stream":true,"background_custody_observed":true,"tool_alive_after_disconnect":true,"explicit_abandonment_cancel":true,"automatic_disconnect_cleanup_claim":false}),
                        );
                    }
                    _ => {
                        assert!(manager.cancel(&span, "TERM").unwrap());
                        let result =
                            frames(&manager, response, &span, &root, "explicit-cancel").await;
                        assert!(result
                            .iter()
                            .any(|f| f["type"] == "exit" && f["exit"] == 143));
                        write_json(
                            root.join("explicit-cancel-result.json"),
                            &json!({"tool":tool,"frames":result}),
                        );
                    }
                }
                wait_turn_free(active.as_ref()).await;
                assert_eq!(active.uaa_session_id().as_deref(), Some(uuid.as_str()));
                assert_fault_quiescent(&root, &bin, label, &tool).await;
            }

            // A separate actual bridge request qualifies existing UAA request.timeout.
            // Manager requests remain timeout=None and retain their existing policy.
            let label = "uaa-timeout";
            let tool_file = fault_scenario(&root, &workspace, &nonce, label, 30);
            let bridge = PromptFulfillmentBridge::for_member_backend(
                &MemberRuntimeBackendKindV1::Codex,
                active.launcher_dir.join("world-entry.sh"),
            )
            .unwrap();
            let mut timeout_request = build_submitted_turn_run_request(
                active.as_ref(),
                format!("{nonce}:F:{label} Perform the bounded blocked command."),
                &uuid,
                None,
            )
            .unwrap();
            timeout_request.timeout = Some(Duration::from_secs(4));
            let control = bridge.run_control(timeout_request).await.unwrap();
            let mut events = control.handle.events;
            let event_task = tokio::spawn(async move {
                let mut all = Vec::new();
                while let Some(event) = events.next().await {
                    all.push(format!("{event:?}"));
                }
                all
            });
            let tool = actual_tool(&tool_file).await;
            let completion =
                tokio::time::timeout(Duration::from_secs(10), control.handle.completion)
                    .await
                    .expect("existing UAA timeout completion");
            let error = completion.expect_err("existing UAA timeout must reject the blocked run");
            let events = tokio::time::timeout(Duration::from_secs(5), event_task)
                .await
                .unwrap()
                .unwrap();
            write_json(
                root.join("uaa-request-timeout-result.json"),
                &json!({"request_timeout_seconds":4,"error":error.to_string(),"events":events,"tool":tool,"manager_timeout_claim":false}),
            );
            assert!(error.to_string().contains("timeout"), "{error:?}");
            assert_fault_quiescent(&root, &bin, label, &tool).await;

            let label = "ambiguous";
            let tool_file = fault_scenario(&root, &workspace, &nonce, label, 1);
            request.run_id = "run_native0156_ambiguous".into();
            request.prompt = format!("{nonce}:F:{label} Perform the bounded command.");
            let response = manager.submit_turn(request.clone()).await.unwrap();
            // The actual subsequent upstream request must carry this call/result.
            ready(&gateway_root.join("fault-ambiguous-oracle.json")).await;
            let reported = ready(&tool_file).await;
            tokio::time::sleep(Duration::from_secs(2)).await;
            let span = active.active_turn_span_id.lock().unwrap().clone();
            let cancelled = span
                .as_deref()
                .map(|span| manager.cancel(span, "TERM").unwrap())
                .unwrap_or(false);
            let result = frames(
                &manager,
                response,
                span.as_deref().unwrap_or("already-finished"),
                &root,
                "ambiguous",
            )
            .await;
            assert!(
                !result.iter().any(|f| f["type"] == "exit" && f["exit"] == 0),
                "truncated completion cannot qualify success"
            );
            wait_turn_free(active.as_ref()).await;
            let markers = fs::read_to_string(workspace.join("fault-ambiguous-markers")).unwrap();
            assert!(!markers.is_empty());
            let requests = fs::read_dir(&gateway_root)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("request-"))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>();
            write_json(
                root.join("ambiguous-result.json"),
                &json!({"frames":result,"already_executed_tool":reported,
                "marker_bytes":markers,"all_upstream_request_files":requests,"abandonment_cancel":cancelled,
                "fixture_automatic_replay":false,"stock_native_retries":"unmodified; see every retained request",
                "exactly_once_claim":false,"remote_failure_or_non_submission_claim":false}),
            );
            let observed_tools: Vec<Process> =
                serde_json::from_slice(&fs::read(root.join("observed-processes.json")).unwrap())
                    .unwrap();
            let tool = observed_tools
                .into_iter()
                .find(|p| {
                    p.pid == reported["pid"].as_u64().unwrap() as u32
                        && p.start == reported["start"].as_str().unwrap()
                })
                .expect("actual ambiguous tool observed");
            assert_fault_quiescent(&root, &bin, label, &tool).await;
            assert_eq!(active.uaa_session_id().as_deref(), Some(uuid.as_str()));
            write_json(
                root.join("fault-oracles-passed.json"),
                &json!({"missing_regular_attach":true,"missing_exact_uuid":true,
                "explicit_cancel":true,"test_deadline_then_cancel":true,"separate_uaa_request_timeout":true,
                "disconnect_background_custody_then_cancel":true,"ambiguous_completion_stock_retries":true}),
            );
        }
        assert!(lane == "history" || lane == "faults");
        manager.unregister_member(&request.participant_id);
        assert!(!active.launcher_dir.exists());
        assert!(!Path::new(&retained_home).exists());
        assert!(active.active_turn_span_id.lock().unwrap().is_none());
        assert!(active.bootstrap.lock().unwrap().is_none());
        assert!(manager.active_turns_by_span_id.read().unwrap().is_empty());
        drop(active);
        drop(gateway);
        let shutdown: serde_json::Value =
            serde_json::from_slice(&fs::read(gateway_root.join("worker-shutdown.json")).unwrap())
                .unwrap();
        assert_eq!(shutdown["success"], true);
        let gateway_port = gateway_ready["url"]
            .as_str()
            .unwrap()
            .trim_start_matches("http://")
            .trim_end_matches("/v1");
        assert!(
            std::net::TcpStream::connect(gateway_port).is_err(),
            "gateway fixture socket remains open"
        );
        assert!(
            std::net::TcpStream::connect((
                std::net::Ipv4Addr::LOCALHOST,
                gateway_ready["tls_port"].as_u64().unwrap() as u16
            ))
            .is_err(),
            "TLS responder socket remains open"
        );
        write_json(
            root.join("fixture-sockets-closed.json"),
            &json!({"gateway":gateway_port,"tls_port":gateway_ready["tls_port"],"gateway_worker_success":true,"connections_rejected_after_shutdown":true}),
        );
        let observed = observer.finish();
        let work = observed
            .iter()
            .filter(|p| {
                p.ppid == std::process::id()
                    && p.exe == bin.join("codex").display().to_string()
                    && p.argv.iter().any(|a| a == "exec")
            })
            .collect::<Vec<_>>();
        let other_native_images = observed
            .iter()
            .filter(|p| {
                p.ppid != std::process::id() && p.exe == bin.join("codex").display().to_string()
            })
            .collect::<Vec<_>>();
        write_json(
            root.join("native-process-classification.json"),
            &json!({"manager_worker_pid":std::process::id(),"direct_uaa_exec_processes":work,"other_observed_native_images":other_native_images,"rule":"direct manager-worker parent plus actual Codex executable and exec argv identifies UAA work, not an inherited descendant image"}),
        );
        if lane == "history" {
            assert_eq!(
                work.len(),
                3,
                "distinct work process/starttime identities: {work:?}"
            );
        } else {
            assert!(
                work.len() >= 9,
                "real history and fault work required: {work:?}"
            );
        }
        let mut tools = Vec::new();
        for turn in 1..=3 {
            let reported: serde_json::Value = serde_json::from_slice(
                &fs::read(workspace.join(format!("tool-{turn}.json"))).unwrap(),
            )
            .unwrap();
            let tool = observed
                .iter()
                .find(|p| {
                    p.pid == reported["pid"].as_u64().unwrap() as u32
                        && p.start == reported["start"].as_str().unwrap()
                })
                .expect("literal history tool identity must be observed");
            assert_eq!(tool.exe, reported["exe"].as_str().unwrap());
            tools.push(tool);
        }
        assert_eq!(
            tools
                .iter()
                .map(|p| (p.pid, &p.start))
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        for p in &work {
            let startup: serde_json::Value = serde_json::from_slice(
                &fs::read(root.join(format!(
                    "observed-native-startup-{}-{}.json",
                    p.pid, p.start
                )))
                .unwrap(),
            )
            .unwrap();
            assert_eq!(startup["environment"]["HOME"], json!(root.join("home")));
            assert_eq!(startup["environment"]["CODEX_HOME"], json!(retained_home));
            assert_eq!(startup["actual_proc_cwd"], json!(workspace));
            assert!(startup["config"]
                .as_str()
                .unwrap()
                .contains("name = \"substrate-native-protocol-fixture\""));
        }
        let probes = observed
            .iter()
            .filter(|p| {
                p.exe == bin.join("codex").display().to_string()
                    && !p.argv.iter().any(|a| a == "exec")
            })
            .collect::<Vec<_>>();
        // Sampling classifies observed capability probes separately from work;
        // it does not promise exhaustive capture of every short-lived process.
        write_json(
            root.join("normal-uaa-probe-observations.json"),
            &json!({"observed":probes,"exhaustive":false,"separate_native_version_campaign":false}),
        );
        write_json(root.join("observed-exit-checks.json"), &json!(observed.iter().map(|p| json!({"process":p,"same_identity_exists_including_zombie":is_alive(p),"identity_after":process_identity(p.pid)})).collect::<Vec<_>>()));
        for p in &observed {
            assert!(
                !is_alive(p),
                "observed PID/start identity still exists, including zombie: {p:?}; current {:?}",
                process_identity(p.pid)
            );
        }
        assert!(!gateway_root.join("fixture-rescue-required").exists());
        let helper = observed
            .iter()
            .filter(|p| p.exe == bin.join("codex-code-mode-host").display().to_string())
            .collect::<Vec<_>>();
        let helper_starts = helper
            .iter()
            .map(|p| {
                serde_json::from_slice::<Process>(
                    &fs::read(root.join(format!(
                        "observed-helper-startup-{}-{}.json",
                        p.pid, p.start
                    )))
                    .unwrap(),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let history_work = tools
            .iter()
            .map(|tool| {
                let native = work
                    .iter()
                    .find(|native| native.pid == tool.ppid)
                    .expect("actual history tool must belong to observed native work");
                assert!(
                    helper_starts.iter().any(|helper| helper.ppid == native.pid),
                    "each history work process must launch its stock helper"
                );
                *native
            })
            .collect::<Vec<_>>();
        assert_eq!(
            history_work
                .iter()
                .map(|p| (p.pid, &p.start))
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        assert!(
            helper.len() >= 3,
            "external stock helper coverage is required by this profile"
        );
        if lane == "history" {
            assert_eq!(helper.len(), 3);
        }
        let source_pair_after = check_pair(&package);
        assert_eq!(source_pair_before, source_pair_after);
        write_json(
            root.join("retained-source-pair-after.json"),
            &source_pair_after,
        );
        let pair_after = check_pair(&bin);
        assert_eq!(pair_before, pair_after);
        write_json(root.join("verified-subject-after.json"), &pair_after);
        write_json(
            root.join("qualification-passed.json"),
            &json!({"lane":lane,"identity_regression":identity_regression,"uuid":uuid,"turns":3,"exact_resumes":2,"native_exec_processes":work,"history_work_processes":history_work,"observed_helper_processes":helper,"helper_startup_identities":helper_starts,"history_tool_processes":tools,"normal_uaa_probes":probes,"external_helper_coverage":!helper.is_empty(),"cleanup":"all observed PID/start identities absent including zombies, without rescue; no whole-descendant guarantee","scope":"ordinary manager component; regular attach surrogate only"}),
        );
    }
    /// Explicit one-turn live operation. Gateway credentials never enter this worker.
    #[tokio::test]
    #[ignore = "explicit live-account authorization required; one initial text-only turn"]
    async fn codex0156_live_text_manager_worker() {
        use hyper::body::HttpBody;
        let root = PathBuf::from(std::env::var_os("SUBSTRATE_LIVE_TEXT_ROOT").unwrap());
        let package =
            PathBuf::from(std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_PACKAGE_DIR").unwrap());
        let source_pair = check_pair(&package);
        let bin = root.join("package/bin");
        fs::create_dir_all(&bin).unwrap();
        for name in ["codex", "codex-code-mode-host"] {
            fs::copy(package.join(name), bin.join(name)).unwrap();
            fs::set_permissions(bin.join(name), fs::Permissions::from_mode(0o700)).unwrap();
        }
        write_json(
            root.join("verified-official-pair.json"),
            &json!({"source":source_pair,"private":check_pair(&bin)}),
        );
        let observer = Observer::start(root.clone());
        let workspace = root.join("workspace");
        let seed = root.join("seed");
        let cgroup = root.join("regular-attach-surrogate");
        for path in [&workspace, &seed, &cgroup] {
            fs::create_dir(path).unwrap();
        }
        fs::write(cgroup.join("cgroup.procs"), "").unwrap();
        fs::write(seed.join("auth.json"), "{}").unwrap();
        let ready: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("gateway-ready.json")).unwrap()).unwrap();
        fs::write(seed.join("config.toml"), format!("model = \"gpt-6-astra\"\nmodel_provider = \"substrate-live-text\"\n[model_providers.substrate-live-text]\nbase_url = {:?}\n", ready["url"].as_str().unwrap())).unwrap();
        let phrase = format!(
            "SUBSTRATE-LIVE-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        );
        write_json(
            root.join("intended-request.json"),
            &json!({"phrase":phrase,"model":"gpt-6-astra","initial_turns":1,"resumes":0,"tools_requested":false}),
        );
        let mut dispatch = sample_authority_managed_dispatch();
        dispatch.participant_id = "ash_member".into();
        dispatch.retained_worker_launch_authority = None;
        dispatch.e2_launch_activation = None;
        dispatch.resolved_runtime.binary_path = bin.join("codex").display().to_string();
        dispatch.initial_prompt = Some(format!("Reply with exactly {phrase} and nothing else. Do not use any tools. Do not read, create, edit, or otherwise operate on any files. This is a text-only connectivity check."));
        let manager = MemberRuntimeManager::new();
        let env = HashMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.into(),
            seed.display().to_string(),
        )]);
        let span = "spn_live_text_only_20261026";
        let response = manager
            .launch(
                "live-text-only".into(),
                env,
                span.into(),
                MemberRuntimeLaunchAdmissionV1 {
                    ordinary_effect: None,
                    dispatch,
                    acceptance_context: None,
                    policy_snapshot: sample_e2_submit_turn_request()
                        .policy_snapshot_carrier
                        .unwrap()
                        .policy_snapshot()
                        .unwrap(),
                },
                sample_world_binding(),
                LinuxWorldPlacementContext {
                    working_dir: workspace.clone(),
                    cgroup_path: cgroup,
                    require_cgroup_attach: true,
                },
            )
            .await
            .expect("actual manager initial launch");
        let mut body = response.into_body();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
        let mut bytes = Vec::new();
        let mut parsed = 0;
        let mut unexpected_tool = false;
        let mut deadline_expired = false;
        let mut cancelled_on_error = false;
        let mut cancel_accepted = false;
        loop {
            match tokio::time::timeout_at(deadline, body.data()).await {
                Ok(Some(Ok(chunk))) => {
                    bytes.extend_from_slice(&chunk);
                    while let Some(end) = bytes[parsed..].iter().position(|b| *b == b'\n') {
                        let line = &bytes[parsed..parsed + end];
                        parsed += end + 1;
                        if line.is_empty() {
                            continue;
                        }
                        let value: serde_json::Value =
                            serde_json::from_slice(line).expect("typed manager frame");
                        unexpected_tool |= live_text_has_tool(&value);
                        if !cancelled_on_error && live_text_should_cancel(&value) {
                            cancelled_on_error = true;
                            cancel_accepted = manager.cancel(span, "TERM").unwrap_or(false);
                        }
                    }
                }
                Ok(None) => break,
                Ok(Some(Err(_))) => break,
                Err(_) => {
                    deadline_expired = true;
                    let _ = manager.cancel(span, "TERM");
                    break;
                }
            }
        }
        drop(body);
        fs::write(root.join("live-manager-frames.jsonl"), &bytes).unwrap();
        let frames: Vec<serde_json::Value> = String::from_utf8(bytes)
            .unwrap()
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        // Failed bootstrap may already have removed the authoritative member slot.
        // Preserve failure receipts and cleanup rather than expecting retention.
        let active = manager
            .find_submit_target(&sample_submit_turn_request())
            .ok();
        if let Some(active) = &active {
            wait_turn_free(active).await;
        }
        let conversation = active.as_ref().and_then(|active| active.uaa_session_id());
        manager.unregister_member("ash_member");
        drop(active);
        drop(manager);
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            let observed: Vec<Process> =
                serde_json::from_slice(&fs::read(root.join("observed-processes.json")).unwrap())
                    .unwrap();
            if observed.iter().all(|p| !is_alive(p)) || Instant::now() >= end {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        let observed = observer.finish();
        let survivors: Vec<_> = observed.iter().filter(|p| is_alive(p)).collect();
        let output_files = fs::read_dir(root.join("tmp"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("codex_last_message_")
            })
            .map(|e| (e.path(), fs::read_to_string(e.path()).unwrap()))
            .collect::<Vec<_>>();
        let success = !unexpected_tool
            && !deadline_expired
            && !cancelled_on_error
            && frames
                .iter()
                .any(|frame| live_text_has_reply(frame, &phrase))
            && survivors.is_empty()
            && frames.iter().any(|f| f["type"] == "exit" && f["exit"] == 0)
            && output_files.iter().any(|(_, text)| text == &phrase);
        write_json(
            root.join("live-result.json"),
            &json!({"success":success,"expected_reply":phrase,"conversation_id":conversation,"deadline_expired":deadline_expired,"reply_deadline_seconds":90,"cancelled_on_error":cancelled_on_error,"cancel_accepted":cancel_accepted,"unexpected_tool":unexpected_tool,"frames":frames,"last_messages":output_files,"observed_processes":observed,"survivors":survivors,"official_pair_after":check_pair(&bin),"workspace_entries":fs::read_dir(&workspace).unwrap().count(),"kernel_membership_proved":false,"whole_descendant_claim":false}),
        );
        assert!(
            success,
            "live text operation failed; inspect private sanitized receipt, no replay"
        );
    }

    fn live_text_has_tool(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(map) => map.iter().any(|(key, value)| {
                ((key == "kind" || key == "type" || key == "item_type")
                    && value.as_str().is_some_and(|s| {
                        matches!(
                            s,
                            "tool_call"
                                | "tool_result"
                                | "function_call"
                                | "command_execution"
                                | "file_change"
                                | "mcp_tool_call"
                                | "web_search"
                        )
                    }))
                    || live_text_has_tool(value)
            }),
            serde_json::Value::Array(values) => values.iter().any(live_text_has_tool),
            _ => false,
        }
    }

    fn live_text_has_reply(value: &serde_json::Value, phrase: &str) -> bool {
        value["type"] == "event"
            && value["event"]["kind"] == "task_progress"
            && value["event"]["data"]["message"].as_str() == Some(phrase)
    }

    #[test]
    fn test_live_text_requires_assistant_progress_not_status_echo() {
        let phrase = "SUBSTRATE-REPLY";
        assert!(live_text_has_reply(
            &json!({"type":"event","event":{"kind":"task_progress","data":{"message":phrase}}}),
            phrase
        ));
        assert!(!live_text_has_reply(
            &json!({"type":"event","event":{"kind":"status","data":{"message":phrase}}}),
            phrase
        ));
        assert!(!live_text_has_reply(
            &json!({"type":"event","event":{"kind":"task_progress","data":{"message":"partial"}}}),
            phrase
        ));
    }

    fn live_text_should_cancel(value: &serde_json::Value) -> bool {
        live_text_has_tool(value) || value["type"] == "error" || value["event"]["kind"] == "alert"
    }

    #[test]
    fn test_live_text_diagnostic_error_gate_requests_cancel() {
        assert!(live_text_should_cancel(
            &json!({"type":"event","event":{"kind":"alert","data":{"message":"redacted"}}})
        ));
        assert!(live_text_should_cancel(&json!({"type":"error"})));
        assert!(live_text_should_cancel(
            &json!({"event":{"data":{"type":"command_execution"}}})
        ));
        assert!(!live_text_should_cancel(
            &json!({"type":"event","event":{"kind":"status"}})
        ));
    }

    #[test]
    fn test_live_text_tool_observer_rejects_tool_activity() {
        assert!(live_text_has_tool(
            &json!({"event":{"data":{"uaa_event":{"item":{"type":"command_execution"}}}}})
        ));
        assert!(live_text_has_tool(&json!({"event":{"kind":"tool_call"}})));
        assert!(!live_text_has_tool(
            &json!({"event":{"kind":"text_output","text":"no tools"}})
        ));
    }

    /// Actual SDK controls with a synthetic CLI, not service admission, an
    /// official native binary, or an early/partial-startup cleanup guarantee.
    #[tokio::test(flavor = "current_thread")]
    async fn test_ordinary_native_sdk_cancellation_retains_history_until_reaped() {
        for harness in [
            transport_api_types::AgentHarnessFamilyV1::Codex,
            transport_api_types::AgentHarnessFamilyV1::ClaudeCode,
        ] {
            let temp = tempfile::tempdir().unwrap();
            let launcher = temp.path().join("launcher");
            fs::create_dir(&launcher).unwrap();
            fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
            let (selected, start, runtime) = ordinary_worker_component_fixture(harness);
            let history = Arc::new(
                OrdinaryNativeWorkerHistoryV1::create(&launcher, &start, &runtime, &selected)
                    .unwrap(),
            );
            let wiring = transport_api_types::GatewayClientWiringV1 {
                openai_base_url: "http://127.0.0.1:32123".into(),
                anthropic_base_url: "http://127.0.0.1:32123".into(),
            };
            let (name, config) = ordinary_native_config(temp.path(), &selected, &wiring).unwrap();
            history.write_config(name, &config).unwrap();
            let binary = temp.path().join("synthetic-native.py");
            fs::write(&binary, r##"#!/usr/bin/python3
import json, os, pathlib, sys, time
args = sys.argv[1:]
if '--version' in args:
    print('codex 1.2.3'); sys.exit(0)
if '--help' in args:
    print('Usage: --add-dir --allow-dangerously-skip-permissions'); sys.exit(0)
if args and args[0] == 'features':
    print('{"features":["add_dir"]}'); sys.exit(0)
home = pathlib.Path(os.environ['HOME'])
turn = os.environ['FIXTURE_TURN']
is_codex = os.environ['FIXTURE_BACKEND'] == 'codex'
if is_codex:
    sys.stdin.read()  # The supported control test begins after prompt stdin/setup.
if turn != '0':
    assert 'thread-sdk-component' in args
    assert ('resume' if is_codex else '--resume') in args
else:
    assert 'resume' not in args and '--resume' not in args
with (home / 'sdk-turns').open('a') as stream:
    stream.write(turn + '\n')
marker = pathlib.Path(os.environ['FIXTURE_MARKER'])
temporary = marker.with_suffix('.tmp')
temporary.write_text(json.dumps({'pid': os.getpid(), 'args': args, 'home': str(home)}))
temporary.rename(marker)
if is_codex:
    print('{"type":"thread.started","thread_id":"thread-sdk-component"}', flush=True)
else:
    print('{"type":"system","subtype":"init","session_id":"thread-sdk-component"}', flush=True)
if turn == '0':
    gate = pathlib.Path(os.environ['FIXTURE_GATE'])
    deadline = time.monotonic() + 10
    while not gate.exists() and time.monotonic() < deadline:
        time.sleep(0.005)
    assert gate.exists(), 'fixture completion gate timed out'
    if is_codex:
        print('{"type":"turn.completed","thread_id":"thread-sdk-component"}')
    else:
        print('{"type":"result","subtype":"success","is_error":false,"result":"component","session_id":"thread-sdk-component","duration_ms":1,"duration_api_ms":1,"num_turns":1,"total_cost_usd":0}')
else:
    time.sleep(10)  # Finite fallback only; supported cancellation must finish first.
"##).unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
            let backend = if harness == transport_api_types::AgentHarnessFamilyV1::Codex {
                MemberRuntimeBackendKindV1::Codex
            } else {
                MemberRuntimeBackendKindV1::ClaudeCode
            };
            let bridge = PromptFulfillmentBridge::for_member_backend(&backend, binary).unwrap();
            let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
            exclusion.finish_recovery().unwrap();
            let mut identities = BTreeSet::new();
            for turn in 0..3 {
                let marker = temp.path().join(format!("started-{turn}.json"));
                let gate = temp.path().join(format!("complete-{turn}"));
                let mut env = BTreeMap::from([
                    ("FIXTURE_MARKER".into(), marker.display().to_string()),
                    ("FIXTURE_GATE".into(), gate.display().to_string()),
                    ("FIXTURE_TURN".into(), turn.to_string()),
                    (
                        "FIXTURE_BACKEND".into(),
                        if harness == transport_api_types::AgentHarnessFamilyV1::Codex {
                            "codex"
                        } else {
                            "claude"
                        }
                        .into(),
                    ),
                ]);
                configure_ordinary_native_env(
                    &mut env,
                    &selected,
                    &history.descriptor_path().unwrap(),
                    &wiring,
                )
                .unwrap();
                let mut extensions =
                    member_runtime_workspace_access_extensions(backend, temp.path());
                extensions.insert(
                    "agent_api.config.model.v1".into(),
                    json!(selected.routed_model_id),
                );
                if turn != 0 {
                    extensions.insert(
                        SESSION_RESUME_EXTENSION_V1.into(),
                        json!({"selector":"id","id":"thread-sdk-component"}),
                    );
                }
                let control = bridge
                    .run_control(AgentWrapperRunRequest {
                        prompt: format!("synthetic turn {turn}"),
                        working_dir: Some(temp.path().to_path_buf()),
                        timeout: None,
                        env,
                        extensions,
                    })
                    .await
                    .unwrap();
                let ready = tokio::time::timeout(Duration::from_secs(5), async {
                    while !marker.exists() {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                })
                .await;
                if ready.is_err() {
                    control.cancel.cancel();
                    let _ = tokio::time::timeout(Duration::from_secs(5), control.handle.completion)
                        .await;
                    panic!("synthetic native did not reach post-stdin setup: {harness:?}/{turn}");
                }
                let observed: serde_json::Value =
                    serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
                let pid = observed["pid"].as_u64().unwrap() as u32;
                let identity =
                    process_identity(pid).expect("observe actual SDK child before cancellation");
                assert!(identities.insert((pid, identity.start.clone())));
                assert_eq!(
                    observed["home"],
                    history.descriptor_path().unwrap().display().to_string()
                );
                let mut events = control.handle.events;
                let registered = tokio::time::timeout(Duration::from_secs(5), async {
                    while let Some(event) = events.next().await {
                        if surfaced_uaa_session_id_from_data(event.data.as_ref()).as_deref()
                            == Some("thread-sdk-component")
                        {
                            return true;
                        }
                    }
                    false
                })
                .await;
                if !matches!(registered, Ok(true)) {
                    control.cancel.cancel();
                    let _ = tokio::time::timeout(Duration::from_secs(5), control.handle.completion)
                        .await;
                    panic!("SDK startup did not surface the exact universal session event: {harness:?}/{turn}");
                }
                let policy_path = launcher.join(format!("turn-policy-{turn}.sh"));
                fs::write(&policy_path, "synthetic owned launcher").unwrap();
                let policy = PreparedSubmittedTurnPolicyLauncher {
                    path: policy_path.clone(),
                };
                let lease = Arc::new(exclusion.acquire_non_e3_child().unwrap());
                let custody = Arc::new(());
                let weak = Arc::downgrade(&custody);
                let terminal_confirmed = Arc::new(AtomicBool::new(false));
                let checked_terminal = Arc::clone(&terminal_confirmed);
                let terminal_start = identity.start.clone();
                let checked: agent_api::DynAgentWrapperCompletion = Box::pin(async move {
                    let result = control.handle.completion.await;
                    assert!(process_identity(pid).is_none_or(|now| now.start != terminal_start), "SDK terminal observation must reap its actual PID/start identity, including zombies");
                    checked_terminal.store(true, Ordering::Release);
                    result
                });
                let cancel = control.cancel;
                let terminate = cancel.clone();
                let completion = retain_ordinary_world_completion(
                    checked,
                    (Arc::clone(&history), lease, policy, custody),
                    move || terminate.cancel(),
                );
                let events = tokio::spawn(async move { while events.next().await.is_some() {} });
                assert!(weak.upgrade().is_some());
                assert!(policy_path.is_file());
                assert!(exclusion.acquire_e3_exclusive("sdk-component", 7).is_err());
                if turn == 0 {
                    fs::write(&gate, "complete").unwrap();
                    let result = tokio::time::timeout(Duration::from_secs(5), completion)
                        .await
                        .unwrap()
                        .unwrap();
                    assert!(result.status.success());
                } else if turn == 1 {
                    cancel.cancel();
                    let result = tokio::time::timeout(Duration::from_secs(5), completion)
                        .await
                        .unwrap();
                    assert!(
                        result
                            .as_ref()
                            .map_or(true, |value| !value.status.success()),
                        "cancelled synthetic child cannot report success"
                    );
                } else {
                    drop(completion);
                    tokio::time::timeout(Duration::from_secs(5), async {
                        while weak.upgrade().is_some() {
                            tokio::time::sleep(Duration::from_millis(5)).await;
                        }
                    })
                    .await
                    .unwrap();
                }
                tokio::time::timeout(Duration::from_secs(5), events)
                    .await
                    .unwrap()
                    .unwrap();
                assert!(
                    terminal_confirmed.load(Ordering::Acquire),
                    "custody release must follow confirmed SDK terminal/reap observation, not a supervisor panic"
                );
                assert!(weak.upgrade().is_none());
                assert!(!policy_path.exists());
                assert!(process_identity(pid).is_none_or(|now| now.start != identity.start));
                history.revalidate().unwrap();
                let mut exclusive = exclusion.acquire_e3_exclusive("sdk-component", 7).unwrap();
                exclusive.release_after_cleanup_v1().unwrap();
                eprintln!("actual SDK synthetic lifecycle: {harness:?} turn={turn} pid={pid} terminal/reaped/custody released");
            }
            assert_eq!(identities.len(), 3);
            assert_eq!(
                fs::read_to_string(history.path.join("sdk-turns")).unwrap(),
                "0\n1\n2\n"
            );
            history.revalidate().unwrap();
        }
    }

    /// Public UAA cancellation during the inner SDK's blocked prompt write.
    /// Eventual absence is observed under a live Tokio runtime; this does not
    /// turn the returned cancellation result into a child-wait acknowledgement.
    #[tokio::test(flavor = "current_thread")]
    async fn test_ordinary_codex_sdk_startup_cancel_observes_reaping() {
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("startup.json");
        let binary = temp.path().join("synthetic-blocked-stdin.py");
        fs::write(
            &binary,
            r##"#!/usr/bin/python3
import fcntl, json, os, pathlib, sys, time
args = sys.argv[1:]
if '--version' in args:
    print('codex 1.2.3'); sys.exit(0)
if '--help' in args:
    print('Usage: --add-dir'); sys.exit(0)
if args and args[0] == 'features':
    print('{"features":["add_dir"]}'); sys.exit(0)
# No stdin read and no session event: inner ExecStreamControl cannot finish setup.
marker = pathlib.Path(os.environ['FIXTURE_MARKER'])
temporary = marker.with_suffix('.tmp')
temporary.write_text(json.dumps({'pid': os.getpid(), 'stdin_pipe_capacity': fcntl.fcntl(0, fcntl.F_GETPIPE_SZ)}))
temporary.rename(marker)
time.sleep(10)  # Finite fallback; the test requires cancellation cleanup sooner.
"##,
        )
        .unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        let (selected, start, runtime) =
            ordinary_worker_component_fixture(transport_api_types::AgentHarnessFamilyV1::Codex);
        let launcher = temp.path().join("launcher");
        fs::create_dir(&launcher).unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
        let history =
            OrdinaryNativeWorkerHistoryV1::create(&launcher, &start, &runtime, &selected).unwrap();
        let wiring = transport_api_types::GatewayClientWiringV1 {
            openai_base_url: "http://127.0.0.1:32123".into(),
            anthropic_base_url: "http://127.0.0.1:32123".into(),
        };
        let (name, config) = ordinary_native_config(temp.path(), &selected, &wiring).unwrap();
        history.write_config(name, &config).unwrap();
        let mut env = BTreeMap::from([("FIXTURE_MARKER".into(), marker.display().to_string())]);
        configure_ordinary_native_env(
            &mut env,
            &selected,
            &history.descriptor_path().unwrap(),
            &wiring,
        )
        .unwrap();
        let bridge =
            PromptFulfillmentBridge::for_member_backend(&MemberRuntimeBackendKindV1::Codex, binary)
                .unwrap();
        // Linux pipes cannot absorb this entire prompt while the child never reads.
        let control = bridge
            .run_control(AgentWrapperRunRequest {
                prompt: "x".repeat(2 * 1024 * 1024),
                working_dir: Some(temp.path().to_path_buf()),
                timeout: None,
                env,
                extensions: BTreeMap::new(),
            })
            .await
            .unwrap();
        let ready = tokio::time::timeout(Duration::from_secs(5), async {
            while !marker.exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;
        if ready.is_err() {
            control.cancel.cancel();
            let _ = tokio::time::timeout(Duration::from_secs(5), control.handle.completion).await;
            panic!("synthetic Codex did not reach blocked prompt stdin");
        }
        let observed: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
        assert!(observed["stdin_pipe_capacity"].as_u64().unwrap() < 2 * 1024 * 1024);
        let pid = observed["pid"].as_u64().unwrap() as u32;
        let identity = process_identity(pid).expect("observe blocked-startup child");
        let mut events = control.handle.events;
        let events = tokio::spawn(async move {
            assert!(
                events.next().await.is_none(),
                "blocked-stdin startup must emit no universal event before cancellation"
            );
        });
        eprintln!(
            "startup child before public cancel: {}",
            json!({"pid":pid,"start":identity.start,"state":identity.state,"stdin_pipe_capacity":observed["stdin_pipe_capacity"]})
        );
        control.cancel.cancel();
        let result = tokio::time::timeout(Duration::from_secs(5), control.handle.completion)
            .await
            .unwrap();
        assert!(
            matches!(&result, Err(AgentWrapperError::Backend { message }) if message == "cancelled"),
            "startup cancellation must return the pinned cancellation error"
        );
        let at_cancellation_return = process_identity(pid);
        tokio::time::timeout(Duration::from_secs(5), async {
            while process_identity(pid).is_some_and(|now| now.start == identity.start) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("observe actual startup-child reaping, including zombies, without rescue");
        tokio::time::timeout(Duration::from_secs(1), events)
            .await
            .unwrap()
            .unwrap();
        history.revalidate().unwrap();
        eprintln!(
            "startup cancellation observation: {}",
            json!({
                "pid":pid,"start":identity.start,"state_before_cancel":identity.state,
                "identity_at_cancel_completion":at_cancellation_return,
                "identity_after_observer_wait":process_identity(pid),
                "returned_universal_control":true,"inner_sdk_setup_blocked":true,
                "wait_acknowledgement":false,"rescue_signals":0,
                "scope":"synthetic Codex blocked-stdin startup; live Tokio runtime observation only"
            })
        );
    }
}
