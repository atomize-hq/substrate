#![allow(unused_crate_dependencies)]

use std::fs;
use std::process::Command;

use agent_drift_sentinel::{
    LiveSessionCoordinator, LiveSessionError, LiveSessionRequest, SchedulerPolicy, WarningPolicy,
};
use camino::Utf8Path;
use serde_json::{json, Value};
use syn::visit::{self, Visit};
use tempfile::TempDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryStep {
    AllocateEmissionOrdinal,
    ObserveRuntime,
    RecordDelivery,
    PersistState,
    AcceptObservation,
}

#[test]
fn real_session_live_source_orders_delivery_after_fallible_runtime_observation() {
    let source_path = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("src/real_session_live.rs");
    let source = fs::read_to_string(&source_path).expect("read real-session live source");
    let parsed = syn::parse_file(&source).expect("parse real-session live source");

    assert_real_session_live_delivery_contract(&source_path, &parsed)
        .expect("protected delivery order");
}

#[test]
fn real_session_live_source_proof_accepts_compile_valid_active_control() {
    let active_control = dominance_fixture(
        r#"
            let observation = self.runtime.observe(event)?;
            self.progress
                .record_delivery(observation.event.cursor.clone());
        "#,
    );
    assert_rustc_check(&active_control);
    assert_poll_once_delivery_contract(
        &syn::parse_file(&active_control).expect("parse active-control fixture"),
    )
    .expect("active control keeps bookkeeping before observe and protected effects after it");
}

#[test]
fn real_session_live_source_proof_rejects_conditional_observe_before_unconditional_effects() {
    let conditional_observe = dominance_fixture(
        r#"
            let observation;
            if checkpoint {
                observation = self.runtime.observe(event)?;
            } else {
                observation = Observation { event };
            }
            self.progress
                .record_delivery(observation.event.cursor.clone());
        "#,
    );
    assert_rustc_check(&conditional_observe);
    assert!(
        assert_poll_once_delivery_contract(
            &syn::parse_file(&conditional_observe).expect("parse conditional-observe fixture"),
        )
        .is_err(),
        "observe in only one branch must not dominate unconditional delivery and persistence"
    );
}

#[test]
fn real_session_live_source_proof_rejects_macro_hidden_delivery_before_observe() {
    let macro_hidden_delivery = dominance_fixture(
        r#"
            deliver_early!(self, event.cursor.clone());
            let observation = self.runtime.observe(event)?;
            self.progress
                .record_delivery(observation.event.cursor.clone());
        "#,
    );
    assert_rustc_check(&macro_hidden_delivery);
    assert!(
        assert_poll_once_delivery_contract(
            &syn::parse_file(&macro_hidden_delivery).expect("parse macro-delivery fixture"),
        )
        .is_err(),
        "an unclassified macro must not hide protected delivery before observe"
    );
}

#[test]
fn real_session_live_source_proof_rejects_aliased_operator_sink() {
    let aliased_operator_sink = dominance_fixture(
        r#"
            let observation = self.runtime.observe(event)?;
            let sink = &mut self.operator_sink;
            sink.emit(&observation);
            self.progress
                .record_delivery(observation.event.cursor.clone());
        "#,
    );
    assert_rustc_check(&aliased_operator_sink);
    assert!(
        assert_poll_once_delivery_contract(
            &syn::parse_file(&aliased_operator_sink).expect("parse sink-alias fixture"),
        )
        .is_err(),
        "operator-sink aliases must not hide a protected effect"
    );
}

#[test]
fn real_session_live_source_proof_rejects_compile_valid_delivery_before_observe() {
    let delivery_before_observe = dominance_fixture(
        r#"
            self.progress.record_delivery(event.cursor.clone());
            let observation = self.runtime.observe(event)?;
        "#,
    );
    assert_rustc_check(&delivery_before_observe);
    assert!(
        assert_poll_once_delivery_contract(
            &syn::parse_file(&delivery_before_observe).expect("parse reordered fixture"),
        )
        .is_err(),
        "compile-valid record_delivery before observe must be rejected"
    );
}

