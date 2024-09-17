use cairo_lang_defs::plugin::{
    InlinePluginResult, MacroPluginMetadata, PluginDiagnostic, PluginGeneratedFile, PluginResult,
};
use cairo_lang_diagnostics::Severity;
use cairo_lang_filesystem::{
    db::Edition,
    ids::{CodeMapping, CodeOrigin},
    span::{TextOffset, TextSpan},
};
use cairo_lang_syntax::node::{db::SyntaxGroup, SyntaxNode};

pub fn macro_plugin_metadata(
    metadata: &MacroPluginMetadata<'_>,
) -> proc_macro_server_api::MacroPluginMetadata {
    proc_macro_server_api::MacroPluginMetadata {
        cfg_set: proc_macro_server_api::CfgSet {
            cfgs: metadata
                .cfg_set
                .into_iter()
                .map(|cfg| proc_macro_server_api::Cfg {
                    key: cfg.key.to_string(),
                    value: cfg.value.as_ref().map(|s| s.to_string()),
                })
                .collect(),
        },
        allowed_features: metadata.allowed_features.into_iter().map(ToString::to_string).collect(),
        declared_derives: metadata.declared_derives.into_iter().map(ToString::to_string).collect(),
        edition: match metadata.edition {
            Edition::V2023_01 => proc_macro_server_api::Edition::V202301,
            Edition::V2023_10 => proc_macro_server_api::Edition::V202310,
            Edition::V2023_11 => proc_macro_server_api::Edition::V202311,
            Edition::V2024_07 => proc_macro_server_api::Edition::V202407,
        } as i32,
    }
}

pub fn inline_plugin_result(
    value: proc_macro_server_api::ExpandInlineResponse,
    root: SyntaxNode,
    db: &dyn SyntaxGroup,
) -> InlinePluginResult {
    InlinePluginResult {
        code: value.code.map(plugin_generated_file),
        diagnostics: value
            .diagnostics
            .into_iter()
            .map(|d| plugin_diagnostic(d, root.clone(), db))
            .collect(),
    }
}

pub fn plugin_result(
    value: proc_macro_server_api::ExpandResponse,
    root: SyntaxNode,
    db: &dyn SyntaxGroup,
) -> PluginResult {
    PluginResult {
        code: value.code.map(plugin_generated_file),
        diagnostics: value
            .diagnostics
            .into_iter()
            .map(|d| plugin_diagnostic(d, root.clone(), db))
            .collect(),
        remove_original_item: value.remove_original_item,
    }
}

fn plugin_generated_file(value: proc_macro_server_api::PluginGeneratedFile) -> PluginGeneratedFile {
    PluginGeneratedFile {
        code_mappings: value.code_mappings.into_iter().map(code_mapping).collect(),
        content: value.content,
        name: value.name.into(),
        aux_data: None,
    }
}

fn code_mapping(value: proc_macro_server_api::CodeMapping) -> CodeMapping {
    CodeMapping { span: text_span(value.span), origin: code_origin(value.origin) }
}

fn text_offset(value: u32) -> TextOffset {
    unsafe { std::mem::transmute(value) } //TODO
}

fn text_span(value: proc_macro_server_api::TextSpan) -> TextSpan {
    TextSpan { start: text_offset(value.start), end: text_offset(value.end) }
}

fn code_origin(value: proc_macro_server_api::CodeOrigin) -> CodeOrigin {
    match value.origin.unwrap() {
        proc_macro_server_api::code_origin::Origin::Start(start) => {
            CodeOrigin::Start(text_offset(start))
        }
        proc_macro_server_api::code_origin::Origin::Span(span) => CodeOrigin::Span(text_span(span)),
    }
}

fn plugin_diagnostic(
    value: proc_macro_server_api::PluginDiagnostic,
    root: SyntaxNode,
    db: &dyn SyntaxGroup,
) -> PluginDiagnostic {
    let ptr = value.stable_ptr;

    PluginDiagnostic {
        message: value.message,
        severity: severity(proc_macro_server_api::Severity::try_from(value.severity).unwrap()),
        stable_ptr: root.lookup_offset(db, text_offset(ptr)).stable_ptr(),
    }
}

fn severity(value: proc_macro_server_api::Severity) -> Severity {
    match value {
        proc_macro_server_api::Severity::Error => Severity::Error,
        proc_macro_server_api::Severity::Warning => Severity::Warning,
    }
}
