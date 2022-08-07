use proc_macro::TokenStream;

use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg};

/// debug = note expanded codes if env PROC_QQ_CODEGEN_DEBUG exists
macro_rules! emit {
    ($tokens:expr) => {{
        use devise::ext::SpanDiagnosticExt;
        let mut tokens = $tokens;
        if std::env::var_os("TELESER_DEBUG").is_some() {
            let debug_tokens = proc_macro2::Span::call_site()
                .note("emitting teleser debug output")
                .note(tokens.to_string())
                .emit_as_item_tokens();

            tokens.extend(debug_tokens);
        }
        tokens.into()
    }};
}

/// event proc
#[proc_macro_error]
#[proc_macro_attribute]
pub fn event(_: TokenStream, input: TokenStream) -> TokenStream {
    // must append to async fn
    let method = parse_macro_input!(input as syn::ItemFn);
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "must be async function");
    }
    // params check
    let params = &method.sig.inputs;
    if params.len() != 2 {
        abort!(&method.sig.span(), "must be 2 params");
    };
    let pm = params.first().unwrap();
    let fp = match pm {
        FnArg::Receiver(_) => abort!(&pm.span(), "do not input self"),
        FnArg::Typed(pt) => pt,
    };
    let cpa = fp.pat.as_ref();
    let cty = fp.ty.as_ref();
    let param = params.last().unwrap();
    let param = match param {
        FnArg::Receiver(_) => abort!(&param.span(), "do not input self"),
        FnArg::Typed(pt) => pt,
    };
    let param_pat = param.pat.as_ref();
    let param_ty = param.ty.as_ref();
    let param_ty = quote! {#param_ty};
    let tokens = match param_ty.to_string().as_str() {
        "& Message" => (
            quote! {::teleser::NewMessageProcess},
            quote! {::teleser::Process::NewMessageProcess},
        ),
        t => abort!(param.span(), format!("unknow param type {}", t),),
    };
    let trait_name = tokens.0;
    let enum_name = tokens.1;
    // gen token stream
    let ident = &method.sig.ident;
    let ident_str = format!("{}", ident);
    let build_struct = quote! {
        #[allow(non_camel_case_types)]
        pub struct #ident {}
    };
    let block = &method.block;
    let build_trait = quote! {
        #[::teleser::re_exports::async_trait::async_trait]
        impl #trait_name for #ident {
            async fn handle(&self, #cpa: #cty,#param_pat: #param_ty) -> Result<bool> #block
        }
    };
    let build_into = quote! {
        impl Into<::teleser::Handler> for #ident {
            fn into(self) -> ::teleser::Handler {
                ::teleser::Handler{
                    id: #ident_str.into(),
                    process: #enum_name(Box::new(self)),
                }
            }
        }
    };
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}
