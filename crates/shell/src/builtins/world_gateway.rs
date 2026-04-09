use crate::execution::{WorldGatewayAction, WorldGatewayCmd, WorldGatewayStatusArgs};

const EXIT_COMPONENT_UNAVAILABLE: i32 = 4;

pub fn run(cmd: &WorldGatewayCmd) -> i32 {
    match &cmd.action {
        WorldGatewayAction::Sync => emit_unavailable("substrate world gateway sync"),
        WorldGatewayAction::Status(args) => run_status(args),
        WorldGatewayAction::Restart => emit_unavailable("substrate world gateway restart"),
    }
}

fn run_status(args: &WorldGatewayStatusArgs) -> i32 {
    if args.json {
        emit_unavailable("substrate world gateway status --json")
    } else {
        emit_unavailable("substrate world gateway status")
    }
}

fn emit_unavailable(command: &str) -> i32 {
    eprintln!("{command}: unavailable (required gateway/world component unavailable)");
    EXIT_COMPONENT_UNAVAILABLE
}