fn assert_rustc_check(source: &str) {
    let temp_dir = TempDir::new().expect("rustc fixture temp dir");
    let source_path = temp_dir.path().join("dominance_fixture.rs");
    let metadata_path = temp_dir.path().join("dominance_fixture.rmeta");
    fs::write(&source_path, source).expect("write rustc fixture");
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-name=dominance_fixture")
        .arg("--emit=metadata")
        .arg("-o")
        .arg(metadata_path)
        .arg(source_path)
        .output()
        .expect("run rustc fixture check");
    assert!(
        output.status.success(),
        "fixture must compile successfully:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn dominance_fixture(delivery_body: &str) -> String {
    format!(
        r#"
        #[derive(Clone)]
        struct Cursor;

        struct Event {{
            cursor: Cursor,
        }}

        struct Observation {{
            event: Event,
        }}

        #[derive(Debug)]
        struct Error;

        struct Runtime;

        impl Runtime {{
            fn observe(&mut self, event: Event) -> Result<Observation, Error> {{
                Ok(Observation {{ event }})
            }}
        }}

        struct Progress {{
            monitor_linked_closure: bool,
        }}

        impl Progress {{
            fn begin_poll(&mut self, _observed_size_bytes: u64) {{}}

            fn checkpoint_ready_event(&mut self, _checkpoint: bool, _path: &str) -> Event {{
                Event {{ cursor: Cursor }}
            }}

            fn record_delivery(&mut self, _cursor: Cursor) {{}}
        }}

        struct OperatorSink;

        impl OperatorSink {{
            fn emit(&mut self, _observation: &Observation) {{}}
        }}

        macro_rules! deliver_early {{
            ($coordinator:expr, $cursor:expr) => {{
                $coordinator.progress.record_delivery($cursor);
            }};
        }}

        struct LiveSessionCoordinator {{
            progress: Progress,
            runtime: Runtime,
            operator_sink: OperatorSink,
            rollout_path: String,
        }}

        impl LiveSessionCoordinator {{
            fn persist_state(&self) -> Result<(), Error> {{
                Ok(())
            }}

            fn poll_once(&mut self) -> Result<Vec<Observation>, Error> {{
                let observed_size_bytes = 1;
                self.progress.monitor_linked_closure = false;
                self.progress.begin_poll(observed_size_bytes);
                let fresh_checkpoints = [true];
                let mut observations = Vec::with_capacity(fresh_checkpoints.len());
                for checkpoint in fresh_checkpoints {{
                    let event = self
                        .progress
                        .checkpoint_ready_event(checkpoint, &self.rollout_path);
                    {delivery_body}
                    self.persist_state()?;
                    observations.push(observation);
                }}
                Ok(observations)
            }}
        }}

        fn main() {{
            let mut coordinator = LiveSessionCoordinator {{
                progress: Progress {{ monitor_linked_closure: false }},
                runtime: Runtime,
                operator_sink: OperatorSink,
                rollout_path: String::new(),
            }};
            let _ = coordinator.poll_once();
        }}
        "#
    )
}

fn assert_real_session_live_delivery_contract(
    source_path: &Utf8Path,
    file: &syn::File,
) -> Result<(), String> {
    let expected_path = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("src/real_session_live.rs");
    if source_path != expected_path {
        return Err(format!(
            "delivery proof must inspect the real_session_live module at {expected_path}"
        ));
    }
    assert_poll_once_delivery_contract(file)
}

fn assert_poll_once_delivery_contract(file: &syn::File) -> Result<(), String> {
    let block = poll_once_block(file).ok_or_else(|| {
        "expected exactly one inherent real_session_live::LiveSessionCoordinator::poll_once(&mut self) -> Result<...>"
            .to_string()
    })?;
    let delivery_loops = block
        .stmts
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            checkpoint_delivery_loop(statement).map(|delivery_loop| (index, delivery_loop))
        })
        .collect::<Vec<_>>();
    let [(delivery_loop_index, delivery_loop)] = delivery_loops.as_slice() else {
        return Err(format!(
            "poll_once must contain exactly one top-level `for checkpoint in fresh_checkpoints` delivery loop; found {}",
            delivery_loops.len()
        ));
    };

    let monitor_indices = block
        .stmts
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            statement_assigns_monitor_linked_closure(statement).then_some(index)
        })
        .collect::<Vec<_>>();
    let [monitor_index] = monitor_indices.as_slice() else {
        return Err(format!(
            "poll_once must contain exactly one top-level monitor-closure assignment; found {}",
            monitor_indices.len()
        ));
    };
    let begin_poll_indices = block
        .stmts
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| statement_is_begin_poll(statement).then_some(index))
        .collect::<Vec<_>>();
    let [begin_poll_index] = begin_poll_indices.as_slice() else {
        return Err(format!(
            "poll_once must contain exactly one top-level pending-poll begin; found {}",
            begin_poll_indices.len()
        ));
    };
    if !(monitor_index < begin_poll_index && begin_poll_index < delivery_loop_index) {
        return Err(
            "monitor-closure and pending-poll bookkeeping must precede the delivery loop"
                .to_string(),
        );
    }

    assert_exact_dominated_delivery_loop(delivery_loop)?;

    let full_scan = protected_surface_in_block(block);
    if !full_scan.errors.is_empty() {
        return Err(format!(
            "poll_once contains an aliased, macro-hidden, or unclassified protected surface: {:?}",
            full_scan.errors
        ));
    }
    for protected_step in [
        DeliveryStep::AllocateEmissionOrdinal,
        DeliveryStep::ObserveRuntime,
        DeliveryStep::RecordDelivery,
        DeliveryStep::AcceptObservation,
    ] {
        let count = full_scan.count(protected_step);
        if count != 1 {
            return Err(format!(
                "poll_once must contain exactly one classified {protected_step:?}; found {count}"
            ));
        }
    }
    if full_scan.runtime_roots != full_scan.count(DeliveryStep::ObserveRuntime) {
        return Err(format!(
            "every self.runtime occurrence must be the classified observe receiver; found {} roots and {} observes",
            full_scan.runtime_roots,
            full_scan.count(DeliveryStep::ObserveRuntime)
        ));
    }

    assert_only_bookkeeping_persistence_outside_delivery(block, *delivery_loop_index)?;
    Ok(())
}

fn assert_exact_dominated_delivery_loop(delivery_loop: &syn::ExprForLoop) -> Result<(), String> {
    // This is intentionally a conservative top-level shape lock, not a general CFG proof. The
    // straight-line loop body makes the fallible observation dominate every protected effect.
    let statements = &delivery_loop.body.stmts;
    if statements.len() != 5 {
        return Err(format!(
            "delivery loop must keep five top-level statements (allocate, observe?, record, persist?, accept); found {}",
            statements.len()
        ));
    }
    if !statement_allocates_event(&statements[0]) {
        return Err(
            "delivery statement 1 must allocate `event` and its emission ordinal".to_string(),
        );
    }
    if !statement_fallibly_observes_event(&statements[1]) {
        return Err(
            "delivery statement 2 must be top-level `let observation = self.runtime.observe(event)?;`"
                .to_string(),
        );
    }
    if !statement_records_observation_cursor(&statements[2]) {
        return Err(
            "delivery statement 3 must record the successfully observed cursor".to_string(),
        );
    }
    if !statement_fallibly_persists_state(&statements[3]) {
        return Err("delivery statement 4 must fallibly persist recorded state".to_string());
    }
    if !statement_accepts_observation(&statements[4]) {
        return Err("delivery statement 5 must accept the persisted observation".to_string());
    }

    let loop_scan = protected_surface_in_block(&delivery_loop.body);
    if !loop_scan.errors.is_empty() {
        return Err(format!(
            "delivery loop contains an aliased, macro-hidden, or unclassified protected surface: {:?}",
            loop_scan.errors
        ));
    }
    for protected_step in [
        DeliveryStep::AllocateEmissionOrdinal,
        DeliveryStep::ObserveRuntime,
        DeliveryStep::RecordDelivery,
        DeliveryStep::PersistState,
        DeliveryStep::AcceptObservation,
    ] {
        let count = loop_scan.count(protected_step);
        if count != 1 {
            return Err(format!(
                "delivery loop must contain exactly one classified {protected_step:?}; found {count}"
            ));
        }
    }
    Ok(())
}

fn assert_only_bookkeeping_persistence_outside_delivery(
    block: &syn::Block,
    delivery_loop_index: usize,
) -> Result<(), String> {
    // Existing sparse-startup exits persist transport bookkeeping without accepting a checkpoint;
    // all delivery/cursor effects remain confined to the dominated loop above.
    for (index, statement) in block.stmts.iter().enumerate() {
        if index == delivery_loop_index {
            continue;
        }
        let scan = protected_surface_in_statement(statement);
        if !scan.errors.is_empty() {
            return Err(format!(
                "statement {index} outside delivery has an unclassified protected surface: {:?}",
                scan.errors
            ));
        }
        if scan
            .steps
            .iter()
            .any(|step| !matches!(step, DeliveryStep::PersistState))
        {
            return Err(format!(
                "statement {index} outside the dominated loop reaches a protected delivery effect: {:?}",
                scan.steps
            ));
        }
        let persist_count = scan.count(DeliveryStep::PersistState);
        if persist_count == 0 {
            continue;
        }
        let permitted_pre_delivery_return = index < delivery_loop_index
            && persist_count == 1
            && statement_has_method_call(statement, "complete_poll", &["self", "progress"])
            && statement_contains_return(statement);
        let permitted_post_delivery_completion = index > delivery_loop_index
            && persist_count == 1
            && statement_fallibly_persists_state(statement)
            && index > 0
            && statement_is_complete_poll(&block.stmts[index - 1]);
        if !permitted_pre_delivery_return && !permitted_post_delivery_completion {
            return Err(format!(
                "statement {index} persists outside the dominated delivery sequence without a classified bookkeeping-only exit"
            ));
        }
    }
    Ok(())
}

