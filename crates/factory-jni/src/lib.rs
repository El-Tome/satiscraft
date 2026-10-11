//! JNI bridge between the NeoForge mod and [`factory_core`].
//!
//! This crate holds no game rules: only translation between JNI and the core
//! contract, panic handling and threading. `unsafe` code is denied everywhere
//! except in the [`exports`] module, which only carries what JNI imposes.

#![deny(unsafe_code)]

mod exports;
mod policy;
