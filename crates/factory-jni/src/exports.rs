//! Native method symbols looked up by the JVM.
//!
//! This is the only module allowed to use `unsafe` code. The single reason:
//! the JVM resolves native methods by their mangled name, which requires
//! `#[unsafe(no_mangle)]` in Rust 2024. Each export must stay a thin wrapper:
//! upgrade the env with `with_env` (which catches panics), call safe code,
//! resolve through [`ThrowFactoryCoreException`]. Any other `unsafe` here needs
//! a justification comment and explicit approval in the PR.

#![allow(unsafe_code)]

use jni::EnvUnowned;
use jni::objects::{JClass, JString};
use jni::sys::jint;

use crate::policy::ThrowFactoryCoreException;

/// `NativeBridge.coreVersion(): String`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_coreVersion<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
) -> JString<'caller> {
    unowned_env
        .with_env(|env| JString::from_str(env, factory_core::VERSION))
        .resolve::<ThrowFactoryCoreException>()
}

/// `NativeBridge.contractVersion(): Int`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_contractVersion<
    'caller,
>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
) -> jint {
    unowned_env
        .with_env(|_env| -> jni::errors::Result<jint> {
            Ok(jint::from(factory_core::CONTRACT_VERSION))
        })
        .resolve::<ThrowFactoryCoreException>()
}
