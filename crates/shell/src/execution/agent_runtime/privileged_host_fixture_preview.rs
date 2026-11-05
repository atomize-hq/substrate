// Inert until the named ignored test is explicitly executed inside the owned fixture.
// No arbitrary-path acquisition, protected constructor or authority factory.
use super::super::host_session_authority::{schema::*, transition::*};
use super::*;
use std::io::{BufRead, Write};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

fn checkpoint(value: serde_json::Value) -> Result<()> {
    println!("TASK11_FIXTURE {}", serde_json::to_string(&value)?);
    std::io::stdout().flush()?;
    Ok(())
}
fn proceed(expected: &str) -> Result<()> {
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    ensure!(
        line.trim_end() == expected,
        "fixture controller closed or changed phase"
    );
    Ok(())
}
fn private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires inspected private privileged fixture, actual UID1000 actor and root service"]
async fn test_fixed_source_two_identity_host_discriminator() -> Result<()> {
    Box::pin(fixed_source_two_identity_host_fixture(false)).await
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires inspected private native Host fixture, actual UID1000 actor and root service"]
async fn test_fixed_source_two_identity_host_start_and_two_resumes() -> Result<()> {
    Box::pin(fixed_source_two_identity_host_fixture(true)).await
}

async fn fixed_source_two_identity_host_fixture(native: bool) -> Result<()> {
    let mode = if native { "native03" } else { "preview01" };
    ensure!(std::env::var("TASK11_PRIVILEGED_HOST_FIXTURE").as_deref() == Ok(mode));
    ensure!(unsafe { libc::getuid() } == 1000 && unsafe { libc::geteuid() } == 1000);
    ensure!(unsafe { libc::getgid() } == 1000 && unsafe { libc::getegid() } == 1000);
    let status = std::fs::read_to_string("/proc/self/status")?;
    for field in ["CapEff:", "CapPrm:", "CapInh:", "CapAmb:", "CapBnd:"] {
        let value = status
            .lines()
            .find_map(|line| line.strip_prefix(field))
            .context("missing capability field")?;
        ensure!(
            u64::from_str_radix(value.trim(), 16)? == 0,
            "actor capabilities are not zero"
        );
    }
    ensure!(
        unsafe { libc::unshare(libc::CLONE_NEWUSER) } == -1
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM),
        "nested user namespaces must remain unavailable"
    );
    ensure!(
        !Path::new("/home/spenser").exists()
            && !Path::new("/dev/shm/substrate-task11-v3-selection-20261029").exists()
    );
    let home = Path::new("/home/fixture/.substrate");
    let workspace = Path::new("/home/fixture/workspace");
    std::fs::create_dir(home)?;
    std::fs::set_permissions(home, std::fs::Permissions::from_mode(0o700))?;
    std::fs::create_dir(workspace)?;
    std::fs::create_dir(home.join("agents"))?;
    std::fs::set_permissions(home.join("agents"), std::fs::Permissions::from_mode(0o700))?;
    private_file(
        &home.join("config.yaml"),
        b"agents:\n  enabled: true\n  hub:\n    orchestrator_agent_id: codex-host\n",
    )?;
    // Preserve the original inert discriminator; the separately named native case
    // declares only this synthetic CLI and uses the same ordinary production reader.
    let mut inventory: serde_yaml::Value =
        serde_yaml::from_str(include_str!("../../../../../config/agents/codex.yaml"))?;
    let binary = if native {
        "/home/fixture/workspace/codex"
    } else {
        "/bin/true"
    };
    inventory["config"]["placements"]["host"]["cli"]["binary"] = binary.into();
    if native {
        private_file(Path::new(binary), HOST_NATIVE_FIXTURE.as_bytes())?;
        std::fs::set_permissions(binary, std::fs::Permissions::from_mode(0o700))?;
    }
    private_file(
        &home.join("agents/codex.yaml"),
        serde_yaml::to_string(&inventory)?.as_bytes(),
    )?;
    private_file(&home.join("policy.yaml"), b"id: fixed-source-host-discriminator\nagents:\n  allowed_backends: [cli:codex-host]\nllm:\n  allowed_backends: [cli:codex-host]\n  constraints:\n    routers: [substrate_gateway]\n    providers: [openai]\n    protocols: [openai.responses]\n    auth_authorities: [codex_subscription]\n  secrets:\n    env_allowed: [SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN, SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID]\n")?;
    let (principal, _) = crate::execution::install_bootstrap::current_unix_principal_and_home()?;
    ensure!(
        principal
            == PlatformPrincipalV1::Unix {
                account: "fixture".into(),
                uid: 1000
            }
    );
    let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
        transport_api_types::InstallBootstrapContextV1::new_unix(
            home.to_str().unwrap(),
            "fixture",
            1000,
        )
        .map_err(anyhow::Error::msg)?,
    )
    .map_err(anyhow::Error::msg)?;
    // Only bytes/physical path cross to root publication. No held authority exists yet.
    checkpoint(
        serde_json::json!({"phase":"files", "carrier":carrier.encode().map_err(anyhow::Error::msg)?}),
    )?;
    proceed("published")?;
    let ordinary = Arc::new(config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority(&carrier)?);
    ordinary.revalidate()?;
    let verifier = OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(
        Arc::clone(&ordinary),
    )?;
    // Every stateful value below is newly derived after the genuine fixed reader.
    let authority = &verifier.authority;
    let root = authority.bootstrap()?;
    let bootstrap = authority.bootstrap_home();
    let config = crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
        workspace,
        &Default::default(),
        &bootstrap,
    )?;
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        workspace, &bootstrap,
    )?;
    let snapshot = crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
        workspace, &bootstrap,
    )?;
    let inventory =
        crate::execution::agent_inventory::load_agent_inventory_snapshot_for_bootstrap_home_v3(
            workspace, &policy, &bootstrap,
        )?;
    let run_native_lifecycle = native;
    let native = inventory.resolve_selection(&config, None, None, AgentExecutionScope::Host)?;
    let selections = OrdinaryStartSelectionsV1 {
        requested: native.clone(),
        native,
    };
    let sources = CapturedOrdinarySourcesV1::capture(&config, &inventory, &selections)?;
    let info = std::fs::metadata(workspace)?;
    let binding = WorkspaceBindingV1 {
        workspace_root: super::super::host_session_authority::schema::CanonicalDirectoryV1 {
            physical_path: workspace.to_str().unwrap().into(),
            physical_identity:
                super::super::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux {
                    device_id: info.dev(),
                    inode: info.ino(),
                },
        },
        authority_store_root: root.bootstrap_home,
        authority_store_id: root.authority_store_id,
    };
    let descriptor = AgentDescriptorV1 {
        schema_version: 1,
        agent_id: selections.native.realized_agent_id.clone(),
        backend_id: selections.native.backend_id.clone(),
        backend_kind: RuntimeBackendKindV1::Codex,
        protocol: "substrate.agent.session".into(),
        execution_scope: AgentExecutionScopeV1::Host,
        binary_path: binary.into(),
    };
    let policy_input = PolicyObjectHashInputV1 {
        schema_version: 1,
        policy_revision: snapshot.snapshot_hash.clone(),
        canonical_policy_snapshot_sha256: snapshot.snapshot_hash.clone(),
    };
    let capabilities = HostAttachCapabilitiesV1 {
        session_resume: true,
        session_fork: true,
        session_stop: true,
        status_snapshot: true,
        event_stream: true,
    };
    let launch_knobs = HostAttachLaunchKnobsV1 {
        requested_execution_scope: AgentExecutionScopeV1::Host,
        host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
        attach_mode_preference: HostAttachModePreferenceV1::ContinuityRequired,
    };
    let complete_plan = serde_json::json!({
        "schema_version":1,"operation":"greenfield-host-start","intent_id":"hti_private_fixed_host01",
        "orchestration_session_id":"aos_private_fixed_host01","shell_trace_session_id":"trace_private_fixed_host01",
        "target_authoritative_participant_id":"ash_private_fixed_host01","target_participant_lease_token":"synthetic-fixed-host-lease",
        "run_id":"run_private_fixed_host01","resulting_authoritative_lineage":["ash_private_fixed_host01"],
        "workspace_binding":&binding,"world_binding":null,"descriptor":&descriptor,"policy":&policy_input,
        "capabilities":&capabilities,"launch_knobs":&launch_knobs,
        "claim_id":"clm_private_fixed_host01","claimant_attempt_id":"atm_private_fixed_host01",
    });
    let complete_plan_bytes = canonical_json::to_vec(&complete_plan)?;
    let mut digest = Sha256::new();
    digest.update(b"substrate.ordinary.native-start-plan.v1");
    digest.update([0]);
    digest.update(&complete_plan_bytes);
    let launch_digest = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let revalidate = || {
        crate::execution::orchestrator_world_dispatch::revalidate_ordinary_current_launch_permission(
            &ordinary,
            authority,
            &binding,
            &selections,
            &sources,
            &snapshot.snapshot_hash,
        )
    };
    let fresh_policy = revalidate()?;
    let context = stage_native_start_context_from_ordinary_configured(
        Arc::clone(&ordinary),
        NativeStartContextInputsV1 {
            authority,
            intended_principal: &principal,
            effective_policy: &fresh_policy,
            workspace_binding: &binding,
            world_binding: None,
            intent_id: "hti_private_fixed_host01",
            orchestration_session_id: "aos_private_fixed_host01",
            shell_trace_session_id: "trace_private_fixed_host01",
            participant_id: "ash_private_fixed_host01",
            run_id: "run_private_fixed_host01",
            descriptor: &descriptor,
            opaque_launch_plan_digest: &launch_digest,
            selections: &selections,
            sources: &sources,
        },
    )?;
    let request = IssueHostSessionTransitionRequestV1 {
        intent_id: context.intent_id.clone(),
        issuer_request_id: context.issuer_request_id()?,
        mode: HostSessionTransitionModeV1::Start,
        authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
        orchestration_session_id: context.orchestration_session_id.clone(),
        shell_trace_session_id: context.shell_trace_session_id.clone(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::Repl,
            caller_participant_id: None,
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: None,
        target_authoritative_participant_id: context.original_participant_id.clone(),
        target_participant_lease_token: b"synthetic-fixed-host-lease".to_vec(),
        run_id: context.original_run_id.clone(),
        resulting_authoritative_lineage: vec![context.original_participant_id.clone()],
        workspace_binding: binding.clone(),
        world_binding: None,
        start_contract: StartContractMaterialV1 {
            descriptor,
            policy: policy_input,
            capabilities,
            launch_knobs,
        },
        transition_input: None,
    };
    revalidate()?;
    let issued = authority.issue_start(&request)?;
    let TransitionIssueOutcomeV1::Issued(intent) = issued else {
        anyhow::bail!("fresh Start must actually issue");
    };
    revalidate()?;
    let claimed = authority.claim_start(&ClaimHostSessionTransitionRequestV1 {
        intent_id: intent.intent_id,
        issuer_request_id: intent.issuer_request_id,
        payload_commitment: intent.payload_commitment,
        expected_intent_revision: intent.intent_revision,
        claim_id: "clm_private_fixed_host01".into(),
        claimant_attempt_id: "atm_private_fixed_host01".into(),
    })?;
    let TransitionClaimOutcomeV1::Claimed(intent) = claimed else {
        anyhow::bail!("fresh Start must actually claim");
    };
    let HostSessionTransitionIntentStateV2::Claimed {
        claim_id,
        claim_revision,
        ..
    } = intent.state
    else {
        anyhow::bail!("missing claim");
    };
    revalidate()?;
    let application = authority.apply_start(&ApplyHostSessionTransitionRequestV1 {
        intent_id: intent.intent_id,
        issuer_request_id: intent.issuer_request_id,
        payload_commitment: intent.payload_commitment,
        expected_intent_revision: claim_revision,
        claim_id,
        expected_claim_revision: claim_revision,
    })?;
    ensure!(
        matches!(&application, TransitionApplicationOutcomeV1::Applied(_)),
        "fresh Start must actually apply"
    );
    let claim = transport_api_types::OrdinaryGatewaySelectionContextV1 {
        schema_version: 1,
        start: transport_api_types::OrdinaryStartSelectionCarrierV1 {
            schema_version: 1,
            authority_store_id: context.workspace_binding.authority_store_id.clone(),
            original_start_intent_id: context.intent_id.clone(),
            original_start_issuer_request_id: context.issuer_request_id()?,
            original_session_id: context.orchestration_session_id.clone(),
            context_sha256: context.context_sha256()?,
            native_selection_sha256: context.selections.native.selection_digest.clone(),
            requested_selection_sha256: context.selections.requested.selection_digest.clone(),
        },
        runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor {
            orchestration_session_id: context.orchestration_session_id.clone(),
            participant_id: context.original_participant_id.clone(),
            backend_id: context.descriptor.backend_id.clone(),
            run_id: context.original_run_id.clone(),
        },
    };
    let admission = verifier.authenticate_host_gateway_v1(&claim)?;
    admission.revalidate_before_effect()?;
    ensure!(
        admission.principal() == &principal
            && admission.workspace_root() == workspace.to_str().unwrap()
    );
    checkpoint(
        serde_json::json!({"phase":"admitted", "context_sha256":context.context_sha256()?, "native_selection":context.selections.native.selection_digest}),
    )?;
    proceed("service")?;
    let client = transport_api_client::AgentClient::unix_socket("/run/substrate.sock")?;
    // Observe is read-only. No Acquire is sent while service startup is unresolved.
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if observe_host_gateway_instance_v3(&client).await.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("authenticated inherited service did not become ready")?;
    if run_native_lifecycle {
        let TransitionApplicationOutcomeV1::Applied(applied) = &application else {
            anyhow::bail!("fresh native Start must actually apply");
        };
        return Box::pin(host_native_lifecycle(&verifier, &context, applied, &claim)).await;
    }
    let acquisition = Arc::new(HostGatewayAcquisitionCustodyV1::new());
    let acquired = tokio::time::timeout(
        Duration::from_secs(30),
        acquire_host_gateway_turn_v3(claim.clone(), Arc::clone(&acquisition)),
    )
    .await;
    let observation: Result<()> = async {
        let (lease, wiring) = acquired.context("Acquire observation timed out; retained owner must settle")??;
        lease.revalidate_before_spawn().await?;
        let endpoint = wiring.openai_base_url.trim_end_matches("/v1").trim_end_matches('/');
        validate_native_loopback_endpoint(endpoint)?;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let address = endpoint.strip_prefix("http://").context("canonical endpoint")?;
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                if let Ok(mut stream) = tokio::net::TcpStream::connect(address).await {
                    stream.write_all(format!("GET /health HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n").as_bytes()).await?;
                    let mut reply = Vec::new();
                    stream.take(65536).read_to_end(&mut reply).await?;
                    if reply.starts_with(b"HTTP/1.1 200 ") { return Ok::<(), anyhow::Error>(()); }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }).await.context("normal foreground gateway never served health")??;
        lease.revalidate_before_spawn().await?;
        checkpoint(serde_json::json!({"phase":"gateway-ready", "native_prompt_sent":false}))?;
        // Held only in RAM. Revocation handle is the actor's policy.yaml and the
        // original/current admissions above; no hook changes their predicates.
        Ok(())
    }.await;
    // Always close the finite production acquisition owner, including timeouts.
    // Only Released proves the service's actual Child terminate_and_wait path.
    let cleanup = tokio::time::timeout(Duration::from_secs(30), acquisition.close_and_wait()).await;
    checkpoint(
        serde_json::json!({"phase":"cleanup", "released":matches!(cleanup, Ok(Ok(HostGatewayCleanupOutcomeV1::Released)))}),
    )?;
    ensure!(
        matches!(cleanup, Ok(Ok(HostGatewayCleanupOutcomeV1::Released))),
        "same-operation Release unresolved; preserve run and no redelivery"
    );
    observation?;
    checkpoint(
        serde_json::json!({"phase":"complete", "gateway_child_wait":"service Released path", "native_prompt_sent":false}),
    )?;
    Ok(())
}

