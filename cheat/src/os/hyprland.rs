//! Talk to the Hyprland compositor over its socket commands (`hyprctl`).
//!
//! Wayland has no protocol for "always on top" or "click through", so under
//! Hyprland the overlay needs window rules. Instead of requiring the user to
//! edit `hyprland.conf` by hand, we push temporary layer rules through the
//! IPC socket while the overlay lives (the same effect the fork achieves with
//! static config entries, but self-contained and removed on exit).

use std::process::Command;

/// Title of the overlay window, kept in sync with `WindowContext`.
pub const OVERLAY_TITLE: &str = "deadlocked_overlay";

const LAYER_RULES: [&str; 3] = [
    // draw above every other window, including fullscreen games
    &format!("layer:^{OVERLAY_TITLE}$,pin"),
    // do not steal keyboard focus from the game
    &format!("layer:^{OVERLAY_TITLE}$,nofocus"),
    // let clicks fall through onto the game
    &format!("layer:^{OVERLAY_TITLE}$,passmouseevents"),
];

fn hyprctl(args: &[&str]) -> Result<(), String> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .map_err(|err| format!("failed to run hyprctl: {err}"))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "hyprctl {} failed: {}",
            args.join(" "),
            error.trim()
        ));
    }

    Ok(())
}

/// Whether we are running inside a Hyprland session.
pub fn is_hyprland() -> bool {
    std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some_and(|signature| {
        !signature.is_empty()
    }) || std::env::var_os("DESKTOP_SESSION")
        .is_some_and(|session| session.to_string_lossy().eq_ignore_ascii_case("hyprland"))
}

/// Applies the overlay window rules. Missing rules are added, existing ones updated.
pub fn apply_overlay_rules() -> Result<(), String> {
    for rule in LAYER_RULES {
        hyprctl(&["layer-opts", "-r", rule])?;
    }
    utils::info!("applied hyprland overlay window rules");
    Ok(())
}

/// Removes the overlay window rules again.
pub fn remove_overlay_rules() {
    let _ = hyprctl(&["layer-opts", "-i", "all"]);
}
