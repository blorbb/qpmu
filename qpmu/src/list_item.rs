use core::fmt;

use color_eyre::eyre::Result;

use super::Plugin;
use crate::{
    hotkey::Hotkey,
    plugin::{proto, Action},
    Input,
};

/// A row in the results list.
#[derive(Clone)]
pub struct ListItem {
    plugin: Plugin,
    item: proto::ListItem,
}

impl ListItem {
    pub(crate) fn new(plugin: Plugin, item: proto::ListItem) -> Self {
        Self { plugin, item }
    }

    pub fn plugin(&self) -> &Plugin {
        &self.plugin
    }

    pub fn title(&self) -> &str {
        &self.item.title
    }

    pub fn description(&self) -> &str {
        &self.item.description
    }

    pub fn icon(&self) -> Option<Icon> {
        self.item.icon.clone().map(Icon::from_proto)
    }

    pub async fn activate(self) -> Result<Vec<Action>> {
        self.plugin.clone().activate(self.item.id).await
    }

    pub async fn alt_activate(self) -> Result<Vec<Action>> {
        self.plugin.clone().alt_activate(self.item.id).await
    }

    pub async fn hotkey_activate(self, hotkey: Hotkey) -> Result<Vec<Action>> {
        self.plugin
            .clone()
            .hotkey_activate(self.item.id, proto::Hotkey::from(hotkey))
            .await
    }

    pub async fn complete(self) -> Result<Option<Input>> {
        self.plugin.clone().complete(self.item.id).await
    }
}

impl fmt::Debug for ListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListItem")
            .field("plugin", &self.plugin())
            .field("title", &self.item.title)
            .field("description", &self.item.description)
            .field("icon", &self.item.icon)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum Icon {
    Name(String),
    Text(String),
}

impl Icon {
    pub(crate) fn from_proto(proto: proto::list_item::Icon) -> Self {
        use proto::list_item::Icon as Proto;
        match proto {
            Proto::Name(name) => Self::Name(name),
            Proto::Text(text) => Self::Text(text),
        }
    }
}
