use az::SaturatingAs;
use gtk::prelude::{EditableExt, EntryExt, GtkWindowExt};
use qpmu::Input;
use reactive_graph::{
    effect::Effect,
    signal::WriteSignal,
    traits::{Get, Set},
    wrappers::read::Signal,
};
use tap::Tap;

use crate::utils::{
    stores::{EventHandler, WidgetRef},
    widget_ext::WidgetSetRef as _,
};

#[tracing::instrument]
#[bon::builder]
pub fn entry(
    #[builder(into)] input: Signal<Input>,
    #[builder(into)] set_input: WriteSignal<Input>,
    #[builder(default)] entry_ref: WidgetRef<gtk::Entry>,
    settings_window: gtk::Window,
) -> gtk::Entry {
    let change_handler = EventHandler::<gtk::Entry>::new();

    Effect::new(move || {
        let Input {
            contents,
            selection,
        } = input.get();
        let selection = (i32::from(selection.0), i32::from(selection.1));
        eprintln!("{contents} at {selection:?}");
        change_handler.suppress(|e| {
            e.set_text(&contents);
            e.select_region(selection.0, selection.1);
        });
    });

    gtk::Entry::builder()
        .placeholder_text("Search...")
        .css_classes(["main-entry"])
        .primary_icon_name("search")
        .secondary_icon_name("settings")
        .secondary_icon_activatable(true)
        // must guarantee that there are no new lines
        .truncate_multiline(true)
        .widget_ref(entry_ref)
        .tap(|entry| {
            change_handler.set(
                entry,
                entry.connect_changed(move |entry| {
                    set_input.set(Input {
                        contents: entry.text().replace('\n', ""),
                        selection: {
                            eprintln!("HERE");
                            let (a, b) = entry.selection_bounds().unwrap_or_else(|| {
                                let pos = entry.position();
                                (pos, pos)
                            });
                            (a.saturating_as(), b.saturating_as())
                        },
                    });
                }),
            );
            entry.connect_icon_press(move |_, icon_position| {
                if icon_position == gtk::EntryIconPosition::Secondary {
                    settings_window.present();
                }
            });
        })
}
