use relm4::{
    gtk::{
        self,
        gdk::Key,
        prelude::{
            EditableExt as _, EntryExt as _, GtkWindowExt as _, ListBoxRowExt, WidgetExt as _,
        },
        EventControllerKey, ListBox,
    },
    Component, ComponentParts, ComponentSender, RelmContainerExt as _, RelmRemoveAllExt,
};

use crate::{
    model::{Launcher, LauncherCmd, LauncherMsg},
    plugins::{self, PluginActivationAction, PluginEvent},
    styles::load_css,
};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

#[derive(Debug)]
pub struct LauncherWidgets {
    entry: gtk::Entry,
    scroller: gtk::ScrolledWindow,
    results_list: gtk::ListBox,
}

// not using the macro because the app has a lot of custom behaviour
// and the list of items is not static.
impl Component for Launcher {
    type Input = LauncherMsg;
    type Output = ();
    type Init = ();
    type Widgets = LauncherWidgets;
    type Root = gtk::Window;
    type CommandOutput = LauncherCmd;

    fn init_root() -> Self::Root {
        gtk::Window::default()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        load_css();
        let model = Launcher::new();
        let window = root.clone();
        window.set_title(Some("qpmu"));
        window.set_default_width(WIDTH);
        window.set_default_height(-1);
        window.set_hide_on_close(true);
        window.set_decorated(false);
        window.set_vexpand(true);
        window.set_css_classes(&["window"]);
        {
            let sender = sender.clone();
            window.connect_visible_notify(move |window| {
                if window.is_visible() {
                    sender.spawn_oneshot_command(|| {
                        // needs a short delay before focusing, otherwise
                        // it doesn't focus properly
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        LauncherCmd::Focus
                    })
                }
            });
        }

        // main box layout
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();

        // main input line
        let entry = gtk::Entry::builder()
            .placeholder_text("Search...")
            .css_classes(["main-entry"])
            .build();
        {
            let sender = sender.clone();
            entry.connect_changed(move |entry| {
                sender.input(LauncherMsg::Query(entry.text().to_string()));
            });
        }

        // results list
        let list_scroller = gtk::ScrolledWindow::builder()
            .min_content_height(0)
            .max_content_height(HEIGHT_MAX)
            .propagate_natural_height(true)
            .css_classes(["main-scroller"])
            .build();
        list_scroller.set_visible(!model.results.is_empty());

        let list = ListBox::builder()
            .selection_mode(gtk::SelectionMode::Browse)
            .css_classes(["main-list"])
            .build();
        list.select_row(list.row_at_index(model.selection as i32).as_ref());
        {
            let sender = sender.clone();
            list.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    sender.input(LauncherMsg::Select(row.index() as usize));
                }
            });
        }

        window.container_add(&vbox);
        window.add_controller(model.key_controller(sender));
        vbox.container_add(&entry);
        vbox.container_add(&list_scroller);
        list_scroller.container_add(&list);

        let widgets = Self::Widgets {
            entry,
            scroller: list_scroller,
            results_list: list,
        };
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        self.reset();
        // should be here to ensure it is always false when update_view is run.
        self.grab_full_focus = false;
        root.set_default_height(-1);

        match message {
            LauncherMsg::Query(query) => {
                self.set_query(query.clone());
                let i = self.next_action();

                sender.oneshot_command(async move {
                    LauncherCmd::PluginEvent(
                        i,
                        plugins::process_ui_event(plugins::UiEvent::InputChanged { query })
                            .await
                            .unwrap(),
                    )
                });
            }
            LauncherMsg::SetList(list) => {
                self.set_results(list);
                // always mark as changed
                self.update_selection(|x| *x = 0);
            }
            LauncherMsg::Select(index) => {
                self.set_selection(index);
            }
            LauncherMsg::SelectDelta(delta) => {
                self.update_selection(|sel| *sel = sel.saturating_add_signed(delta));
            }
            LauncherMsg::Activate => {
                if let Some(plugin) = self.results.get(self.selection).cloned() {
                    let i = self.next_action();
                    sender.oneshot_command(async move {
                        LauncherCmd::PluginEvent(
                            i,
                            plugins::process_ui_event(plugins::UiEvent::Activate { item: plugin })
                                .await
                                .unwrap(),
                        )
                    });
                }
            }
            LauncherMsg::Close => {
                root.close();
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            LauncherCmd::PluginEvent(index, e) => match e {
                PluginEvent::SetList(vec) => {
                    if self.should_perform(index) {
                        sender.input(LauncherMsg::SetList(vec))
                    }
                }
                PluginEvent::Activate(vec) => {
                    use std::process::{Command, Stdio};
                    // TODO: remove unwraps
                    for ev in vec {
                        match ev {
                            PluginActivationAction::Close => sender.input(LauncherMsg::Close),
                            PluginActivationAction::RunCommand((cmd, args)) => {
                                Command::new(cmd)
                                    .args(args)
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .unwrap();
                            }
                            PluginActivationAction::RunShell(str) => {
                                Command::new("sh")
                                    .arg("-c")
                                    .arg(str)
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .unwrap();
                            }
                            PluginActivationAction::Copy(string) => {
                                arboard::Clipboard::new().unwrap().set_text(string).unwrap();
                            }
                        }
                    }
                }
            },
            LauncherCmd::Focus => {
                self.grab_full_focus = true;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let Self::Widgets {
            entry,
            scroller,
            results_list,
        } = widgets;

        if self.grab_full_focus {
            entry.grab_focus();
        } else {
            entry.grab_focus_without_selecting();
        }

        // if *self.get_query() != entry.text() {
        //     entry.set_text(&self.get_query());
        // }

        scroller.set_visible(!self.results.is_empty());

        if self.changed_results() {
            results_list.remove_all();
            // recreate list of results
            for item in &self.results {
                // item format:
                // icon | title
                //      | description

                let hbox = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(16)
                    .build();
                if let Some(icon_name) = &item.icon {
                    let icon = gtk::Image::from_icon_name(&icon_name);
                    icon.set_size_request(32, 32);
                    icon.set_icon_size(gtk::IconSize::Large);
                    hbox.container_add(&icon);
                }

                let vbox = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .spacing(4)
                    .build();
                let title = gtk::Label::builder()
                    .label(&item.title)
                    .halign(gtk::Align::Start)
                    .css_classes(["list-item-title"])
                    .build();
                vbox.container_add(&title);

                if !item.description.is_empty() {
                    let description = gtk::Label::builder()
                        .label(&item.description)
                        .halign(gtk::Align::Start)
                        .css_classes(["list-item-description"])
                        .build();
                    vbox.container_add(&description);
                }
                hbox.container_add(&vbox);

                results_list.container_add(
                    &gtk::ListBoxRow::builder()
                        .css_classes(["list-item"])
                        .child(&hbox)
                        .build(),
                );
            }
        }

        if self.changed_selection() {
            results_list.select_row(
                results_list
                    .row_at_index(*self.get_selection() as i32)
                    .as_ref(),
            );
        }
    }
}

impl Launcher {
    fn key_controller(&self, sender: ComponentSender<Self>) -> EventControllerKey {
        let key_events = EventControllerKey::builder()
            .propagation_phase(gtk::PropagationPhase::Capture)
            .build();

        key_events.connect_key_pressed(move |_self, key, _keycode, _modifiers| {
            match key {
                Key::Escape => sender.input(LauncherMsg::Close),
                Key::Down => sender.input(LauncherMsg::SelectDelta(1)),
                Key::Up => sender.input(LauncherMsg::SelectDelta(-1)),
                Key::Return => sender.input(LauncherMsg::Activate),
                _ => return gtk::glib::Propagation::Proceed,
            }
            gtk::glib::Propagation::Stop
        });

        key_events
    }
}
