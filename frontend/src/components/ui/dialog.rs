// Adapted from the rust-ui `dialog` component (https://rust-ui.com, MIT).
// Upstream opens/closes through an inline script bound to a trigger button;
// this version keeps its markup and classes but is driven by an `RwSignal<bool>`,
// since the app opens dialogs from things that aren't buttons (avatar, name).

use icons::X;
use leptos::html;
use leptos::prelude::*;
use leptos_ui::clx;
use tw_merge::*;
use wasm_bindgen::JsCast;

mod components {
    use super::*;
    clx! {DialogBody, div, "flex flex-col gap-4"}
    clx! {DialogHeader, div, "flex flex-col gap-2 text-center sm:text-left"}
    clx! {DialogTitle, h3, "text-lg leading-none font-semibold"}
    clx! {DialogDescription, div, "text-muted-foreground text-sm flex flex-col gap-2"}
    clx! {DialogFooter, footer, "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end"}
}

pub use components::*;

#[component]
pub fn Dialog(
    open: RwSignal<bool>,
    children: ChildrenFn,
    #[prop(optional, into)] class: String,
    /// Id of the element that labels the dialog (usually its `DialogTitle`).
    #[prop(optional, into)]
    labelledby: Option<String>,
) -> impl IntoView {
    let merged_class = tw_merge!(
        "flex flex-col gap-4 bg-background border rounded-2xl shadow-lg p-6 w-full max-w-[calc(100%-2rem)] sm:max-w-md max-h-[85vh] overflow-y-auto fixed top-[50%] left-[50%] translate-x-[-50%] translate-y-[-50%] z-100",
        class
    );
    let content_ref = NodeRef::<html::Div>::new();

    // Move focus into the dialog when it opens, and hand it back on close.
    let previous_focus = StoredValue::new_local(None::<web_sys::HtmlElement>);
    Effect::new(move |_| {
        if open.get() {
            previous_focus.set_value(
                document().active_element().and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok()),
            );
        } else if let Some(el) = previous_focus.try_update_value(Option::take).flatten() {
            let _ = el.focus();
        }
    });
    Effect::new(move |_| {
        if let Some(content) = content_ref.get() {
            let target = content
                .query_selector("input, button:not([data-dialog-close])")
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
            if let Some(target) = target {
                let _ = target.focus();
            }
        }
    });

    let children = StoredValue::new(children);

    view! {
        <Show when=move || open.get()>
            <div
                data-name="DialogBackdrop"
                class="fixed inset-0 z-60 bg-black/50"
                on:click=move |_| open.set(false)
            />
            <div
                data-name="DialogContent"
                role="dialog"
                aria-modal="true"
                aria-labelledby=labelledby.clone()
                class=merged_class.clone()
                node_ref=content_ref
                on:keydown=move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Escape" {
                        e.prevent_default();
                        open.set(false);
                    }
                }
            >
                <button
                    type="button"
                    class="absolute top-4 right-4 p-1 rounded-sm opacity-70 hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-none focus:ring-ring [&_svg:not([class*='size-'])]:size-4"
                    data-dialog-close=""
                    aria-label="Close dialog"
                    on:click=move |_| open.set(false)
                >
                    <X />
                </button>
                {children.read_value()()}
            </div>
        </Show>
    }
}
