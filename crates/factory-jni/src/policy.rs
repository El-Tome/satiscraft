//! Maps errors and panics from native methods to a Kotlin `FactoryCoreException`.

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};

use jni::Env;
use jni::errors::ErrorPolicy;
use jni::strings::JNIString;

/// JNI name of the exception thrown to the JVM for any error or panic.
pub(crate) const EXCEPTION_CLASS: &str = "io/github/eltome/satiscraft/core/FactoryCoreException";

/// Error policy for every export: throws `FactoryCoreException` and returns a
/// default value (`null`, `0`, `false`) to the JVM.
///
/// If a Java exception is already pending, it takes precedence and nothing new is thrown.
#[derive(Debug, Default)]
pub(crate) struct ThrowFactoryCoreException;

impl<T: Default, E: std::error::Error> ErrorPolicy<T, E> for ThrowFactoryCoreException {
    type Captures<'unowned_env_local: 'native_method, 'native_method> = ();

    fn on_error<'unowned_env_local: 'native_method, 'native_method>(
        env: &mut Env<'unowned_env_local>,
        _captures: &mut Self::Captures<'unowned_env_local, 'native_method>,
        err: E,
    ) -> jni::errors::Result<T> {
        throw(env, &error_message(&err));
        Ok(T::default())
    }

    fn on_panic<'unowned_env_local: 'native_method, 'native_method>(
        env: &mut Env<'unowned_env_local>,
        _captures: &mut Self::Captures<'unowned_env_local, 'native_method>,
        payload: Box<dyn Any + Send + 'static>,
    ) -> jni::errors::Result<T> {
        throw(env, &panic_message(payload));
        Ok(T::default())
    }
}

/// Throws `FactoryCoreException` unless an exception is already pending.
fn throw(env: &mut Env<'_>, message: &str) {
    if env.exception_check() {
        return;
    }
    // `throw_new` reports the freshly pending exception as an `Err`: that is the
    // expected outcome here, the exception propagates once we return to the JVM.
    let _ = env.throw_new(JNIString::from(EXCEPTION_CLASS), JNIString::from(message));
}

/// Formats a Rust error crossing the JNI boundary.
fn error_message(err: &dyn std::error::Error) -> String {
    format!("factory-core error: {err}")
}

/// Extracts a readable message from a panic payload, without letting a payload
/// whose `Drop` panics escape.
fn panic_message(payload: Box<dyn Any + Send + 'static>) -> String {
    let detail = if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    };
    if let Err(drop_panic) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
        std::mem::forget(drop_panic);
    }
    format!("factory-core panic: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exception_class_matches_kotlin_package() {
        assert_eq!(
            EXCEPTION_CLASS.replace('/', "."),
            "io.github.eltome.satiscraft.core.FactoryCoreException"
        );
    }

    #[test]
    fn error_message_includes_source() {
        let err = std::io::Error::other("disk full");
        assert_eq!(error_message(&err), "factory-core error: disk full");
    }

    #[test]
    fn panic_message_reads_static_str() {
        let payload = catch_unwind(|| panic!("boom")).unwrap_err();
        assert_eq!(panic_message(payload), "factory-core panic: boom");
    }

    #[test]
    fn panic_message_reads_formatted_string() {
        let island = 7;
        let payload = catch_unwind(|| panic!("island {island} faulted")).unwrap_err();
        assert_eq!(
            panic_message(payload),
            "factory-core panic: island 7 faulted"
        );
    }

    #[test]
    fn panic_message_handles_other_payloads() {
        let payload: Box<dyn Any + Send> = Box::new(42_u8);
        assert_eq!(
            panic_message(payload),
            "factory-core panic: non-string panic payload"
        );
    }
}
