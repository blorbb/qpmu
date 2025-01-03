use qpmu::{config::Config, plugin::Plugin};

use super::plugin_list;
use crate::model::LauncherMsg;

#[derive(Debug)]
pub enum SettingsMsg {
    SetPluginList(Vec<Plugin>),
    SetSelection(Option<usize>),
    Show,
}

#[derive(Debug)]
pub enum SettingsOutput {
    ReloadPlugins(Config),
}

pub fn output_transform(input: SettingsOutput) -> LauncherMsg {
    match input {
        SettingsOutput::ReloadPlugins(c) => LauncherMsg::ReloadPlugins(c),
    }
}

impl From<plugin_list::Output> for SettingsMsg {
    fn from(value: plugin_list::Output) -> Self {
        match value {
            plugin_list::Output::SetPluginList(vec) => Self::SetPluginList(vec),
            plugin_list::Output::SetSelection(index) => Self::SetSelection(Some(index)),
        }
    }
}
