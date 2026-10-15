//! Server settings, from `config.toml`: how the simulation runs and when factories do.

use std::fmt;
use std::num::NonZeroU32;

use serde::Deserialize;
use toml::Spanned;

use super::error::ContentError;
use super::sources::Sources;

/// The engine's server settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub sim: Sim,
    pub activity: Activity,
}

/// How the simulation runs: `[sim]`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sim {
    /// Threads of the simulation pool; 0 for one per CPU core. Needs a restart.
    pub threads: u16,
    /// Time the core may spend on one server tick, in milliseconds: past it, the least urgent
    /// aggregated islands wait for the next tick.
    pub tick_budget_ms: NonZeroU32,
}

/// When teams' factories run: `[activity]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    /// The policy a new team starts with, one of `allowed`.
    pub default: ActivityPolicy,
    /// The policies a team may choose at its HUB.
    pub allowed: Vec<ActivityPolicy>,
}

/// Whether a team's factories run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityPolicy {
    /// Always, members connected or not.
    Always,
    /// While at least one member is connected, with no catching up afterwards.
    TeamOnline,
    /// Never: everything is on hold.
    Paused,
}

impl ActivityPolicy {
    /// The name content files use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::TeamOnline => "team_online",
            Self::Paused => "paused",
        }
    }
}

impl fmt::Display for ActivityPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// `config.toml` as written, defaults and overrides merged.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConfigFile {
    sim: Sim,
    activity: ActivityFile,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActivityFile {
    default: Spanned<ActivityPolicy>,
    allowed: Vec<ActivityPolicy>,
}

impl ConfigFile {
    /// The settings, once checked together; problems go to `errors`.
    pub(super) fn validate(self, sources: &Sources<'_>, errors: &mut Vec<ContentError>) -> Config {
        let ActivityFile { default, allowed } = self.activity;
        if !allowed.contains(default.get_ref()) {
            let allowed: Vec<_> = allowed.iter().map(|policy| policy.name()).collect();
            let message = format!(
                "the default activity policy `{}` is not among the allowed ones [{}]",
                default.get_ref(),
                allowed.join(", ")
            );
            errors.push(sources.error(default.span(), message));
        }
        Config {
            sim: self.sim,
            activity: Activity {
                default: default.into_inner(),
                allowed,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policies_are_named_as_in_the_files() {
        for policy in [
            ActivityPolicy::Always,
            ActivityPolicy::TeamOnline,
            ActivityPolicy::Paused,
        ] {
            let read: ActivityPolicy = toml::from_str::<toml::Table>(&format!("p = '{policy}'"))
                .unwrap()["p"]
                .clone()
                .try_into()
                .unwrap();
            assert_eq!(read, policy);
        }
        assert_eq!(ActivityPolicy::TeamOnline.to_string(), "team_online");
    }
}
