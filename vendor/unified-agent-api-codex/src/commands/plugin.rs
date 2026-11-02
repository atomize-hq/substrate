use std::ffi::OsString;

use crate::{
    ApplyDiffArtifacts, CodexClient, CodexError, PluginCommandRequest, PluginHelpRequest,
    PluginMarketplaceAddRequest, PluginMarketplaceCommandRequest, PluginMarketplaceHelpRequest,
    PluginMarketplaceRemoveRequest, PluginMarketplaceUpgradeRequest,
};

impl CodexClient {
    /// Runs `codex plugin` and returns captured output.
    pub async fn plugin(
        &self,
        request: PluginCommandRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        self.run_simple_command_with_overrides(vec![OsString::from("plugin")], request.overrides)
            .await
    }

    /// Runs `codex plugin help [COMMAND]...` and returns captured output.
    pub async fn plugin_help(
        &self,
        request: PluginHelpRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let PluginHelpRequest { command, overrides } = request;
        let mut args = vec![OsString::from("plugin"), OsString::from("help")];
        args.extend(command.into_iter().map(OsString::from));
        self.run_simple_command_with_overrides(args, overrides)
            .await
    }

    /// Runs `codex plugin marketplace` and returns captured output.
    pub async fn plugin_marketplace(
        &self,
        request: PluginMarketplaceCommandRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        self.run_simple_command_with_overrides(
            vec![OsString::from("plugin"), OsString::from("marketplace")],
            request.overrides,
        )
        .await
    }

    /// Runs `codex plugin marketplace help [COMMAND]...` and returns captured output.
    pub async fn plugin_marketplace_help(
        &self,
        request: PluginMarketplaceHelpRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let PluginMarketplaceHelpRequest { command, overrides } = request;
        let mut args = vec![
            OsString::from("plugin"),
            OsString::from("marketplace"),
            OsString::from("help"),
        ];
        args.extend(command.into_iter().map(OsString::from));
        self.run_simple_command_with_overrides(args, overrides)
            .await
    }

    /// Runs `codex plugin marketplace add <SOURCE>` and returns captured output.
    pub async fn plugin_marketplace_add(
        &self,
        request: PluginMarketplaceAddRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let PluginMarketplaceAddRequest {
            source,
            source_ref,
            sparse_path,
            overrides,
        } = request;

        let mut args = vec![
            OsString::from("plugin"),
            OsString::from("marketplace"),
            OsString::from("add"),
        ];
        if let Some(source_ref) = source_ref {
            args.push(OsString::from("--ref"));
            args.push(OsString::from(source_ref));
        }
        if let Some(sparse_path) = sparse_path {
            args.push(OsString::from("--sparse"));
            args.push(OsString::from(sparse_path));
        }
        args.push(OsString::from(source));

        self.run_simple_command_with_overrides(args, overrides)
            .await
    }

    /// Runs `codex plugin marketplace remove <MARKETPLACE_NAME>` and returns captured output.
    pub async fn plugin_marketplace_remove(
        &self,
        request: PluginMarketplaceRemoveRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let PluginMarketplaceRemoveRequest {
            marketplace_name,
            overrides,
        } = request;

        self.run_simple_command_with_overrides(
            vec![
                OsString::from("plugin"),
                OsString::from("marketplace"),
                OsString::from("remove"),
                OsString::from(marketplace_name),
            ],
            overrides,
        )
        .await
    }

    /// Runs `codex plugin marketplace upgrade [MARKETPLACE_NAME]` and returns captured output.
    pub async fn plugin_marketplace_upgrade(
        &self,
        request: PluginMarketplaceUpgradeRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let PluginMarketplaceUpgradeRequest {
            marketplace_name,
            overrides,
        } = request;

        let mut args = vec![
            OsString::from("plugin"),
            OsString::from("marketplace"),
            OsString::from("upgrade"),
        ];
        if let Some(marketplace_name) = marketplace_name {
            args.push(OsString::from(marketplace_name));
        }

        self.run_simple_command_with_overrides(args, overrides)
            .await
    }
}
