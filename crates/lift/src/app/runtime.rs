//! Phase D runtime bootstrap boundary.

use crate::pack::{CompiledPackSet, PackCompiler, PackResult, PackSource};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct ProfileBootstrap {
    pub bundle: CompiledPackSet,
}

impl ProfileBootstrap {
    #[allow(dead_code)]
    pub(crate) fn from_pack_set(bundle: CompiledPackSet) -> Self {
        Self { bundle }
    }
}

#[allow(dead_code)]
pub(crate) fn bootstrap_profile(source: PackSource) -> PackResult<ProfileBootstrap> {
    let compiler = PackCompiler::new();
    let profile = compiler.compile_profile(source)?;
    let bundle = compiler.resolve_profile_pack_set(&profile)?;
    Ok(ProfileBootstrap::from_pack_set(bundle))
}
