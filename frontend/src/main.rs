mod terminal;

use shared_frontend::components::{
    app_shell::AppShell,
    footer::FooterProps,
    header::HeaderProps,
};
use terminal::TerminalComponent;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let header_props = HeaderProps {
        site_title: "Network Diagnostics".into(),
        theme: "dark".into(),
        language: shared_frontend::i18n::Language::from_code("en"),
        toggle_theme: Callback::noop(),
        on_language_change: Callback::noop(),
        is_authenticated: false,
        pin_required: false,
        on_logout: Callback::noop(),
        logout_tooltip: "".into(),
        theme_toggle_tooltip: "".into(),
        print_tooltip: "".into(),
        on_print: None,
        enable_translation: false,
        enable_themes: false,
        enable_print: false,
        print_disabled: true,
        site_url: Some("/".into()),
        repo: Some("studio2201/diag".into()),
        version: Some("0.1.0".into()),
        version_url: None,
    };

    let footer_props = FooterProps {
        show_version: true,
        version: "0.1.0".into(),
        show_github: true,
        github_url: Some("https://github.com/studio2201/diag".into()),
        version_url: None,
        repo: Some("studio2201/diag".into()),
        show_coffee: false,
        coffee_url: None,
        children: html! {},
    };

    html! {
        <AppShell header={header_props} footer={footer_props}>
            <TerminalComponent />
        </AppShell>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
