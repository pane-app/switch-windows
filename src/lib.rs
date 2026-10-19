//! Pane's Switch Windows, a default extension (ADR 0040): a command
//! listing the windows Alt+Tab would show, so the user finds a window by
//! typing — its title, its application's name and icon — and switches to
//! it with Enter. The host lists the windows (`pane_extension::windows`)
//! in z-order with the front application's first, so switching back to
//! what the user was in before opening Pane is one keystroke; typing
//! filters by title and application name. The screen is a designed List
//! (pane-app/pane#240), which Pane filters by the text typed in its search
//! field, each item's keywords naming its application. Enter
//! brings the chosen window to the front, restoring it if it is
//! minimized, and a window that closed says so. Window actions (close,
//! minimize, maximize, restore, keep on top) are a later slice.
#![no_std]

use pane_extension::alloc::{
    format,
    string::{String, ToString},
};
use pane_extension::feedback::{HudStyle, show_hud};
use pane_extension::view::{Cx, IntoAnswer, View, empty_state, item, list};
use pane_extension::window::{PopToRootType, close};
use pane_extension::windows::{self, Window};
use pane_extension::{Command, Icon, LaunchRecord};

/// The command's id in `pane.json`.
const SWITCH: &str = "switch-windows";

struct Switch;
pane_extension::export!(Switch);

/// The error's message, as the answer the launcher shows.
fn explain(error: windows::WindowsError) -> String {
    error.message().to_string()
}

/// One listed window: its title, its application's name (also its
/// keyword, which the List's filtering matches besides the title), and the
/// application's icon, with "on another desktop" said of one that is.
/// Enter switches to it.
fn window_item(cx: &mut Cx<Windows>, window: &Window) -> pane_extension::view::Item {
    let id = window.id.clone();
    let icon = window.icon.clone().map(Icon::file);
    let listed = item(window.id.clone())
        .title(window.title.clone())
        .subtitle(subtitle(window))
        .keyword(window.application_name.clone())
        .on_press(cx.listener(move |_: &mut Windows| switch(&id)));
    match icon {
        Some(icon) => listed.icon(icon),
        None => listed,
    }
}

/// The second line of a listed window: its application's name, saying
/// "on another desktop" of a window on one.
fn subtitle(window: &Window) -> String {
    if window.elsewhere {
        format!("{} — on another desktop", window.application_name)
    } else {
        window.application_name.clone()
    }
}

/// Switches to the window `id` names: Pane's window closes first, so the
/// window can take the foreground, then the host brings it to the front,
/// restoring it if it is minimized. A window that closed, or one that did
/// not come to the front, says why in a HUD.
fn switch(id: &str) {
    close(false, PopToRootType::Default);
    if let Err(error) = windows::activate(id) {
        show_hud(error.message(), HudStyle::Failure);
    }
}

/// The command's screen: the windows Alt+Tab would show, in z-order with
/// the front application's first, listed anew each time it is drawn;
/// Enter switches to one. With none to show (nothing else is open) the
/// list says so.
struct Windows;

impl View for Windows {
    fn render(&mut self, cx: &mut Cx<Self>) -> impl IntoAnswer {
        let listed = windows::list_windows().map_err(explain)?;
        let mut view = list()
            .navigation_title("Switch Windows")
            .search_placeholder("Filter the windows…");
        if listed.is_empty() {
            view = view.child(
                empty_state("No other windows are open")
                    .description("The window you were in is the only one"),
            );
        }
        for window in listed.iter() {
            view = view.child(window_item(cx, window));
        }
        Ok::<_, String>(view)
    }
}

impl Command for Switch {
    /// The windows the command lists: a designed view (its `pane.json`
    /// entry says `"mode": "designed"`).
    type DesignedView = Windows;

    /// The windows Alt+Tab would show, listed anew each time the view is
    /// drawn.
    async fn open_designed_view(command: String, _launch: LaunchRecord) -> Result<Windows, String> {
        if command != SWITCH {
            return Err(format!("unknown command: {command}"));
        }
        Ok(Windows)
    }
}
