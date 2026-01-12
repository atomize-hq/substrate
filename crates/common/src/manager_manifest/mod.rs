mod resolver;
mod schema;
mod validator;

pub use schema::{
    DetectSpec, GuestSpec, InitSpec, InstallClass, InstallSpec, ManagerManifest, ManagerSpec,
    Platform, RegexPattern, SystemPackagesSpec, MANAGER_MANIFEST_VERSION,
};

#[cfg(test)]
mod tests;