fn poll_once_block(file: &syn::File) -> Option<&syn::Block> {
    let mut result = None;
    for item in &file.items {
        let syn::Item::Impl(item_impl) = item else {
            continue;
        };
        if item_impl.trait_.is_some()
            || type_path(&item_impl.self_ty) != Some(vec!["LiveSessionCoordinator".to_string()])
        {
            continue;
        }
        for item in &item_impl.items {
            let syn::ImplItem::Fn(method) = item else {
                continue;
            };
            if method.sig.ident != "poll_once" {
                continue;
            }
            if !poll_once_signature_is_exact(&method.sig) || result.is_some() {
                return None;
            }
            result = Some(&method.block);
        }
    }
    result
}

fn poll_once_signature_is_exact(signature: &syn::Signature) -> bool {
    if signature.constness.is_some()
        || signature.asyncness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || signature.inputs.len() != 1
    {
        return false;
    }
    let Some(syn::FnArg::Receiver(receiver)) = signature.inputs.first() else {
        return false;
    };
    if receiver.reference.is_none()
        || receiver.mutability.is_none()
        || receiver.colon_token.is_some()
    {
        return false;
    }
    match &signature.output {
        syn::ReturnType::Type(_, ty) => type_path(ty)
            .and_then(|path| path.last().cloned())
            .is_some_and(|name| name == "Result"),
        syn::ReturnType::Default => false,
    }
}

fn checkpoint_delivery_loop(statement: &syn::Stmt) -> Option<&syn::ExprForLoop> {
    let syn::Stmt::Expr(syn::Expr::ForLoop(delivery_loop), None) = statement else {
        return None;
    };
    if delivery_loop.label.is_some()
        || !simple_pattern_is(&delivery_loop.pat, "checkpoint")
        || !expression_is_path(&delivery_loop.expr, &["fresh_checkpoints"])
    {
        return None;
    }
    Some(delivery_loop)
}

fn statement_allocates_event(statement: &syn::Stmt) -> bool {
    let Some(initializer) = simple_local_initializer(statement, "event") else {
        return false;
    };
    let syn::Expr::MethodCall(call) = initializer else {
        return false;
    };
    call.method == "checkpoint_ready_event"
        && call.turbofish.is_none()
        && expression_is_path(&call.receiver, &["self", "progress"])
        && call.args.len() == 2
        && call
            .args
            .first()
            .is_some_and(|argument| expression_is_path(argument, &["checkpoint"]))
        && call.args.iter().nth(1).is_some_and(|argument| {
            matches!(argument, syn::Expr::Reference(reference)
                if reference.mutability.is_none()
                    && expression_is_path(&reference.expr, &["self", "rollout_path"]))
        })
}

fn statement_fallibly_observes_event(statement: &syn::Stmt) -> bool {
    let Some(initializer) = simple_local_initializer(statement, "observation") else {
        return false;
    };
    let syn::Expr::Try(observation_try) = initializer else {
        return false;
    };
    let syn::Expr::MethodCall(call) = observation_try.expr.as_ref() else {
        return false;
    };
    call.method == "observe"
        && call.turbofish.is_none()
        && expression_is_path(&call.receiver, &["self", "runtime"])
        && call.args.len() == 1
        && call
            .args
            .first()
            .is_some_and(|argument| expression_is_path(argument, &["event"]))
}

fn statement_records_observation_cursor(statement: &syn::Stmt) -> bool {
    let Some(call) = top_level_method_call(statement) else {
        return false;
    };
    call.method == "record_delivery"
        && call.turbofish.is_none()
        && expression_is_path(&call.receiver, &["self", "progress"])
        && call.args.len() == 1
        && call.args.first().is_some_and(|argument| {
            let syn::Expr::MethodCall(clone_call) = argument else {
                return false;
            };
            clone_call.method == "clone"
                && clone_call.turbofish.is_none()
                && clone_call.args.is_empty()
                && expression_is_path(&clone_call.receiver, &["observation", "event", "cursor"])
        })
}

fn statement_fallibly_persists_state(statement: &syn::Stmt) -> bool {
    let syn::Stmt::Expr(syn::Expr::Try(persist_try), Some(_)) = statement else {
        return false;
    };
    let syn::Expr::MethodCall(call) = persist_try.expr.as_ref() else {
        return false;
    };
    call.method == "persist_state"
        && call.turbofish.is_none()
        && call.args.is_empty()
        && expression_is_path(&call.receiver, &["self"])
}

fn statement_accepts_observation(statement: &syn::Stmt) -> bool {
    let Some(call) = top_level_method_call(statement) else {
        return false;
    };
    call.method == "push"
        && call.turbofish.is_none()
        && expression_is_path(&call.receiver, &["observations"])
        && call.args.len() == 1
        && call
            .args
            .first()
            .is_some_and(|argument| expression_is_path(argument, &["observation"]))
}

fn statement_assigns_monitor_linked_closure(statement: &syn::Stmt) -> bool {
    matches!(statement,
    syn::Stmt::Expr(syn::Expr::Assign(assignment), Some(_))
        if expression_is_path(
            &assignment.left,
            &["self", "progress", "monitor_linked_closure"],
        ))
}

fn statement_is_begin_poll(statement: &syn::Stmt) -> bool {
    top_level_method_call(statement).is_some_and(|call| {
        call.method == "begin_poll"
            && call.turbofish.is_none()
            && expression_is_path(&call.receiver, &["self", "progress"])
            && call.args.len() == 1
            && call
                .args
                .first()
                .is_some_and(|argument| expression_is_path(argument, &["observed_size_bytes"]))
    })
}

fn statement_is_complete_poll(statement: &syn::Stmt) -> bool {
    top_level_method_call(statement).is_some_and(|call| {
        call.method == "complete_poll"
            && call.turbofish.is_none()
            && expression_is_path(&call.receiver, &["self", "progress"])
            && call.args.len() == 1
            && call
                .args
                .first()
                .is_some_and(|argument| expression_is_path(argument, &["observed_size_bytes"]))
    })
}

fn top_level_method_call(statement: &syn::Stmt) -> Option<&syn::ExprMethodCall> {
    match statement {
        syn::Stmt::Expr(syn::Expr::MethodCall(call), Some(_)) => Some(call),
        _ => None,
    }
}

fn simple_local_initializer<'a>(statement: &'a syn::Stmt, name: &str) -> Option<&'a syn::Expr> {
    let syn::Stmt::Local(local) = statement else {
        return None;
    };
    if !local.attrs.is_empty() || !simple_pattern_is(&local.pat, name) {
        return None;
    }
    let initializer = local.init.as_ref()?;
    initializer
        .diverge
        .is_none()
        .then_some(initializer.expr.as_ref())
}

fn simple_pattern_is(pattern: &syn::Pat, name: &str) -> bool {
    matches!(pattern,
        syn::Pat::Ident(ident)
            if ident.attrs.is_empty()
                && ident.by_ref.is_none()
                && ident.mutability.is_none()
                && ident.subpat.is_none()
                && ident.ident == name)
}

