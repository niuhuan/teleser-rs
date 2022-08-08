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

#[proc_macro_error]
#[proc_macro_attribute]
pub fn new_message(_: TokenStream, input: TokenStream) -> TokenStream {
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
    let param_ty_str: String = param_ty.to_string();
    if !(param_ty_str.starts_with("&") && param_ty_str.ends_with("Message")) {
        abort!(
            param.span(),
            format!(
                "unknown param type {}, please modify to &Message",
                param_ty_str
            )
        );
    }
    let trait_name = quote! {::teleser::NewMessageProcess};
    let enum_name = quote! {::teleser::Process::NewMessageProcess};
    // gen token stream
    let ident = &method.sig.ident;
    // let ident_str = format!("{}", ident);
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
        impl Into<::teleser::Process> for #ident {
            fn into(self) -> ::teleser::Process {
                #enum_name(Box::new(self))
            }
        }
    };
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn message_edited(_: TokenStream, input: TokenStream) -> TokenStream {
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
    let param_ty_str: String = param_ty.to_string();
    if !(param_ty_str.starts_with("&") && param_ty_str.ends_with("Message")) {
        abort!(
            param.span(),
            format!(
                "unknown param type {}, please modify to &Message",
                param_ty_str
            )
        );
    }
    let trait_name = quote! {::teleser::MessageEditedProcess};
    let enum_name = quote! {::teleser::Process::MessageEditedProcess};
    // gen token stream
    let ident = &method.sig.ident;
    // let ident_str = format!("{}", ident);
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
            impl Into<::teleser::Process> for #ident {
                fn into(self) -> ::teleser::Process {
    #enum_name(Box::new(self))
                }
            }
        };
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn message_deleted(_: TokenStream, input: TokenStream) -> TokenStream {
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
    let param_ty_str: String = param_ty.to_string();
    if !(param_ty_str.starts_with("&") && param_ty_str.ends_with("MessageDeletion")) {
        abort!(
            param.span(),
            format!(
                "unknown param type {}, please modify to &MessageDeletion",
                param_ty_str
            )
        );
    }
    let trait_name = quote! {::teleser::MessageDeletedProcess};
    let enum_name = quote! {::teleser::Process::MessageDeletedProcess};
    // gen token stream
    let ident = &method.sig.ident;
    // let ident_str = format!("{}", ident);
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
            impl Into<::teleser::Process> for #ident {
                fn into(self) -> ::teleser::Process {
    #enum_name(Box::new(self))
                }
            }
        };
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn callback_query(_: TokenStream, input: TokenStream) -> TokenStream {
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
    let param_ty_str: String = param_ty.to_string();
    if !(param_ty_str.starts_with("&") && param_ty_str.ends_with("CallbackQuery")) {
        abort!(
            param.span(),
            format!(
                "unknown param type {}, please modify to &CallbackQuery",
                param_ty_str
            )
        );
    }
    let trait_name = quote! {::teleser::CallbackQueryProcess};
    let enum_name = quote! {::teleser::Process::CallbackQueryProcess};
    // gen token stream
    let ident = &method.sig.ident;
    // let ident_str = format!("{}", ident);
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
            impl Into<::teleser::Process> for #ident {
                fn into(self) -> ::teleser::Process {
    #enum_name(Box::new(self))
                }
            }
        };
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn inline_query(_: TokenStream, input: TokenStream) -> TokenStream {
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
    let param_ty_str: String = param_ty.to_string();
    if !(param_ty_str.starts_with("&") && param_ty_str.ends_with("InlineQuery")) {
        abort!(
            param.span(),
            format!(
                "unknown param type {}, please modify to &InlineQuery",
                param_ty_str
            )
        );
    }
    let trait_name = quote! {::teleser::InlineQueryProcess};
    let enum_name = quote! {::teleser::Process::InlineQueryProcess};
    // gen token stream
    let ident = &method.sig.ident;
    // let ident_str = format!("{}", ident);
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
            impl Into<::teleser::Process> for #ident {
                fn into(self) -> ::teleser::Process {
    #enum_name(Box::new(self))
                }
            }
        };
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn raw(_: TokenStream, input: TokenStream) -> TokenStream {
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
    let param_ty_str: String = param_ty.to_string();
    if !(param_ty_str.starts_with("&") && param_ty_str.ends_with("Update")) {
        abort!(
            param.span(),
            format!(
                "unknown param type {}, please modify to &tl::enums::Update",
                param_ty_str
            )
        );
    }
    let trait_name = quote! {::teleser::RawProcess};
    let enum_name = quote! {::teleser::Process::RawProcess};
    // gen token stream
    let ident = &method.sig.ident;
    // let ident_str = format!("{}", ident);
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
            impl Into<::teleser::Process> for #ident {
                fn into(self) -> ::teleser::Process {
    #enum_name(Box::new(self))
                }
            }
        };
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}
