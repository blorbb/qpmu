pub use comette_tauri_types::{Event, ListItem, ListStyle};
use tauri::{ipc::Channel, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

use crate::{state::AppState, window};

#[derive(Clone)]
pub struct EventChannel {
    pub channel: Channel<Event>,
    pub app: tauri::AppHandle,
}

impl comette::Frontend for EventChannel {
    fn close(&mut self) {
        window::hide_menu(&self.app);
    }

    fn copy(&mut self, str: String) {
        self.app.clipboard().write_text(str).unwrap();
    }

    fn set_input(&mut self, input: comette::Input) {
        self.channel
            .send(Event::SetInput {
                contents: input.contents,
                selection: input.selection,
            })
            .unwrap();
    }

    fn set_list(&mut self, list: comette::List) {
        let state = self.app.state::<AppState>();
        self.channel
            .send(Event::SetList {
                items: state.register_list_items(list.items.into_iter()),
                style: list.style.map(list_style_from_comette),
            })
            .unwrap();
    }

    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report) {
        self.app
            .notification()
            .builder()
            .title(title)
            .body(format!("{error:#}"))
            .show()
            .unwrap();
    }
}

fn list_style_from_comette(value: comette::ListStyle) -> ListStyle {
    match value {
        comette::ListStyle::Rows => ListStyle::Rows,
        comette::ListStyle::Grid => ListStyle::Grid,
        comette::ListStyle::GridWithColumns(columns) => ListStyle::GridWithColumns { columns },
    }
}