fn statement_has_method_call(statement: &syn::Stmt, method: &str, receiver: &[&str]) -> bool {
    struct MethodCallVisitor<'a> {
        method: &'a str,
        receiver: &'a [&'a str],
        found: bool,
    }

    impl<'ast> Visit<'ast> for MethodCallVisitor<'_> {
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            if call.method == self.method && expression_is_path(&call.receiver, self.receiver) {
                self.found = true;
            }
            visit::visit_expr_method_call(self, call);
        }
    }

    let mut visitor = MethodCallVisitor {
        method,
        receiver,
        found: false,
    };
    visitor.visit_stmt(statement);
    visitor.found
}

fn statement_contains_return(statement: &syn::Stmt) -> bool {
    #[derive(Default)]
    struct ReturnVisitor {
        found: bool,
    }

    impl<'ast> Visit<'ast> for ReturnVisitor {
        fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
            self.found = true;
            visit::visit_expr_return(self, expression);
        }
    }

    let mut visitor = ReturnVisitor::default();
    visitor.visit_stmt(statement);
    visitor.found
}

#[derive(Default)]
struct ProtectedSurfaceVisitor {
    steps: Vec<DeliveryStep>,
    errors: Vec<String>,
    runtime_roots: usize,
}

impl ProtectedSurfaceVisitor {
    fn count(&self, step: DeliveryStep) -> usize {
        self.steps.iter().filter(|found| **found == step).count()
    }

    fn merge(&mut self, other: Self) {
        self.steps.extend(other.steps);
        self.errors.extend(other.errors);
        self.runtime_roots += other.runtime_roots;
    }
}

impl<'ast> Visit<'ast> for ProtectedSurfaceVisitor {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(initializer) = &local.init {
            if let Some(target) = direct_protected_alias_target(&initializer.expr) {
                self.errors
                    .push(format!("local alias to protected target {target}"));
            }
        }
        visit::visit_local(self, local);
    }

    fn visit_expr_assign(&mut self, assignment: &'ast syn::ExprAssign) {
        if let Some(target) = direct_protected_alias_target(&assignment.right) {
            self.errors
                .push(format!("assignment alias to protected target {target}"));
        }
        visit::visit_expr_assign(self, assignment);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if let Some(step) = classify_protected_method_call(call) {
            self.steps.push(step);
        } else {
            let receiver = expression_path(&call.receiver);
            if protected_method_name(&call.method.to_string())
                || receiver
                    .as_deref()
                    .is_some_and(path_has_forbidden_capability)
            {
                self.errors.push(format!(
                    "unclassified method call {}.{}",
                    receiver
                        .map(|path| path.join("."))
                        .unwrap_or_else(|| "<expression>".to_string()),
                    call.method
                ));
            }
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
        if let Some(path) = expression_path(&syn::Expr::Field(field.clone())) {
            if path == ["self", "runtime"] {
                self.runtime_roots += 1;
            }
            if path_has_forbidden_capability(&path) {
                self.errors
                    .push(format!("forbidden protected capability {}", path.join(".")));
            }
        }
        visit::visit_expr_field(self, field);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        let path = expression
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        if path_has_forbidden_capability(&path)
            || path.last().is_some_and(|name| protected_method_name(name))
        {
            self.errors.push(format!(
                "protected function or capability path {}",
                path.join("::")
            ));
        }
        visit::visit_expr_path(self, expression);
    }

    fn visit_macro(&mut self, item_macro: &'ast syn::Macro) {
        if !path_is(&item_macro.path, &["matches"]) {
            self.errors.push(format!(
                "unclassified macro {}!",
                item_macro
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            ));
            return;
        }
        match syn::parse2::<MatchesMacroInput>(item_macro.tokens.clone()) {
            Ok(arguments) => {
                let mut inspected = Self::default();
                inspected.visit_expr(&arguments.expression);
                if let Some(guard) = arguments.guard {
                    inspected.visit_expr(&guard);
                }
                self.merge(inspected);
            }
            Err(error) => self
                .errors
                .push(format!("could not inspect allowed matches! macro: {error}")),
        }
    }
}

struct MatchesMacroInput {
    expression: syn::Expr,
    guard: Option<syn::Expr>,
}

impl syn::parse::Parse for MatchesMacroInput {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let expression = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let _pattern = input.call(syn::Pat::parse_multi_with_leading_vert)?;
        let guard = if input.peek(syn::Token![if]) {
            input.parse::<syn::Token![if]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("unexpected matches! tokens"));
        }
        Ok(Self { expression, guard })
    }
}

fn protected_surface_in_block(block: &syn::Block) -> ProtectedSurfaceVisitor {
    let mut visitor = ProtectedSurfaceVisitor::default();
    visitor.visit_block(block);
    visitor
}

fn protected_surface_in_statement(statement: &syn::Stmt) -> ProtectedSurfaceVisitor {
    let mut visitor = ProtectedSurfaceVisitor::default();
    visitor.visit_stmt(statement);
    visitor
}

fn classify_protected_method_call(call: &syn::ExprMethodCall) -> Option<DeliveryStep> {
    match call.method.to_string().as_str() {
        "checkpoint_ready_event" if expression_is_path(&call.receiver, &["self", "progress"]) => {
            Some(DeliveryStep::AllocateEmissionOrdinal)
        }
        "observe" if expression_is_path(&call.receiver, &["self", "runtime"]) => {
            Some(DeliveryStep::ObserveRuntime)
        }
        "record_delivery" if expression_is_path(&call.receiver, &["self", "progress"]) => {
            Some(DeliveryStep::RecordDelivery)
        }
        "persist_state" if expression_is_path(&call.receiver, &["self"]) => {
            Some(DeliveryStep::PersistState)
        }
        "push" if expression_is_path(&call.receiver, &["observations"]) => {
            Some(DeliveryStep::AcceptObservation)
        }
        _ => None,
    }
}

fn protected_method_name(name: &str) -> bool {
    matches!(
        name,
        "checkpoint_ready_event"
            | "observe"
            | "record_delivery"
            | "persist_state"
            | "push"
            | "emit"
            | "send"
            | "adjudicate"
            | "shape_request"
    )
}

fn direct_protected_alias_target(expression: &syn::Expr) -> Option<String> {
    let expression = match expression {
        syn::Expr::Reference(reference) => reference.expr.as_ref(),
        syn::Expr::Group(group) => group.expr.as_ref(),
        syn::Expr::Paren(paren) => paren.expr.as_ref(),
        _ => expression,
    };
    let path = expression_path(expression)?;
    let protected_root = matches!(path.as_slice(), [self_name, root]
        if self_name == "self"
            && matches!(root.as_str(), "runtime" | "progress" | "operator_sink"))
        || path_has_forbidden_capability(&path)
        || path.last().is_some_and(|name| protected_method_name(name));
    protected_root.then(|| path.join("."))
}

