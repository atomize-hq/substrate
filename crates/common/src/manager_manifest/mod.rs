mod resolver;
mod schema;
mod validator;

pub use schema::{
    DetectSpec, GuestSpec, InitSpec, InstallSpec, ManagerManifest, ManagerSpec, Platform,
    RegexPattern,
};

#[cfg(test)]
mod tests;
