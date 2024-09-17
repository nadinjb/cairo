use cairo_lang_semantic::plugin::PluginSuite;
use inline_plugin::ProcMacroRemoteInlinePlugin;
use plugin::ProcMacroRemotePlugin;
use proc_macro_server_api::{proc_macros_client::ProcMacrosClient, tonic::transport::Uri};
use std::{future::Future, sync::Arc};
use tokio::{runtime::Handle, task::block_in_place};

pub mod conversions;
pub mod inline_plugin;
pub mod plugin;

/// Runs future to completion in sync world.
fn future_sync<T>(fut: T) -> T::Output
where
    T: Future,
{
    block_in_place(|| Handle::current().block_on(fut))
}

pub fn proc_macros_plugin_suite(uri: Uri) -> PluginSuite {
    let client = future_sync(ProcMacrosClient::connect(uri)).unwrap();

    let mut suite = ProcMacroRemoteInlinePlugin::plugin_suite(client.clone());

    suite.add_plugin_ex(Arc::new(ProcMacroRemotePlugin::new(client)));

    suite
}
