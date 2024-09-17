use super::conversions::{inline_plugin_result, macro_plugin_metadata};
use super::future_sync;
use cairo_lang_defs::plugin::InlineMacroExprPlugin;
use cairo_lang_semantic::plugin::PluginSuite;
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use proc_macro_server_api::{
    proc_macros_client::ProcMacrosClient, tonic, Empty, ExpandInlineParams,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct ProcMacroRemoteInlinePlugin {
    client: ProcMacrosClient<tonic::transport::Channel>,
}

impl ProcMacroRemoteInlinePlugin {
    pub fn plugin_suite(mut client: ProcMacrosClient<tonic::transport::Channel>) -> PluginSuite {
        let mut suite = PluginSuite::default();

        let names =
            future_sync(client.defined_inline_macros(Empty {})).unwrap().into_inner().macros;

        let this = Arc::new(Self { client });

        // Register this plugin to handle the expansion of each macro supported by proc-macro-server.
        for name in names {
            suite.add_inline_macro_plugin_ex(&name, this.clone());
        }

        suite
    }
}

impl InlineMacroExprPlugin for ProcMacroRemoteInlinePlugin {
    fn generate_code(
        &self,
        db: &dyn cairo_lang_syntax::node::db::SyntaxGroup,
        item_ast: &cairo_lang_syntax::node::ast::ExprInlineMacro,
        metadata: &cairo_lang_defs::plugin::MacroPluginMetadata<'_>,
    ) -> cairo_lang_defs::plugin::InlinePluginResult {
        let item = item_ast.as_syntax_node().get_text(db);
        let mut client = self.client.clone();

        let response = future_sync(client.expand_inline(ExpandInlineParams {
            code: item,
            metadata: macro_plugin_metadata(metadata),
        }))
        .unwrap();

        let response = response.into_inner();

        let root = std::iter::successors(Some(item_ast.as_syntax_node()), SyntaxNode::parent)
            .last()
            .unwrap();

        inline_plugin_result(response, root, db)
    }
}