fn path_has_forbidden_capability(path: &[String]) -> bool {
    path.iter().any(|segment| {
        matches!(
            segment.as_str(),
            "operator_sink" | "adjudication" | "adjudicator" | "delivery" | "persistence"
        )
    })
}

fn expression_is_path(expression: &syn::Expr, expected: &[&str]) -> bool {
    expression_path(expression)
        .is_some_and(|path| path.iter().map(String::as_str).eq(expected.iter().copied()))
}

fn path_is(path: &syn::Path, expected: &[&str]) -> bool {
    path.leading_colon.is_none()
        && path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .eq(expected.iter().copied())
}

fn expression_path(expression: &syn::Expr) -> Option<Vec<String>> {
    match expression {
        syn::Expr::Path(path) => Some(
            path.path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect(),
        ),
        syn::Expr::Field(field) => {
            let mut path = expression_path(&field.base)?;
            path.push(match &field.member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            });
            Some(path)
        }
        syn::Expr::Group(group) => expression_path(&group.expr),
        syn::Expr::Paren(paren) => expression_path(&paren.expr),
        _ => None,
    }
}

fn type_path(ty: &syn::Type) -> Option<Vec<String>> {
    match ty {
        syn::Type::Path(path) => Some(
            path.path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect(),
        ),
        syn::Type::Group(group) => type_path(&group.elem),
        syn::Type::Paren(paren) => type_path(&paren.elem),
        _ => None,
    }
}

#[test]
fn real_session_live_coordinator_emits_only_checkpoint_deltas_for_append_only_growth() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    let rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    fs::write(&rollout_path, first_rollout_phase()).expect("write first phase");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home.clone()),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create coordinator");

    let first = coordinator.poll_once().expect("first poll");
    assert!(first.reran_pipeline);
    assert!(!first.observations.is_empty());
    let first_latest_cursor = first
        .latest_cursor
        .clone()
        .expect("first poll should establish a cursor");
    assert_eq!(first_latest_cursor.session_id, "session-live");

    let second = coordinator.poll_once().expect("idle poll");
    assert!(!second.reran_pipeline);
    assert!(second.observations.is_empty());
    assert_eq!(second.latest_cursor.as_ref(), Some(&first_latest_cursor));

    fs::write(
        &rollout_path,
        format!("{}{}", first_rollout_phase(), second_rollout_phase()),
    )
    .expect("append second phase");

    let third = coordinator.poll_once().expect("growth poll");
    assert!(third.reran_pipeline);
    assert!(!third.observations.is_empty());
    assert!(third
        .observations
        .iter()
        .all(|observation| observation.event.cursor.session_id == "session-live"));
    assert!(third
        .observations
        .iter()
        .all(|observation| { observation.event.cursor.ordinal > first_latest_cursor.ordinal }));
    assert_eq!(
        coordinator.latest_cursor(),
        third
            .observations
            .last()
            .map(|observation| &observation.event.cursor)
    );
}

#[test]
fn real_session_live_coordinator_restores_progress_for_restart_idle_decision() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    let rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    fs::write(&rollout_path, first_rollout_phase()).expect("write first phase");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let first_latest_cursor = {
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home.clone()),
                session_id: "session-live".to_string(),
                state_dir: state_dir.clone(),
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create first coordinator");

        let first = coordinator.poll_once().expect("first poll");
        assert!(first.reran_pipeline);
        assert!(first.emitted_checkpoints > 0);
        let first_latest_cursor = first
            .latest_cursor
            .clone()
            .expect("first poll should establish a cursor");
        let next_emission_ordinal = first
            .observations
            .last()
            .expect("first poll should emit at least one observation")
            .event
            .emission_ordinal
            + 1;
        let persisted_state = read_persisted_state(&state_dir);
        assert_eq!(persisted_state["schema_version"].as_u64(), Some(3));
        assert_eq!(
            persisted_state["root_session_id"].as_str(),
            Some("session-live")
        );
        assert_eq!(
            persisted_state["progress"]["last_observed_size_bytes"].as_u64(),
            Some(first.observed_size_bytes)
        );
        assert!(persisted_state["progress"]["pending_observed_size_bytes"].is_null());
        assert_eq!(
            persisted_state["progress"]["last_delivered_cursors"]["session-live"]["session_id"]
                .as_str(),
            Some("session-live")
        );
        assert_eq!(
            persisted_state["progress"]["last_delivered_cursors"]["session-live"]["ordinal"]
                .as_u64(),
            Some(first_latest_cursor.ordinal as u64)
        );
        assert_eq!(
            persisted_state["progress"]["next_emission_ordinal"].as_u64(),
            Some(next_emission_ordinal as u64)
        );
        first_latest_cursor
    };

    let mut restarted = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home.clone()),
            session_id: "session-live".to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create restarted coordinator");

    let resumed_idle = restarted.poll_once().expect("restart poll");
    assert!(!resumed_idle.reran_pipeline);
    assert_eq!(
        resumed_idle.observed_size_bytes as usize,
        first_rollout_phase().len()
    );
    assert_eq!(resumed_idle.emitted_checkpoints, 0);
    assert!(resumed_idle.observations.is_empty());
    assert_eq!(
        resumed_idle.latest_cursor.as_ref(),
        Some(&first_latest_cursor)
    );
}

#[test]
fn real_session_live_coordinator_replays_pending_growth_after_interrupted_poll_restart() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    let rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    fs::write(&rollout_path, first_rollout_phase()).expect("write first phase");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let first_phase = {
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home.clone()),
                session_id: "session-live".to_string(),
                state_dir: state_dir.clone(),
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create coordinator");

        coordinator.poll_once().expect("first poll")
    };

    let helper_state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("helper-state");
    fs::create_dir_all(&helper_state_dir).expect("create helper state dir");
    fs::copy(
        state_dir.join("live-session-state.json").as_std_path(),
        helper_state_dir
            .join("live-session-state.json")
            .as_std_path(),
    )
    .expect("copy first persisted state");

    let full_rollout = format!(
        "{}{}{}",
        first_rollout_phase(),
        second_rollout_phase(),
        third_rollout_phase()
    );
    fs::write(&rollout_path, &full_rollout).expect("write full rollout");

    let growth_poll = {
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home.clone()),
                session_id: "session-live".to_string(),
                state_dir: helper_state_dir,
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create helper coordinator");

        coordinator.poll_once().expect("growth poll")
    };
    assert!(
        growth_poll.observations.len() >= 2,
        "fixture should create multiple fresh observations for interrupted-poll recovery"
    );

    let partially_delivered = growth_poll
        .observations
        .first()
        .expect("growth poll should emit observations");
    let interrupted_state = json!({
        "schema_version": 2,
        "session_id": "session-live",
        "progress": {
            "last_observed_size_bytes": first_phase.observed_size_bytes,
            "pending_observed_size_bytes": full_rollout.len(),
            "last_delivered_cursor": {
                "session_id": partially_delivered.event.cursor.session_id,
                "ordinal": partially_delivered.event.cursor.ordinal,
            },
            "next_emission_ordinal": partially_delivered.event.emission_ordinal + 1,
        }
    });
    write_persisted_state(&state_dir, &interrupted_state);

    let mut restarted = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create restarted coordinator");

    let resumed = restarted.poll_once().expect("resumed poll");
    assert!(resumed.reran_pipeline);
    assert_eq!(
        resumed.emitted_checkpoints,
        growth_poll.observations.len() - 1
    );
    assert_eq!(
        resumed
            .observations
            .first()
            .expect("resumed poll should emit remaining observations")
            .event
            .emission_ordinal,
        partially_delivered.event.emission_ordinal + 1
    );
    assert_eq!(resumed.latest_cursor, growth_poll.latest_cursor);

    let persisted_state = read_persisted_state(&state_dir);
    assert_eq!(
        persisted_state["progress"]["last_observed_size_bytes"].as_u64(),
        Some(full_rollout.len() as u64)
    );
    assert!(persisted_state["progress"]["pending_observed_size_bytes"].is_null());
}

