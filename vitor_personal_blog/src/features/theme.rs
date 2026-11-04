//! The light and dark color schemes, with the visitor's choice kept in a cookie.
//!
//! The Topcoat UI theme switches its tokens when `<html>` has the `dark` class. The root
//! layout renders that class from the cookie, so a returning visitor gets their scheme
//! in the first paint. Until a visitor picks one, the page follows the system setting.

use topcoat::{
    Result,
    context::Cx,
    cookie::{Cookie, Cookies, SameSite, cookies, time::Duration},
    icon::{icon, iconify::iconify_icon},
    runtime::{Event, procedure},
    view::{Unescaped, View, attributes, component, view},
};

use crate::components::button::{ButtonSize, ButtonVariant, button};

/// The cookie holding the visitor's choice, `light` or `dark`.
const COOKIE: &str = "theme";

/// Adds the `dark` class before the first paint when the system prefers a dark scheme.
const FOLLOW_SYSTEM: &str = "if (matchMedia('(prefers-color-scheme: dark)').matches) \
                             document.documentElement.classList.add('dark')";

/// A color scheme the visitor picked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// Returns the visitor's choice, or `None` if the page should follow the system.
    pub fn from_cookie(cx: &Cx) -> Option<Theme> {
        match cookies(cx).get(COOKIE)?.value() {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}

/// Remembers the visitor's color scheme for later page loads.
#[procedure]
pub async fn set_theme(cx: &Cx, dark: bool) -> Result<()> {
    let value = if dark { "dark" } else { "light" };
    cookies(cx).add(
        Cookie::build((COOKIE, value))
            .path("/")
            .same_site(SameSite::Lax)
            .max_age(Duration::days(365))
            .build(),
    );
    Ok(())
}

/// A `<head>` script that applies the system color scheme when the visitor has not
/// picked one. It renders nothing once they have.
#[component]
pub async fn theme_script(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        if Theme::from_cookie(cx).is_none() {
            <script>(Unescaped::new_unchecked(FOLLOW_SYSTEM))</script>
        }
    })
}

/// A button that switches between the light and dark schemes and remembers the choice.
#[component]
pub async fn theme_toggle() -> Result<impl View> {
    Ok(view! {
        button(
            variant: ButtonVariant::Outline,
            size: ButtonSize::Icon,
            attrs: attributes! {
                type="button"
                aria-label="Toggle dark mode"
                @click=$(async |_e: Event| {
                    // The class changes the page right away. The cookie makes the server
                    // render the same scheme on the next load.
                    let dark = raw!(
                        "document.documentElement.classList.toggle('dark')",
                        false,
                    );
                    set_theme(dark).await;
                })
            },
            // Each icon shows the scheme the button switches to.
            icon(
                data: iconify_icon!("lucide:moon"),
                attrs: attributes! { class="size-4 dark:hidden" }
            )
            icon(
                data: iconify_icon!("lucide:sun"),
                attrs: attributes! { class="hidden size-4 dark:block" }
            )
        )
    })
}