const HOST_NATIVE_FIXTURE: &str = r##"#!/usr/bin/python3
import json, os, pathlib, re, sys, time, urllib.error, urllib.request
args = sys.argv[1:]
if args and args[0] in ('--version', '--help', 'features'):
    print({'--version':'codex 1.2.3', '--help':'Usage: --add-dir --allow-dangerously-skip-permissions',
           'features':'{"features":["add_dir"]}'}[args[0]])
    raise SystemExit(0)
root = pathlib.Path(__file__).resolve().parent
def stage(name):
    fd = os.open(root / 'host-native-stages.jsonl',
                 os.O_WRONLY | os.O_CREAT | os.O_APPEND | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'a') as output:
        output.write(json.dumps(dict(stage=name, pid=os.getpid())) + '\n')
stage('native_entered')
observations = root / 'host-native.jsonl'
prior = [json.loads(line) for line in observations.read_text().splitlines()] if observations.exists() else []
turn = len(prior) + 1
assert turn in (1, 2, 3)
session = 'thread-task11-host-component'
if turn == 1:
    assert 'resume' not in args
else:
    assert args[args.index('resume') + 1] == session
assert '--last' not in args and '--fork-session' not in args
expected_prompt = 'synthetic host turn %s' % turn
assert expected_prompt not in args
output_arg = args.index('--output-last-message')
assert args[output_arg + 2:] == ([] if turn == 1 else ['resume', session, '-'])
stage('argv_validated')
assert sys.stdin.buffer.read() == expected_prompt.encode('utf-8') + b'\n'
stage('prompt_validated')
home = pathlib.Path(os.environ['CODEX_HOME'])
assert str(home).startswith('/proc/')
assert not (home / 'auth.json').exists()
for name in ('OPENAI_API_KEY','SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN',
             'SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID'):
    assert not os.environ.get(name)