#[test]
fn real_session_live_coordinator_keeps_polling_through_sparse_startup_until_analyzable() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    let rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    fs::write(&rollout_path, "").expect("write empty rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create coordinator");

    let empty_poll = coordinator
        .poll_once()
        .expect("empty rollout should stay pending");
    assert!(empty_poll.reran_pipeline);
    assert_eq!(empty_poll.emitted_checkpoints, 0);
    assert!(empty_poll.observations.is_empty());
    assert!(empty_poll.latest_cursor.is_none());
    assert_eq!(empty_poll.observed_size_bytes, 0);

    fs::write(&rollout_path, session_meta_only_rollout_phase()).expect("write session meta");

    let session_meta_poll = coordinator
        .poll_once()
        .expect("session_meta-only rollout should stay pending");
    assert!(session_meta_poll.reran_pipeline);
    assert_eq!(session_meta_poll.emitted_checkpoints, 0);
    assert!(session_meta_poll.observations.is_empty());
    assert!(session_meta_poll.latest_cursor.is_none());

    fs::write(
        &rollout_path,
        format!(
            "{}{}",
            session_meta_only_rollout_phase(),
            initial_analyzable_growth_phase()
        ),
    )
    .expect("append analyzable growth");

    let analyzable_poll = coordinator
        .poll_once()
        .expect("analyzable rollout should emit checkpoints");
    assert!(analyzable_poll.reran_pipeline);
    assert!(!analyzable_poll.observations.is_empty());
    assert!(analyzable_poll
        .latest_cursor
        .as_ref()
        .is_some_and(|cursor| cursor.session_id == "session-live"));
}

#[test]
fn real_session_live_coordinator_surfaces_posture_in_live_presentations() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    let rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    fs::write(&rollout_path, first_rollout_phase()).expect("write first phase");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create coordinator");

    let first_poll = coordinator.poll_once().expect("first poll");
    let posture_observation = first_poll
        .observations
        .iter()
        .find(|observation| observation.presentation.posture.is_some())
        .expect("real-session live seam should surface posture-bearing presentations");

    assert!(posture_observation
        .presentation
        .render_console_block(None)
        .contains("- Posture: "));
}

#[test]
fn real_session_live_coordinator_upgrades_valid_legacy_schema_v1_state_to_v3_progress() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    let rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    fs::write(&rollout_path, first_rollout_phase()).expect("write first phase");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let first_latest_cursor = {
        let mut coordinator = LiveSessionCoordinator::new(
            LiveSessionRequest {
                codex_home: Some(codex_home.clone()),
                session_id: "session-live".to_string(),
                state_dir: state_dir.clone(),
            },
            SchedulerPolicy::default(),
            WarningPolicy::default(),
        )
        .expect("create coordinator");

        coordinator
            .poll_once()
            .expect("first poll")
            .latest_cursor
            .expect("first poll should establish a cursor")
    };

    let legacy_state = json!({
        "schema_version": 1,
        "session_id": "session-live",
        "last_delivered_cursor": {
            "session_id": first_latest_cursor.session_id,
            "ordinal": first_latest_cursor.ordinal,
        }
    });
    write_persisted_state(&state_dir, &legacy_state);

    let mut restarted = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create restarted coordinator");
    assert_eq!(restarted.latest_cursor(), Some(&first_latest_cursor));

    let restored = restarted.poll_once().expect("legacy restored poll");
    assert!(restored.reran_pipeline);
    assert_eq!(restored.emitted_checkpoints, 0);
    assert!(restored.observations.is_empty());
    assert_eq!(restored.latest_cursor.as_ref(), Some(&first_latest_cursor));

    let upgraded_state = read_persisted_state(&state_dir);
    assert_eq!(upgraded_state["schema_version"].as_u64(), Some(3));
    assert_eq!(
        upgraded_state["root_session_id"].as_str(),
        Some("session-live")
    );
    assert_eq!(
        upgraded_state["progress"]["last_observed_size_bytes"].as_u64(),
        Some(first_rollout_phase().len() as u64)
    );
    assert!(upgraded_state["progress"]["pending_observed_size_bytes"].is_null());
    assert_eq!(
        upgraded_state["progress"]["last_delivered_cursors"]["session-live"]["session_id"].as_str(),
        Some("session-live")
    );
    assert_eq!(
        upgraded_state["progress"]["last_delivered_cursors"]["session-live"]["ordinal"].as_u64(),
        Some(first_latest_cursor.ordinal as u64)
    );
    assert_eq!(
        upgraded_state["progress"]["next_emission_ordinal"].as_u64(),
        Some(1)
    );
}

#[test]
fn real_session_live_coordinator_rejects_invalid_persisted_cursor_state() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    fs::write(
        rollout_dir.join("rollout-session-live.jsonl"),
        first_rollout_phase(),
    )
    .expect("write rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    fs::create_dir_all(&state_dir).expect("create state dir");
    fs::write(
        state_dir.join("live-session-state.json"),
        "{\"schema_version\":1,\"session_id\":\"other-session\",\"last_delivered_cursor\":{\"session_id\":\"other-session\",\"ordinal\":6}}\n",
    )
    .expect("write invalid state");

    let error = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect_err("mismatched persisted state should fail");

    assert!(error.to_string().contains("persisted live session state"));
}

#[test]
fn real_session_live_coordinator_rejects_persisted_cursor_outside_verified_closure() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    fs::write(
        rollout_dir.join("rollout-session-live.jsonl"),
        first_rollout_phase(),
    )
    .expect("write rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    write_persisted_state(
        &state_dir,
        &json!({
            "schema_version": 3,
            "root_session_id": "session-live",
            "progress": {
                "last_observed_size_bytes": null,
                "pending_observed_size_bytes": null,
                "last_delivered_cursors": {
                    "session-unexpected": {
                        "session_id": "session-unexpected",
                        "ordinal": 1
                    }
                },
                "next_emission_ordinal": 2
            }
        }),
    );

    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("persisted cursor is structurally valid before closure validation");

    let error = coordinator
        .poll_once()
        .expect_err("cursor outside analyzer-owned closure should fail closed");
    assert!(matches!(
        error,
        LiveSessionError::UnexpectedCheckpointSessions { .. }
    ));
}

