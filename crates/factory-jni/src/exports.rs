//! Native method symbols looked up by the JVM.
//!
//! This is the only module allowed to use `unsafe` code. The single reason:
//! the JVM resolves native methods by their mangled name, which requires
//! `#[unsafe(no_mangle)]` in Rust 2024. Each export must stay a thin wrapper:
//! upgrade the env with `with_env` (which catches panics), call safe code,
//! resolve through [`ThrowFactoryCoreException`]. Any other `unsafe` here needs
//! a justification comment and explicit approval in the PR.

#![allow(unsafe_code)]

use factory_core::contract::reencode;
use jni::EnvUnowned;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jint, jlong};

use crate::error::BridgeError;
use crate::policy::ThrowFactoryCoreException;
use crate::simulations;

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

/// `NativeBridge.init(content: ByteArray): Long`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_init<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    content: JByteArray<'caller>,
) -> jlong {
    unowned_env
        .with_env(|env| -> Result<jlong, BridgeError> {
            simulations::init(&env.convert_byte_array(&content)?)
        })
        .resolve::<ThrowFactoryCoreException>()
}

/// `NativeBridge.load(content: ByteArray, save: ByteArray): Long`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_load<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    content: JByteArray<'caller>,
    save: JByteArray<'caller>,
) -> jlong {
    unowned_env
        .with_env(|env| -> Result<jlong, BridgeError> {
            let content = env.convert_byte_array(&content)?;
            simulations::load(&content, &env.convert_byte_array(&save)?)
        })
        .resolve::<ThrowFactoryCoreException>()
}

/// `NativeBridge.submit(id: Long, commands: ByteArray): ByteArray`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_submit<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    id: jlong,
    commands: JByteArray<'caller>,
) -> JByteArray<'caller> {
    unowned_env
        .with_env(|env| -> Result<_, BridgeError> {
            let results = simulations::submit(id, &env.convert_byte_array(&commands)?)?;
            Ok(env.byte_array_from_slice(&results)?)
        })
        .resolve::<ThrowFactoryCoreException>()
}

/// `NativeBridge.tick(id: Long): ByteArray`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_tick<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    id: jlong,
) -> JByteArray<'caller> {
    unowned_env
        .with_env(|env| -> Result<_, BridgeError> {
            Ok(env.byte_array_from_slice(&simulations::tick(id)?)?)
        })
        .resolve::<ThrowFactoryCoreException>()
}

/// `NativeBridge.query(id: Long, query: ByteArray): ByteArray`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_query<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    id: jlong,
    query: JByteArray<'caller>,
) -> JByteArray<'caller> {
    unowned_env
        .with_env(|env| -> Result<_, BridgeError> {
            let answer = simulations::query(id, &env.convert_byte_array(&query)?)?;
            Ok(env.byte_array_from_slice(&answer)?)
        })
        .resolve::<ThrowFactoryCoreException>()
}

/// `NativeBridge.save(id: Long): ByteArray`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_save<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    id: jlong,
) -> JByteArray<'caller> {
    unowned_env
        .with_env(|env| -> Result<_, BridgeError> {
            Ok(env.byte_array_from_slice(&simulations::save(id)?)?)
        })
        .resolve::<ThrowFactoryCoreException>()
}

/// `NativeBridge.shutdown(id: Long)`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_shutdown<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    id: jlong,
) {
    unowned_env
        .with_env(|_env| simulations::shutdown(id))
        .resolve::<ThrowFactoryCoreException>();
}

/// `NativeBridge.reencode(type: String, bytes: ByteArray): ByteArray`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_eltome_satiscraft_core_NativeBridge_reencode<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    type_name: JString<'caller>,
    bytes: JByteArray<'caller>,
) -> JByteArray<'caller> {
    unowned_env
        .with_env(|env| -> Result<_, BridgeError> {
            let type_name = type_name.try_to_string(env)?;
            let bytes = env.convert_byte_array(&bytes)?;
            let again = reencode(&type_name, &bytes).map_err(BridgeError::codec("value"))?;
            Ok(env.byte_array_from_slice(&again)?)
        })
        .resolve::<ThrowFactoryCoreException>()
}
