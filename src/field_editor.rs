// filepath: /workspaces/leptos-ssr-concurrency/field-editor/src/field_editor.rs
use crate::db::{DbManager, Fields};
use leptos::prelude::*;
use leptos::suspense::Suspense;
use leptos::*;
use server_fn::error::ServerFnError;
use wasm_bindgen_futures::spawn_local;

#[server(GetFields)]
pub async fn get_fields() -> Result<Fields, ServerFnError> {
    let mut db = DbManager::new("sqlite:/tmp/fields.db");
    db.initialize()
        .await
        .map_err(|e| ServerFnError::<sqlx::Error>::ServerError(e.to_string()))?;

    let fields = db
        .get_fields()
        .await
        .map_err(|e| ServerFnError::<sqlx::Error>::ServerError(e.to_string()))?;

    Ok(fields)
}

#[server(UpdateFields)]
pub async fn update_fields(
    field1: String,
    field2: String,
    field3: String,
    field4: String,
    expected_version: i64,
) -> Result<bool, ServerFnError> {
    dbg!(format!(
        "server-fn: Updating fields with version: {}",
        expected_version
    ));
    let mut db = DbManager::new("sqlite:/tmp/fields.db");
    db.initialize()
        .await
        .map_err(|e| ServerFnError::<sqlx::Error>::ServerError(e.to_string()))?;

    let success = db
        .update_fields(&field1, &field2, &field3, &field4, expected_version)
        .await
        .map_err(|e| ServerFnError::<sqlx::Error>::ServerError(e.to_string()))?;

    Ok(success)
}

#[component]
pub fn FieldEditor() -> impl IntoView {
    leptos::logging::debug_warn!("FieldEditor component loaded");
    // Set up client state
    let source = RwSignal::new(());
    let fields = Resource::new(
        move || source.get(),
        |_| async move {
            let x = get_fields().await;
            leptos::logging::debug_warn!("Got fields");
            x
        },
    );

    let edit_field1 = RwSignal::new(String::new());
    let edit_field2 = RwSignal::new(String::new());
    let edit_field3 = RwSignal::new(String::new());
    let edit_field4 = RwSignal::new(String::new());
    let version = RwSignal::new(0);
    let show_error = RwSignal::new(false);
    let saving = RwSignal::new(false);

    // Load initial data
    create_effect(move |_| {
        if let Some(Ok(data)) = fields.get() {
            edit_field1.set(data.field1.clone());
            edit_field2.set(data.field2.clone());
            edit_field3.set(data.field3.clone());
            edit_field4.set(data.field4.clone());
            version.set(data.version);
        }
    });

    // Handle save action
    let on_save = move |_| {
        saving.set(true);
        show_error.set(false);

        spawn_local(async move {
            let result = update_fields(
                edit_field1.get(),
                edit_field2.get(),
                edit_field3.get(),
                edit_field4.get(),
                version.get(),
            )
            .await;

            saving.set(false);

            match result {
                Ok(true) => {
                    // Successfully saved
                    // Refresh the data to get the new version
                    source.set(());
                }
                Ok(false) => {
                    // Concurrency conflict - someone else updated the data
                    show_error.set(true);
                    // Refresh the data to get the latest values
                    source.set(());
                }
                Err(_) => {
                    // Error saving
                    show_error.set(true);
                }
            }
        });
    };

    // Define the view
    view! {
        <div class="field-editor">
            <h1>"Field Editor"</h1>

            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    fields.get().map(|fields_result| match fields_result {
                        Err(e) => view! { <div class="error">"Error loading fields: " {e.to_string()}</div> }.into_any(),
                        Ok(data) => view! {
                            <div>
                                <div class="form-group">
                                    <label for="field1">"Field 1"</label>
                                    <input
                                        id="field1"
                                        type="text"
                                        prop:value=edit_field1
                                        on:input=move |ev| {
                                            edit_field1.set(event_target_value(&ev));
                                        }
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="field2">"Field 2"</label>
                                    <input
                                        id="field2"
                                        type="text"
                                        prop:value=edit_field2
                                        on:input=move |ev| {
                                            edit_field2.set(event_target_value(&ev));
                                        }
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="field3">"Field 3"</label>
                                    <input
                                        id="field3"
                                        type="text"
                                        prop:value=edit_field3
                                        on:input=move |ev| {
                                            edit_field3.set(event_target_value(&ev));
                                        }
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="field4">"Field 4"</label>
                                    <input
                                        id="field4"
                                        type="text"
                                        prop:value=edit_field4
                                        on:input=move |ev| {
                                            edit_field4.set(event_target_value(&ev));
                                        }
                                    />
                                </div>

                                <button
                                    on:click=on_save
                                    disabled=saving
                                >
                                    {move || if saving.get() { "Saving..." } else { "Save Changes" }}
                                </button>

                                {move || {
                                    if show_error.get() {
                                        view! {
                                            <div class="error-message">
                                                "Save failed. Another user has updated the fields since you loaded them.
                                                Your changes have been discarded and the fields now show the current values. 
                                                Please try again."
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div class="no-error"></div> }.into_any()
                                    }
                                }}
                            </div>
                        }.into_any()
                    })
                }}
            </Suspense>
        </div>
    }
}