#[test]
fn real_session_live_coordinator_rejects_persisted_child_cursor_ahead_of_analyzer_closure() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");

    let root_session_id = "session-live";
    let child_session_id = "session-child-a";
    fs::write(
        rollout_dir.join("rollout-session-live.jsonl"),
        linked_root_rollout(root_session_id, &[child_session_id]),
    )
    .expect("write linked root rollout");
    fs::write(
        rollout_dir.join("rollout-session-child-a.jsonl"),
        linked_child_rollout(root_session_id, child_session_id, first_rollout_phase()),
    )
    .expect("write linked child rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    write_persisted_state(
        &state_dir,
        &json!({
            "schema_version": 3,
            "root_session_id": root_session_id,
            "progress": {
                "last_observed_size_bytes": null,
                "pending_observed_size_bytes": null,
                "last_delivered_cursors": {
                    child_session_id: {
                        "session_id": child_session_id,
                        "ordinal": 999
                    }
                },
                "monitor_linked_closure": true,
                "next_emission_ordinal": 2
            }
        }),
    );

    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: root_session_id.to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("persisted child cursor is structurally valid before closure validation");

    let error = coordinator
        .poll_once()
        .expect_err("cursor ahead of analyzer-owned closure should fail closed");
    assert!(matches!(
        error,
        LiveSessionError::PersistedCursorAheadOfAnalyzerClosure {
            session_id,
            persisted_ordinal: 999,
            current_max_ordinal,
        } if session_id == child_session_id && current_max_ordinal < 999
    ));
}

#[test]
fn real_session_live_coordinator_rejects_ambiguous_rollout_artifacts() {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    fs::write(
        rollout_dir.join("rollout-session-live-a.jsonl"),
        first_rollout_phase(),
    )
    .expect("write rollout a");
    fs::write(
        rollout_dir.join("rollout-session-live-b.jsonl"),
        first_rollout_phase(),
    )
    .expect("write rollout b");

    let error = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: "session-live".to_string(),
            state_dir: Utf8Path::from_path(temp_dir.path())
                .expect("utf8 temp dir")
                .join("state"),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect_err("ambiguous rollout artifacts should fail");

    assert!(matches!(
        error,
        LiveSessionError::AmbiguousRolloutArtifacts { .. }
    ));
}

#[test]
fn real_session_live_coordinator_accepts_verified_children_and_advances_each_session_independently()
{
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");

    let root_session_id = "session-live";
    let first_child_session_id = "session-child-a";
    let second_child_session_id = "session-child-b";
    let root_rollout_path = rollout_dir.join("rollout-session-live.jsonl");
    let first_child_rollout_path = rollout_dir.join("rollout-session-child-a.jsonl");
    let second_child_rollout_path = rollout_dir.join("rollout-session-child-b.jsonl");
    fs::write(
        &root_rollout_path,
        linked_root_rollout(
            root_session_id,
            &[first_child_session_id, second_child_session_id],
        ),
    )
    .expect("write linked root rollout");
    fs::write(
        &first_child_rollout_path,
        linked_child_rollout(
            root_session_id,
            first_child_session_id,
            first_rollout_phase(),
        ),
    )
    .expect("write first linked child rollout");
    fs::write(
        &second_child_rollout_path,
        linked_child_rollout(
            root_session_id,
            second_child_session_id,
            first_rollout_phase(),
        ),
    )
    .expect("write second linked child rollout");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: root_session_id.to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create linked coordinator");

    let first = coordinator.poll_once().expect("first linked poll");
    assert!(first.reran_pipeline);
    assert!(first
        .observations
        .iter()
        .any(|observation| observation.event.cursor.session_id == root_session_id));
    assert!(first
        .observations
        .iter()
        .any(|observation| observation.event.cursor.session_id == first_child_session_id));
    assert!(first
        .observations
        .iter()
        .any(|observation| observation.event.cursor.session_id == second_child_session_id));
    let first_root_cursor = first
        .observations
        .iter()
        .rev()
        .find(|observation| observation.event.cursor.session_id == root_session_id)
        .map(|observation| observation.event.cursor.clone())
        .expect("root cursor");
    let first_child_cursor = first
        .observations
        .iter()
        .rev()
        .find(|observation| observation.event.cursor.session_id == first_child_session_id)
        .map(|observation| observation.event.cursor.clone())
        .expect("first child cursor");
    let second_child_cursor = first
        .observations
        .iter()
        .rev()
        .find(|observation| observation.event.cursor.session_id == second_child_session_id)
        .map(|observation| observation.event.cursor.clone())
        .expect("second child cursor");
    assert_eq!(first.latest_cursor.as_ref(), Some(&first_root_cursor));

    fs::write(
        &second_child_rollout_path,
        linked_child_rollout(
            root_session_id,
            second_child_session_id,
            &format!("{}{}", first_rollout_phase(), second_rollout_phase()),
        ),
    )
    .expect("append child-only growth");

    let child_growth = coordinator.poll_once().expect("child growth poll");
    assert!(child_growth.reran_pipeline);
    assert!(!child_growth.observations.is_empty());
    assert!(child_growth.observations.iter().all(|observation| {
        observation.event.cursor.session_id == second_child_session_id
            && observation.event.cursor.ordinal > second_child_cursor.ordinal
    }));
    assert_eq!(
        child_growth.latest_cursor.as_ref(),
        Some(&first_root_cursor)
    );

    let persisted_state = read_persisted_state(&state_dir);
    assert_eq!(persisted_state["schema_version"].as_u64(), Some(3));
    assert_eq!(
        persisted_state["root_session_id"].as_str(),
        Some(root_session_id)
    );
    assert_eq!(
        persisted_state["progress"]["last_delivered_cursors"][root_session_id]["ordinal"].as_u64(),
        Some(first_root_cursor.ordinal as u64)
    );
    assert_eq!(
        persisted_state["progress"]["last_delivered_cursors"][first_child_session_id]["ordinal"]
            .as_u64(),
        Some(first_child_cursor.ordinal as u64)
    );
    assert!(
        persisted_state["progress"]["last_delivered_cursors"][second_child_session_id]["ordinal"]
            .as_u64()
            > Some(second_child_cursor.ordinal as u64)
    );
}

