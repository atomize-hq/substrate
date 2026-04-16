//! Internal pack compiler seam.

#![allow(dead_code)]
#![allow(unused_imports)]

pub(crate) mod builtin;
pub(crate) mod compiler;
pub(crate) mod diagnostics;
pub(crate) mod error;
pub(crate) mod names;
pub(crate) mod refs;
pub(crate) mod schema;
pub(crate) mod source;

pub(crate) mod compiled;
pub(crate) mod raw;

pub(crate) use compiled::{
    CompiledAnalysisDefaults, CompiledPackHeader, CompiledPathClasses, CompiledProfile,
    CompiledProfileApps, CompiledProfileIncludes, CompiledProfileScore, CompiledProfileTopology,
};
pub(crate) use compiler::PackCompiler;
pub(crate) use diagnostics::{PackDiagnostic, PackLocation, PackRelatedLocation};
pub(crate) use error::{PackError, PackResult};
pub(crate) use names::{AppName, LanguageId, PackName};
pub(crate) use raw::PackKind;
pub(crate) use refs::{PackFileRef, PackRef};
pub(crate) use schema::{
    PACK_COMMON_V1_SCHEMA_FILE, PACK_COMMON_V1_SCHEMA_ID, PACK_COMMON_V1_SCHEMA_JSON,
    PACK_COMMON_V1_SCHEMA_VERSION, PACK_PROFILE_V1_SCHEMA_FILE, PACK_PROFILE_V1_SCHEMA_ID,
    PACK_PROFILE_V1_SCHEMA_JSON, PACK_PROFILE_V1_SCHEMA_VERSION,
};
pub(crate) use source::{PackFormat, PackOrigin, PackSource};
