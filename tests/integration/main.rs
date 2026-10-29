#[cfg(unix)]
mod broken_pipe;
mod codebase;
mod commands;
mod hook_tools;
mod pre_commit_config;
mod pre_commit_install;
mod skills_install;
mod skills_update;
mod smells;
#[cfg(unix)]
mod unreadable_dir;
mod vendor_paths;
