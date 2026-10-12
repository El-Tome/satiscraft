//! Everything that can go wrong in a JNI call short of a panic.

use std::fmt;

use factory_core::LoadError;
use factory_core::contract::CodecError;

use crate::registry::RegistryError;

/// An error returned by a native method, thrown to the JVM as `FactoryCoreException`.
#[derive(Debug)]
pub(crate) enum BridgeError {
    /// The JVM refused an operation: reading an argument, allocating the result.
    Jni(jni::errors::Error),
    /// The id does not reach a healthy simulation.
    Registry(RegistryError),
    /// An argument is not the postcard encoding it should be, or a result cannot be encoded.
    Codec {
        what: &'static str,
        source: CodecError,
    },
    /// The save does not load.
    Load(LoadError),
}

impl BridgeError {
    /// Wraps a codec error with the value it concerns, for `map_err`.
    pub(crate) fn codec(what: &'static str) -> impl FnOnce(CodecError) -> Self {
        move |source| Self::Codec { what, source }
    }
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Jni(err) => write!(f, "JNI call failed: {err}"),
            Self::Registry(err) => err.fmt(f),
            Self::Codec { what, source } => write!(f, "{what}: {source}"),
            Self::Load(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for BridgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Jni(err) => Some(err),
            Self::Registry(err) => Some(err),
            Self::Codec { source, .. } => Some(source),
            Self::Load(err) => Some(err),
        }
    }
}

impl From<jni::errors::Error> for BridgeError {
    fn from(err: jni::errors::Error) -> Self {
        Self::Jni(err)
    }
}

impl From<RegistryError> for BridgeError {
    fn from(err: RegistryError) -> Self {
        Self::Registry(err)
    }
}

impl From<LoadError> for BridgeError {
    fn from(err: LoadError) -> Self {
        Self::Load(err)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    /// Each kind of error, with the message the JVM gets after the policy prefix.
    fn cases() -> Vec<(BridgeError, &'static str)> {
        vec![
            (
                jni::errors::Error::NullPtr("commands").into(),
                "JNI call failed: ",
            ),
            (
                RegistryError::Unknown(3).into(),
                "no simulation 3: never created or already shut down",
            ),
            (
                BridgeError::codec("commands")(CodecError::TrailingBytes(2)),
                "commands: cannot decode: 2 bytes left after the value",
            ),
            (
                LoadError::Malformed(CodecError::TrailingBytes(1)).into(),
                "malformed save: cannot decode: 1 bytes left after the value",
            ),
        ]
    }

    #[test]
    fn messages_say_what_failed() {
        for (err, message) in cases() {
            assert!(err.to_string().starts_with(message), "{err}");
        }
    }

    #[test]
    fn every_error_keeps_its_cause() {
        for (err, _) in cases() {
            let source = err.source().expect("every variant wraps a cause");
            assert!(err.to_string().ends_with(&source.to_string()), "{err}");
        }
    }
}