#[test]
fn real_session_live_coordinator_fails_closed_on_root_cursor_regression_after_late_verified_child_discovery(
) {
    let temp_dir = TempDir::new().expect("temp dir");
    let codex_home = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join(".codex");
    let rollout_dir = codex_home.join("sessions/2026/06/01");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");

    let root_session_id = "session-live";
    let child_session_id = "session-child-late";
    fs::write(
        rollout_dir.join("rollout-session-live.jsonl"),
        linked_root_rollout(root_session_id, &[child_session_id]),
    )
    .expect("write root rollout before child artifact exists");

    let state_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp dir")
        .join("state");
    let mut coordinator = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home.clone()),
            session_id: root_session_id.to_string(),
            state_dir: state_dir.clone(),
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("create coordinator");

    let root_only = coordinator.poll_once().expect("root-only poll");
    assert!(root_only.reran_pipeline);
    assert!(root_only
        .observations
        .iter()
        .all(|observation| observation.event.cursor.session_id == root_session_id));
    drop(coordinator);

    fs::write(
        rollout_dir.join("rollout-session-child-late.jsonl"),
        linked_child_rollout(root_session_id, child_session_id, first_rollout_phase()),
    )
    .expect("write late child rollout");

    let mut restarted = LiveSessionCoordinator::new(
        LiveSessionRequest {
            codex_home: Some(codex_home),
            session_id: root_session_id.to_string(),
            state_dir,
        },
        SchedulerPolicy::default(),
        WarningPolicy::default(),
    )
    .expect("restart coordinator while linked closure is pending");
    let error = restarted
        .poll_once()
        .expect_err("late closure expansion must not swallow a regressed root cursor");
    assert!(matches!(
        error,
        LiveSessionError::PersistedCursorAheadOfAnalyzerClosure {
            session_id,
            persisted_ordinal: 2,
            current_max_ordinal: 1,
        } if session_id == root_session_id
    ));
}

fn linked_root_rollout(root_session_id: &str, child_session_ids: &[&str]) -> String {
    let mut rollout = first_rollout_phase().replace("session-live", root_session_id);
    for (index, child_session_id) in child_session_ids.iter().enumerate() {
        rollout.push_str(&format!(
            "{{\"type\":\"response_item\",\"payload\":{{\"type\":\"function_call\",\"name\":\"spawn_agent\",\"call_id\":\"call-spawn-child-{index}\"}}}}\n{{\"type\":\"response_item\",\"payload\":{{\"type\":\"function_call_output\",\"call_id\":\"call-spawn-child-{index}\",\"output\":\"{{\\\"agent_id\\\":\\\"{child_session_id}\\\"}}\"}}}}\n"
        ));
    }
    rollout
}

fn linked_child_rollout(root_session_id: &str, child_session_id: &str, rollout: &str) -> String {
    let source = json!({
        "subagent": {
            "thread_spawn": {
                "parent_thread_id": root_session_id,
                "depth": 1,
                "agent_nickname": "agent-child",
                "agent_role": "default"
            }
        }
    });
    rollout
        .replace("session-live", child_session_id)
        .lines()
        .map(|line| {
            let mut value: Value = serde_json::from_str(line).expect("parse rollout line");
            if value["type"] == "session_meta" {
                value["payload"]["source"] = source.clone();
            }
            format!(
                "{}\n",
                serde_json::to_string(&value).expect("encode rollout line")
            )
        })
        .collect()
}

fn first_rollout_phase() -> &'static str {
    concat!(
        "{\"timestamp\":\"2026-06-01T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-live\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:01Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:02Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-1\",\"user_instructions\":\"Repo-local rules\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:03Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"/goal Implement Packet 18 with docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md and crates/agent-drift-sentinel/src/cli.rs in scope\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:03.001Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"turn_id\":\"turn-1\",\"message\":\"/goal Implement Packet 18 with docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md and crates/agent-drift-sentinel/src/cli.rs in scope\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:04Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel -- --nocapture\\\",\\\"workdir\\\":\\\"/repo\\\"}\",\"call_id\":\"call-1\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:05Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-1\",\"output\":\"ok\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:06Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"First phase complete\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:06.001Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"First phase complete\"}]}}\n"
    )
}

fn second_rollout_phase() -> &'static str {
    concat!(
        "{\"timestamp\":\"2026-06-01T12:00:07Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_complete\",\"turn_id\":\"turn-1\",\"last_agent_message\":\"First phase complete\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:08Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-2\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:09Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-2\",\"user_instructions\":\"Follow the packet acceptance criteria exactly\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:10Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"also prove the live command on the active session and update docs/specs/hybrid-drift-sentinel-implementation-order.md\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:10.001Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"turn_id\":\"turn-2\",\"message\":\"also prove the live command on the active session and update docs/specs/hybrid-drift-sentinel-implementation-order.md\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:11Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel live_runtime -- --nocapture\\\",\\\"workdir\\\":\\\"/repo\\\"}\",\"call_id\":\"call-2\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:12Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-2\",\"output\":\"ok\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:13Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Second phase complete\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:13.001Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Second phase complete\"}]}}\n"
    )
}

fn third_rollout_phase() -> &'static str {
    concat!(
        "{\"timestamp\":\"2026-06-01T12:00:14Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_complete\",\"turn_id\":\"turn-2\",\"last_agent_message\":\"Second phase complete\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:15Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-3\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:16Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-3\",\"user_instructions\":\"Keep the restart continuity seam bounded to real_session_live.rs\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:17Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"only fix the flagged review issue in real_session_live.rs and preserve the packet boundary\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:17.001Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"turn_id\":\"turn-3\",\"message\":\"only fix the flagged review issue in real_session_live.rs and preserve the packet boundary\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:18Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel real_session_live -- --nocapture\\\",\\\"workdir\\\":\\\"/repo\\\"}\",\"call_id\":\"call-3\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:19Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-3\",\"output\":\"ok\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:20Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Third phase complete\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:20.001Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Third phase complete\"}]}}\n"
    )
}

fn read_persisted_state(state_dir: &Utf8Path) -> Value {
    serde_json::from_str(
        &fs::read_to_string(state_dir.join("live-session-state.json"))
            .expect("read persisted state"),
    )
    .expect("parse persisted state json")
}

fn write_persisted_state(state_dir: &Utf8Path, state: &Value) {
    fs::create_dir_all(state_dir).expect("create state dir");
    fs::write(
        state_dir.join("live-session-state.json"),
        serde_json::to_vec_pretty(state).expect("encode persisted state"),
    )
    .expect("write persisted state");
}

fn session_meta_only_rollout_phase() -> &'static str {
    "{\"timestamp\":\"2026-06-01T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-live\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n"
}

fn initial_analyzable_growth_phase() -> &'static str {
    concat!(
        "{\"timestamp\":\"2026-06-01T12:00:01Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-1\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:02Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-1\",\"user_instructions\":\"Repo-local rules\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:03Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"/goal Implement Packet 18 with docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md and crates/agent-drift-sentinel/src/cli.rs in scope\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:03.001Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"turn_id\":\"turn-1\",\"message\":\"/goal Implement Packet 18 with docs/specs/agent-drift-sentinel-real-session-live-v0.5-spec.md and crates/agent-drift-sentinel/src/cli.rs in scope\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:04Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"functions.shell_command\",\"arguments\":\"{\\\"command\\\":\\\"cargo test -p agent-drift-sentinel -- --nocapture\\\",\\\"workdir\\\":\\\"/repo\\\"}\",\"call_id\":\"call-1\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:05Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-1\",\"output\":\"ok\"}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:06Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"First phase complete\"}]}}\n",
        "{\"timestamp\":\"2026-06-01T12:00:06.001Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"First phase complete\"}]}}\n"
    )
}