assert args[args.index('--model') + 1] == 'codex-mini-latest'
stage('context_validated')
origin = re.search(r'base_url\s*=\s*"(http://127[.]0[.]0[.]1:[0-9]+)', ' '.join(args)).group(1)
cap = os.environ['SUBSTRATE_INTERNAL_CODEX_LOCAL_ROUTE_CAPABILITY']
assert len(cap) == 43
def refused(token):
    headers = {'Content-Type':'application/json'}
    if token is not None: headers['Authorization'] = 'Bearer ' + token
    request = urllib.request.Request(origin + '/v1/responses', data=b'{', headers=headers)
    try:
        urllib.request.urlopen(request, timeout=5)
        raise AssertionError('malformed request unexpectedly accepted')
    except urllib.error.HTTPError as error:
        return error.code
# Route middleware authenticates before Json extraction. None of these calls can
# enter the Responses handler/router/provider. The cap itself is never recorded.
ready = time.monotonic() + 5
while True:
    try:
        missing = refused(None)
        break
    except urllib.error.URLError:
        assert time.monotonic() < ready
        time.sleep(0.02)
assert [missing, refused('wrong'), refused(cap)] == [403, 403, 400]
stage('route_authenticated')
history = home / 'component-host-history'
expected = session + ''.join('\nturn-%s' % n for n in range(1, turn))
if turn == 1:
    with history.open('x') as output: output.write(session)
