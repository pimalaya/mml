#[cfg(feature = "cli")]
use pimalaya_toolbox::build::{features_env, git_envs, target_envs};

#[cfg(feature = "cli")]
fn main() {
    features_env(include_str!("./Cargo.toml"));
    target_envs();
    git_envs();
}

#[cfg(not(feature = "cli"))]
fn main() {
    // nothing to do
}
