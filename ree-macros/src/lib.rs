use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatIdent};

/// 🚀 Middleware macros -Unified processing of parameterized and non-parametric middleware
/// 
/// This macro can handle two types of middleware:
/// 1. Middleware with parameters (must use macros)
/// 2. Middleware without parameters (optional use of macros, recommended for consistency)
/// 
/// # How to use
/// 
/// ## Method 1: Parameter version (must use macros)
/// ```rust
/// use ree::middleware;
/// 
/// #[middleware]
/// async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
///     if let Some(auth) = ctx.request.headers().get("Authorization") {
///         if auth.to_str().unwrap_or("") == token {
///             return next(ctx).await;
///         }
///     }
///     ResponseBuilder::unauthorized_json(r#"{"error": "Unauthorized"}"#)
/// }
/// 
/// // use:
/// app.use_middleware(auth("Bearer secret-token"));
/// ```
/// 
/// ## Method 2: No parameter version (optional use of macros, recommended for consistency)
/// ```rust
/// #[middleware]
/// async fn cors(ctx: RequestCtx, next: Next) -> Response {
///     let mut response = next(ctx).await;
///     response.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
///     Response
/// }
/// 
/// // use:
/// app.use_middleware(cors);
/// ```
/// 
/// ## Version without using macros (it's totally OK)
/// ```rust
/// async fn cors(ctx: RequestCtx, next: Next) -> Response {
///     let mut response = next(ctx).await;
///     response.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
///     Response
/// }
/// 
/// // use:
/// app.use_middleware(cors);
/// ```
/// 
/// ## Conversion principle
/// 
/// Functions with parameters will be converted to:
/// ```rust
/// fn auth(token: &'static str) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static {
///     move |ctx, next| {
///         Box::pin(async move {
///             // The original function body
///         })
///     }
/// }
/// ```
/// 
/// Functions without parameters remain unchanged and are used directly as middleware:
/// ```rust
/// async fn cors(ctx: RequestCtx, next: Next) -> Response {
///     // The original function body
/// }
/// ```
/// 
/// ## Recommended usage
/// 
/// For code consistency and maintainability, it is recommended to use the `#[middleware]` macro in a unified way:
/// -✅ Consistent code style
/// -✅ Unified learning costs  
/// -✅ Compatibility for future expansions
/// -✅ Better error prompts
/// ```
/// 
/// Functions without parameters remain unchanged and are used directly as middleware.
#[proc_macro_attribute]
pub fn middleware(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // Check if the function is async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input_fn.sig.fn_token,
            "middleware macro can only be applied to async functions"
        ).to_compile_error().into();
    }
    
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    
    let mut param_args = Vec::new();
    let mut param_names = Vec::new();
    let mut has_ctx = false;
    let mut has_next = false;
    
    for arg in &input_fn.sig.inputs {
        match arg {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                    let param_name = ident;
                    
                    // Check if it is a ctx or next parameter
                    if param_name == "ctx" {
                        has_ctx = true;
                    } else if param_name == "next" {
                        has_next = true;
                    } else {
                        // 普通参数
                        param_args.push(pat_type.clone());
                        param_names.push(param_name.clone());
                    }
                }
            }
            _ => {
                return syn::Error::new_spanned(
                    arg,
                    "middleware functions should only have typed parameters"
                ).to_compile_error().into();
            }
        }
    }
    
    if !has_ctx || !has_next {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "middleware function must have 'ctx: RequestCtx' and 'next: Next' parameters"
        ).to_compile_error().into();
    }
    
    // Generate new functions
    let expanded = if param_args.is_empty() {
        // No parameter version: return directly to the original function
        quote! {
            #(#fn_attrs)*
            #fn_vis async fn #fn_name(ctx: ree::RequestCtx, next: ree::Next) -> ree::Response #fn_body
        }
    } else {
        // Parameter version: Generate parameterized middleware
        quote! {
            #(#fn_attrs)*
            #fn_vis fn #fn_name(#(#param_args),*) -> impl Fn(ree::RequestCtx, ree::Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = ree::Response> + Send>> + Send + Sync + 'static {
                move |ctx, next| {
                    #(let #param_names = #param_names.clone();)*
                    Box::pin(async move #fn_body)
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}