else:
    assert history.read_text() == expected
before = history.stat()
with history.open('a') as output: output.write('\nturn-%s' % turn)
after = history.stat()
assert (before.st_dev, before.st_ino) == (after.st_dev, after.st_ino)
output = pathlib.Path(args[args.index('--output-last-message') + 1])
assert output.is_absolute() and output.parent == pathlib.Path(os.environ['TMPDIR'])
with output.open('x') as handle: handle.write('synthetic host output %s' % turn)
stage('history_and_output_written')
entry = dict(turn=turn, pid=os.getpid(), parent_pid=os.getppid(), cwd=os.getcwd(),
             home_device=home.stat().st_dev, home_inode=home.stat().st_ino,
             history_device=after.st_dev, history_inode=after.st_ino,
             history=history.read_text(), authenticated_body_refusal=True)
with observations.open('a') as handle: handle.write(json.dumps(entry) + '\n')
for event in [dict(type='thread.started' if turn == 1 else 'thread.resumed', thread_id=session),
              dict(type='turn.started', thread_id=session, turn_id='host-turn-%s' % turn),
              dict(type='turn.completed', thread_id=session, turn_id='host-turn-%s' % turn)]:
    print(json.dumps(event), flush=True)
stage('events_emitted')
deadline = time.monotonic() + 15
while not (root / ('host-release-%s' % turn)).exists():
    assert time.monotonic() < deadline
    time.sleep(0.01)
"##;

struct SafeNativeStreamDiagnostic;

#[derive(Default)]
struct SafeNativeStreamFields(serde_json::Map<String, serde_json::Value>);

impl tracing::field::Visit for SafeNativeStreamFields {
    fn record_debug(&mut self, _: &tracing::field::Field, _: &dyn std::fmt::Debug) {}

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if matches!(
            field.name(),
            "operation" | "stream_error_variant" | "codex_error_variant"
        ) {
            self.0.insert(field.name().into(), value.into());
        }
    }
}

