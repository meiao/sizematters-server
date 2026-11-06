use leptos::prelude::*;
use tw_merge::*;

/// A toggle switch bound to an `RwSignal<bool>`. Renders a `button` with `role="switch"`, so
/// label it with `Label html_for=...` (via `id`) or `aria-labelledby`.
#[component]
pub fn Switch(
    checked: RwSignal<bool>,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] id: String,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "peer inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs outline-none transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input",
        class
    );

    view! {
        <button
            type="button"
            role="switch"
            id=id
            class=merged_class
            aria-checked=move || checked.get().to_string()
            data-state=move || if checked.get() { "checked" } else { "unchecked" }
            on:click=move |_| checked.update(|value| *value = !*value)
        >
            <span
                class="pointer-events-none block size-4 rounded-full bg-background ring-0 transition-transform data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0"
                data-state=move || if checked.get() { "checked" } else { "unchecked" }
            ></span>
        </button>
    }
}
