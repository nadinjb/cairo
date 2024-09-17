use super::conversions::{macro_plugin_metadata, plugin_result};
use super::future_sync;
use cairo_lang_defs::plugin::MacroPlugin;
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use proc_macro_server_api::{proc_macros_client::ProcMacrosClient, tonic, Empty, ExpandParams};

#[derive(Debug)]
pub struct ProcMacroRemotePlugin {
    client: ProcMacrosClient<tonic::transport::Channel>,
}

impl ProcMacroRemotePlugin {
    pub fn new(client: ProcMacrosClient<tonic::transport::Channel>) -> Self {
        Self { client }
    }
}

impl MacroPlugin for ProcMacroRemotePlugin {
    fn generate_code(
        &self,
        db: &dyn cairo_lang_syntax::node::db::SyntaxGroup,
        item_ast: cairo_lang_syntax::node::ast::ModuleItem,
        metadata: &cairo_lang_defs::plugin::MacroPluginMetadata<'_>,
    ) -> cairo_lang_defs::plugin::PluginResult {
        let item = item_ast.as_syntax_node().get_text(db);
        let mut client = self.client.clone();

        let response = future_sync(
            client.expand(ExpandParams { code: item, metadata: macro_plugin_metadata(metadata) }),
        )
        .unwrap();

        let response = response.into_inner();

        let root = std::iter::successors(Some(item_ast.as_syntax_node()), SyntaxNode::parent)
            .last()
            .unwrap();

        plugin_result(response, root, db)
    }

    fn declared_attributes(&self) -> Vec<String> {
        future_sync(self.client.clone().defined_attributes(Empty {}))
            .unwrap()
            .into_inner()
            .attributes
    }
}