impl tracing::Subscriber for SafeNativeStreamDiagnostic {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.target() == "substrate::native_stream_error"
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, event: &tracing::Event<'_>) {
        let mut fields = SafeNativeStreamFields::default();
        event.record(&mut fields);
        eprintln!(
            "TASK11_SAFE_NATIVE_STREAM_ERROR {}",
            serde_json::Value::Object(fields.0)
        );
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

fn native_timestamp() -> Result<TimestampV1> {
    Ok(TimestampV1::parse(
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
    )?)
}

async fn host_native_lifecycle(
    verifier: &OrdinarySelectionAuthorityV1,
    context: &NativeLaunchContextV1,
    applied: &super::super::host_session_authority::store_schema::HostSessionTransitionIntentV2,
    original_claim: &transport_api_types::OrdinaryGatewaySelectionContextV1,
) -> Result<()> {
    use crate::execution::agent_runtime::control::*;
    use crate::execution::agent_runtime::host_session_authority::{
        start_continuity::*, store_schema::*,
    };
    use crate::execution::prompt_fulfillment::PromptFulfillmentBridge;
    use futures::StreamExt as _;
    use std::os::fd::AsRawFd as _;
    tracing::subscriber::set_global_default(SafeNativeStreamDiagnostic)?;
    let authority = &verifier.authority;
    let workspace = Path::new(&context.workspace_binding.workspace_root.physical_path);
    let descriptor = super::super::validator::RuntimeSelectionDescriptor {
        agent_id: context.descriptor.agent_id.clone(),
        backend_id: context.descriptor.backend_id.clone(),
        backend_kind: super::super::mapping::AgentRuntimeBackendKind::Codex,
        protocol: context.descriptor.protocol.clone(),
        execution_scope: AgentExecutionScope::Host,
        binary_path: context.descriptor.binary_path.clone().into(),
    };
    let HostSessionTransitionIntentStateV2::Applied {
        application_result_ref,
        authority_revision_after,
        ..
    } = &applied.state
    else {
        anyhow::bail!("actual applied Start missing");
    };
    let prompt_hash = format!("{:x}", Sha256::digest(b"synthetic host turn 1"));
    let transaction = StartTransactionRecordV1 {
        schema_version: 1,
        transaction_id: "stx_private_host_native03".into(),
        request_key_sha256: prompt_hash.clone(),
        prompt_sha256: prompt_hash.clone(),
        authority_store_id: context.workspace_binding.authority_store_id.clone(),
        orchestration_session_id: context.orchestration_session_id.clone(),
        shell_trace_session_id: context.shell_trace_session_id.clone(),
        authoritative_participant_id: context.original_participant_id.clone(),
        backend_id: context.descriptor.backend_id.clone(),
        protocol: context.descriptor.protocol.clone(),
        workspace_root: workspace.display().to_string(),
        world_id: None,
        world_generation: None,
        public_backend_id: context.descriptor.backend_id.clone(),
        public_scope: "host".into(),
        start_intent_id: applied.intent_id.clone(),
        start_issuer_request_id: applied.issuer_request_id.clone(),
        start_payload_commitment: applied.payload_commitment.clone(),
        start_application_result_ref: application_result_ref.clone(),
        start_run_id: applied.run_id.clone(),
        start_authority_revision: *authority_revision_after,
        created_at: native_timestamp()?,
        updated_at: native_timestamp()?,
        state: StartTransactionStateV1::PromptNotSubmitted,
    };
    ensure!(matches!(
        authority.begin_start_transaction(&transaction)?,
        StartTransactionBeginOutcomeV1::Applied(_)
    ));
    let current = authority.resolve_current_exact(&context.orchestration_session_id, None)?;
    authority.enroll_direct_host_start(&current)?;
    authority.mark_start_prompt_submission_no_replay_barrier(
        &transaction.transaction_id,
        &transaction.request_key_sha256,
        native_timestamp()?,
    )?;
    let native_session = "thread-task11-host-component";
    let mut resume_handle = None;
    let mut previous_runtime = None;
    let mut pids = std::collections::BTreeSet::new();
    for turn in 1..=3 {
        let prompt = format!("synthetic host turn {turn}");
        let mut successor = None;
        let mut guard = None;
        let previous_guard = std::env::var_os("SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD");
        let (participant, run) = if turn == 1 {
            (
                context.original_participant_id.clone(),
                context.original_run_id.clone(),
            )
        } else {
            let current =
                authority.resolve_current_exact(&context.orchestration_session_id, None)?;
            let participant = format!("ash_private_host_native03_{turn}");
            let mut lineage = current.authority.authoritative_participant_lineage.clone();
            lineage.push(participant.clone());
            let successor_request = IssueSuccessorTransitionRequestV1 {
                intent_id: format!("hti_private_host_native03_{turn}"),
                issuer_request_id: format!("req_private_host_native03_{turn}"),
                mode: HostSessionTransitionModeV1::ResumeOneTurn,
                authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
                    authority_revision: current.observation.authority_revision,
                    authority_record_commitment: current
                        .observation
                        .authority_record_commitment
                        .clone(),
                    active_authoritative_participant_id: current
                        .authority
                        .active_authoritative_participant_id
                        .clone()
                        .context("current participant missing")?,
                    authoritative_lineage_commitment: current
                        .observation
                        .authoritative_lineage_commitment
                        .clone(),
                    lifecycle_posture: current.authority.lifecycle_posture,
                },
                orchestration_session_id: context.orchestration_session_id.clone(),
                shell_trace_session_id: context.shell_trace_session_id.clone(),
                caller: HostSessionTransitionCallerV1 {
                    kind: HostSessionTransitionCallerKindV1::Repl,
                    caller_participant_id: current
                        .authority
                        .active_authoritative_participant_id
                        .clone(),
                    auto_attach_obligation_id: None,
                    auto_attach_claim_owner: None,
                },
                source_authoritative_participant_id: current
                    .authority
                    .active_authoritative_participant_id
                    .clone(),
                target_authoritative_participant_id: participant.clone(),
                target_participant_lease_token: format!("synthetic-host-lease-{turn}").into_bytes(),
                run_id: format!("run_private_host_native03_{turn}"),
                resulting_authoritative_lineage: lineage,
                workspace_binding: context.workspace_binding.clone(),
                world_binding: None,
                resume_handle_ref: resume_handle.take(),
                transition_input: Some(prompt.as_bytes().to_vec()),
                post_turn_disposition: Some(
                    HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal,
                ),
            };
            let SuccessorTransitionIssueOutcomeV1::Issued(issued) =
                authority.issue_successor(&current, &successor_request)?
            else {
                anyhow::bail!("fresh Resume must actually issue");
            };
            let claim_id = format!("clm_private_host_native03_{turn}");
            let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) =
                authority.claim_successor(&ClaimHostSessionTransitionRequestV1 {
                    intent_id: issued.intent_id.clone(),
                    issuer_request_id: issued.issuer_request_id.clone(),
                    payload_commitment: issued.payload_commitment.clone(),
                    expected_intent_revision: issued.intent_revision,
                    claim_id: claim_id.clone(),
                    claimant_attempt_id: format!("atm_private_host_native03_{turn}"),
                })?
            else {
                anyhow::bail!("fresh Resume must actually claim");
            };
            let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = &claimed.state
            else {
                anyhow::bail!("actual Resume claim missing");
            };
            let SuccessorTransitionApplicationOutcomeV1::Applied(applied) = authority
                .apply_successor(&ApplyHostSessionTransitionRequestV1 {
                    intent_id: claimed.intent_id.clone(),
                    issuer_request_id: claimed.issuer_request_id.clone(),
                    payload_commitment: claimed.payload_commitment.clone(),
                    expected_intent_revision: claimed.intent_revision,
                    claim_id,
                    expected_claim_revision: *claim_revision,
                })?
            else {
                anyhow::bail!("fresh Resume must actually apply");
            };
            authority.accept_transition_input(&AcceptTransitionInputRequestV1 {
                intent_id: applied.intent_id.clone(),
                issuer_request_id: applied.issuer_request_id.clone(),
                payload_commitment: applied.payload_commitment.clone(),
                accepting_participant_id: participant.clone(),
                accepted_at: native_timestamp()?,
            })?;
            let guards = Path::new(&context.workspace_binding.authority_store_root.physical_path)
                .join("runtime-control/durable-start/successor-launch-guards");
            let mut component =
                Path::new(&context.workspace_binding.authority_store_root.physical_path)
                    .to_path_buf();
            for name in [
                "runtime-control",
                "durable-start",
                "successor-launch-guards",
            ] {
                component.push(name);
                match std::fs::create_dir(&component) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error.into()),
                }
                std::fs::set_permissions(&component, std::fs::Permissions::from_mode(0o700))?;
            }
            let lock_path = guards.join(format!("{}.lock", applied.intent_id));
            let file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(lock_path)?;
            fs2::FileExt::lock_exclusive(&file)?;
            std::env::set_var(
                "SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD",
                file.as_raw_fd().to_string(),
            );
            guard = Some(file);
            let plan = AuthorityManagedSuccessorLaunchPlanV1 {
                schema_version: 1,
                authority_store_id: context.workspace_binding.authority_store_id.clone(),
                helper_plan: HiddenOwnerHelperLaunchPlan {
                    mode: OwnerHelperMode::ResumeOneTurn,
                    descriptor: ResolvedRuntimeDescriptor {
                        agent_id: context.descriptor.agent_id.clone(),
                        backend_id: context.descriptor.backend_id.clone(),
                        backend_kind: ResolvedRuntimeBackendKind::Codex,
                        protocol: context.descriptor.protocol.clone(),
                        execution_scope: AgentExecutionScope::Host,
                        binary_path: context.descriptor.binary_path.clone(),
                    },
                    session: HiddenOwnerHelperSessionPlan {
                        orchestration_session_id: context.orchestration_session_id.clone(),
                        shell_trace_session_id: context.shell_trace_session_id.clone(),
                        workspace_root: workspace.display().to_string(),
                        world_id: None,
                        world_generation: None,
                    },
                    participant: HiddenOwnerHelperParticipantPlan {
                        participant_id: participant.clone(),
                        lease_token: String::from_utf8(
                            successor_request.target_participant_lease_token,
                        )?,
                        run_id: applied.run_id.clone(),
                        resumed_from_participant_id: successor_request
                            .source_authoritative_participant_id,
                        internal_uaa_session_id: Some(native_session.into()),
                    },
                    host_attach_contract: None,
                    startup_prompt: Some(HiddenOwnerHelperStartupPromptPlan {
                        prompt_text: prompt.clone(),
                        stream_path: workspace.join(format!("host-native-stream-{turn}")),
                        request_key_sha256: None,
                        prompt_sha256: None,
                        public_backend_id: None,
                        public_scope: None,
                    }),
                    source_orchestration_session_id: None,
                },
                applied_transition: applied.clone(),
            };
            successor = Some(plan);
            (participant, applied.run_id)
        };
        let runtime = transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor {
            orchestration_session_id: context.orchestration_session_id.clone(),
            participant_id: participant.clone(),
            backend_id: context.descriptor.backend_id.clone(),
            run_id: run.clone(),
        };
        if let Some(old_runtime) = previous_runtime.take() {
            let mut old_claim = original_claim.clone();
            old_claim.runtime = old_runtime;
            ensure!(
                verifier.authenticate_host_gateway_v1(&old_claim).is_err(),
                "prior run must refuse"
            );
        }
        let selected = Arc::new(verifier.authenticate(&context.orchestration_session_id)?);
        let bridge = PromptFulfillmentBridge::for_ordinary_descriptor(
            &descriptor,
            selected,
            runtime.clone(),
        )?;
        let attempt = bridge.native_attempt();
        let native_request = agent_api::AgentWrapperRunRequest {
            prompt,
            working_dir: Some(workspace.into()),
            timeout: Some(Duration::from_secs(30)),
            env: BTreeMap::new(),
            extensions: if turn == 1 {
                BTreeMap::new()
            } else {
                BTreeMap::from([(
                    "agent_api.session.resume.v1".into(),
                    serde_json::json!({"selector":"id","id":native_session}),
                )])
            },
        };
        let mut control = if turn == 1 {
            bridge
                .run_durable_start_control(native_request, &attempt)
                .await?
        } else {
            bridge
                .run_authority_managed_resume_one_turn_control(
                    native_session,
                    native_request,
                    &attempt,
                )
                .await?
        };
        let mut events = Vec::new();
        while let Some(event) = control.handle.events.next().await {
            let completed = event
                .data
                .as_ref()
                .is_some_and(|data| data["turn"]["phase"] == "completed");
            checkpoint(serde_json::json!({
                "phase": "native-event-observation",
                "turn": turn,
                "index": events.len() + 1,
                "kind": match &event.kind {
                    agent_api::AgentWrapperEventKind::TextOutput => "text_output",
                    agent_api::AgentWrapperEventKind::ToolCall => "tool_call",
                    agent_api::AgentWrapperEventKind::ToolResult => "tool_result",
                    agent_api::AgentWrapperEventKind::Status => "status",
                    agent_api::AgentWrapperEventKind::Error => "error",
                    agent_api::AgentWrapperEventKind::Unknown => "unknown",
                },
                "typed_phase": match event.data.as_ref()
                    .and_then(|data| data.pointer("/turn/phase"))
                    .and_then(serde_json::Value::as_str) {
                    Some("exchange_opened") => "exchange_opened",
                    Some("started") => "started",
                    Some("completed") => "completed",
                    Some("failed") => "failed",
                    Some(_) => "unrecognized",
                    None => "absent",
                },
                "message_present": event.message.is_some(),
                "data_present": event.data.is_some(),
            }))?;
            events.push(event);
            if completed {
                break;
            }
        }
        checkpoint(serde_json::json!({
            "phase": "native-event-summary", "turn": turn, "events": events.len()
        }))?;
        ensure!(
            events.len() == 3,
            "actual typed lifecycle event sequence missing/extra"
        );
        for (event, phase) in events
            .iter()
            .zip(["exchange_opened", "started", "completed"])
        {
            let data = event.data.as_ref().context("typed native data missing")?;
            ensure!(data["turn"]["phase"] == phase && data["session"]["id"] == native_session);
            ensure!(data["turn"]["thread_id"] == native_session);
            if phase != "exchange_opened" {
                ensure!(data["turn"]["turn_id"] == format!("host-turn-{turn}"));
            }
        }
        ensure!(
            tokio::time::timeout(Duration::from_millis(100), control.terminal.wait())
                .await
                .is_err(),
            "provider terminal cannot acknowledge the still-held actual child"
        );
        private_file(&workspace.join(format!("host-release-{turn}")), b"release")?;
        ensure!(
            control.handle.events.next().await.is_none(),
            "extra native events after terminal"
        );
        let completion = control.handle.completion.await?;
        ensure!(
            completion.status.success()
                && completion.final_text.as_deref()
                    == Some(format!("synthetic host output {turn}").as_str())
        );
        let terminal = control.terminal.wait().await;
        terminal.require_checked()?;
        ensure!(terminal
            .raw
            .as_ref()
            .is_ok_and(|completion| completion.status.success()));
        let opened = events[0].data.as_ref().unwrap();
        let completed = events[2].data.as_ref().unwrap();
        let evidence = completed["evidence"]["raw_event_sha256"]
            .as_str()
            .context("terminal evidence missing")?;
        if turn == 1 {
            let registration = EstablishStartContinuationRequestV1 {
                start_transaction_id: transaction.transaction_id.clone(),
                request_key_sha256: prompt_hash.clone(),
                authority_store_id: transaction.authority_store_id.clone(),
                orchestration_session_id: context.orchestration_session_id.clone(),
                intent_id: applied.intent_id.clone(),
                issuer_request_id: applied.issuer_request_id.clone(),
                payload_commitment: applied.payload_commitment.clone(),
                application_result_ref: application_result_ref.clone(),
                run_id: applied.run_id.clone(),
                expected_authority_revision: *authority_revision_after,
                authoritative_participant_id: participant.clone(),
                backend_id: context.descriptor.backend_id.clone(),
                protocol: context.descriptor.protocol.clone(),
                exchange_id: "exchange_private_host_native03".into(),
                exchange_sequence: 1,
                provider_event_kind: opened["evidence"]["provider_event_kind"]
                    .as_str()
                    .context("open kind")?
                    .into(),
                exchange_evidence_sha256: opened["evidence"]["raw_event_sha256"]
                    .as_str()
                    .context("open evidence")?
                    .into(),
                internal_uaa_session_id: native_session.into(),
                observed_at: native_timestamp()?,
            };
            let StartContinuationOutcomeV1::Applied(registered) =
                authority.establish_start_continuation(&registration)?
            else {
                anyhow::bail!("actual Start continuation must commit");
            };
            let StartTurnSettlementOutcomeV1::Applied(settled) =
                authority.settle_start_turn(&SettleStartTurnRequestV1 {
                    start_transaction_id: transaction.transaction_id.clone(),
                    request_key_sha256: prompt_hash.clone(),
                    authority_store_id: transaction.authority_store_id.clone(),
                    orchestration_session_id: context.orchestration_session_id.clone(),
                    intent_id: applied.intent_id.clone(),
                    issuer_request_id: applied.issuer_request_id.clone(),
                    payload_commitment: applied.payload_commitment.clone(),
                    application_result_ref: application_result_ref.clone(),
                    run_id: applied.run_id.clone(),
                    authoritative_participant_id: participant.clone(),
                    backend_id: context.descriptor.backend_id.clone(),
                    protocol: context.descriptor.protocol.clone(),
                    registered_resume_handle_ref: registered.resume_handle_ref,
                    expected_authority_revision: registered.authority_revision_after,
                    protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                        participant_id: participant.clone(),
                    },
                    event_id: "event_private_host_native03_start".into(),
                    event_sequence: 2,
                    provider_event_kind: "turn.completed".into(),
                    thread_id: native_session.into(),
                    turn_id: format!("host-turn-{turn}"),
                    completion_evidence_sha256: evidence.into(),
                    kind: StartTurnCompletionKindV1::ResumableClean,
                    obligation_ledger_read: None,
                    completed_at: native_timestamp()?,
                })?
            else {
                anyhow::bail!("actual Start must settle");
            };
            resume_handle = Some(settled.continuation_resume_handle_ref);
        } else {
            let plan = successor
                .as_ref()
                .context("actual successor plan missing")?;
            let native = DirectHostNativeTerminalEvidenceV1 {
                schema_version: 1,
                backend_id: context.descriptor.backend_id.clone(),
                protocol: context.descriptor.protocol.clone(),
                native_session_id: native_session.into(),
                thread_id: native_session.into(),
                turn_id: format!("host-turn-{turn}"),
                sequence: DirectHostNativeSequenceV1::Codex {
                    resumed_thread_id: native_session.into(),
                    started_turn_id: format!("host-turn-{turn}"),
                },
                terminal_event_kind: "turn.completed".into(),
                raw_event_sha256: evidence.into(),
                terminal_ordinal: 1,
            };
            let PostTurnResolutionOutcomeV1::Applied(settled) = authority
                .settle_direct_host_resume(
                    plan,
                    &native,
                    HostPostTurnProtocolEventKindV1::ResumableClean,
                    &native_timestamp()?,
                )?
            else {
                anyhow::bail!("actual successor settlement must commit");
            };
            ensure!(
                settled.intent_id == plan.applied_transition.intent_id
                    && settled.issuer_request_id == plan.applied_transition.issuer_request_id
                    && settled.payload_commitment == plan.applied_transition.payload_commitment
                    && settled.run_id == run
                    && settled.target_authoritative_participant_id == participant,
                "settlement returned a different successor"
            );
            let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &settled.state
            else {
                anyhow::bail!("settled successor is not Applied");
            };
            let HostSessionPostTurnApplicationV2::Applied {
                authority_revision_before,
                authority_revision_after,
                resulting_posture,
                ..
            } = post_turn.as_ref()
            else {
                anyhow::bail!("actual successor completion missing");
            };
            ensure!(
                authority_revision_before.checked_add(1) == Some(*authority_revision_after)
                    && *resulting_posture == HostSessionPostureV1::ParkedResumable,
                "settled successor is not the next resumable authority"
            );
            // The existing typed facade verifies the exact V3 history, including
            // native completion and its produced-handle edge. No raw HSA object
            // or root is parsed here, and no last-handle guess substitutes for it.
            let before = authority.resolve_exact_at_revision(
                &settled.orchestration_session_id,
                *authority_revision_before,
            )?;
            let after = authority.resolve_exact_at_revision(
                &settled.orchestration_session_id,
                *authority_revision_after,
            )?;
            ensure!(
                before
                    .authority
                    .active_authoritative_participant_id
                    .as_deref()
                    == Some(participant.as_str())
                    && after
                        .authority
                        .active_authoritative_participant_id
                        .as_deref()
                        == Some(participant.as_str())
                    && before.authority.workspace_binding == context.workspace_binding
                    && after.authority.workspace_binding == context.workspace_binding
                    && before.authority.world_binding.is_none()
                    && after.authority.world_binding.is_none(),
                "typed settlement ancestry differs from the actual Host successor"
            );
            let prior_handles = &before.authority.internal_resume_handle_refs;
            let settled_handles = &after.authority.internal_resume_handle_refs;
            ensure!(
                prior_handles.len().checked_add(1) == Some(settled_handles.len())
                    && settled_handles.get(..prior_handles.len()) == Some(prior_handles.as_slice()),
                "typed settlement did not preserve the prefix and append exactly one handle"
            );
            let produced = settled_handles
                .get(prior_handles.len())
                .context("typed settlement produced no successor handle")?;
            ensure!(
                produced.object_kind == AuthorityObjectKindV1::ResumeHandle
                    && produced.schema_version == 1,
                "typed settlement produced an unsupported successor handle"
            );
            let current =
                authority.resolve_current_exact(&settled.orchestration_session_id, None)?;
            ensure!(
                current.observation == after.observation()
                    && current.caller.participant_id == participant,
                "settled typed authority is no longer current"
            );
            authority.resolve_current_exact(
                &settled.orchestration_session_id,
                Some(&current.observation),
            )?;
            resume_handle = Some(produced.clone());
        }
        match previous_guard {
            Some(value) => {
                std::env::set_var("SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD", value)
            }
            None => std::env::remove_var("SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD"),
        }
        drop(guard);
        let rows = std::fs::read_to_string(workspace.join("host-native.jsonl"))?
            .lines()
            .map(serde_json::from_str::<serde_json::Value>)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(rows.len() == turn);
        let row = rows.last().unwrap();
        ensure!(
            row["parent_pid"] == std::process::id() && row["authenticated_body_refusal"] == true
        );
        ensure!(pids.insert(row["pid"].as_u64().context("actual PID missing")?));
        for field in [
            "home_device",
            "home_inode",
            "history_device",
            "history_inode",
        ] {
            ensure!(
                row[field] == rows[0][field],
                "physical original history changed"
            );
        }
        ensure!(
            row["history"]
                == format!(
                    "{native_session}{}",
                    (1..=turn)
                        .map(|n| format!("\nturn-{n}"))
                        .collect::<String>()
                )
        );
        checkpoint(
            serde_json::json!({"phase":"native-turn-complete", "turn":turn, "current_run":run,
            "checked_native_and_host_cleanup":true, "source_history_retained":true}),
        )?;
        previous_runtime = Some(runtime);
    }
    checkpoint(serde_json::json!({"phase":"complete", "native_turns":3,
        "all_gateway_releases_checked":true, "provider_requests":0, "component_only":true}))?;
    Ok(())
}
