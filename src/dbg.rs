use structopt::clap::crate_version;

pub fn dbg_info() -> String {
    format!(
        "Crate version {}.\nBuilt from commit {} by {} for target {} with profile '{}'.",
        crate_version!(),
        env!("BUILD_GIT_COMMIT"),
        env!("BUILD_RUSTC_VERSION"),
        env!("BUILD_TARGET"),
        env!("BUILD_PROFILE"),
    )
}
