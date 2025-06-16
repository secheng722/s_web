use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatIdent};

/// 🚀 中间件宏 - 统一处理参数化和非参数化中间件
/// 
/// 这个宏可以处理两种类型的中间件：
/// 1. 带参数的中间件（必须使用宏）
/// 2. 无参数的中间件（可选使用宏，为了一致性推荐使用）
/// 
/// # 使用方式
/// 
/// ## 方式1: 带参数版本（必须使用宏）
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
/// // 使用：
/// app.use_middleware(auth("Bearer secret-token"));
/// ```
/// 
/// ## 方式2: 无参数版本（可选使用宏，推荐用于一致性）
/// ```rust
/// #[middleware]
/// async fn cors(ctx: RequestCtx, next: Next) -> Response {
///     let mut response = next(ctx).await;
///     response.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
///     response
/// }
/// 
/// // 使用：
/// app.use_middleware(cors);
/// ```
/// 
/// ## 不使用宏的版本（也完全可以）
/// ```rust
/// async fn cors(ctx: RequestCtx, next: Next) -> Response {
///     let mut response = next(ctx).await;
///     response.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
///     response
/// }
/// 
/// // 使用：
/// app.use_middleware(cors);
/// ```
/// 
/// ## 转换原理
/// 
/// 带参数的函数会被转换为：
/// ```rust
/// fn auth(token: &'static str) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static {
///     move |ctx, next| {
///         Box::pin(async move {
///             // 原始的函数体
///         })
///     }
/// }
/// ```
/// 
/// 无参数的函数保持不变，直接作为中间件使用：
/// ```rust
/// async fn cors(ctx: RequestCtx, next: Next) -> Response {
///     // 原始的函数体
/// }
/// ```
/// 
/// ## 推荐使用方式
/// 
/// 为了代码的一致性和可维护性，推荐统一使用 `#[middleware]` 宏：
/// - ✅ 一致的代码风格
/// - ✅ 统一的学习成本  
/// - ✅ 未来扩展的兼容性
/// - ✅ 更好的错误提示
/// ```
/// 
/// 无参数的函数保持不变，直接作为中间件使用。
#[proc_macro_attribute]
pub fn middleware(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // 检查函数是否为 async
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
    
    // 解析参数
    let mut param_args = Vec::new();
    let mut param_names = Vec::new();
    let mut has_ctx = false;
    let mut has_next = false;
    
    for arg in &input_fn.sig.inputs {
        match arg {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                    let param_name = ident;
                    
                    // 检查是否为 ctx 或 next 参数
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
    
    // 生成新的函数
    let expanded = if param_args.is_empty() {
        // 无参数版本：直接返回原函数
        quote! {
            #(#fn_attrs)*
            #fn_vis async fn #fn_name(ctx: ree::RequestCtx, next: ree::Next) -> ree::Response #fn_body
        }
    } else {
        // 有参数版本：生成参数化中间件
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

/// 🚀 简化的中间件构造器宏
/// 
/// 这个宏专门用于创建返回中间件闭包的函数，语法更简洁。
/// 
/// # 使用方式
/// 
/// ```rust
/// use ree::middleware_fn;
/// 
/// #[middleware_fn]
/// fn rate_limit(max_requests: usize) -> impl Fn(RequestCtx, Next) -> Response {
///     let counter = Arc::new(AtomicUsize::new(0));
///     move |ctx, next| async move {
///         let current = counter.fetch_add(1, Ordering::SeqCst);
///         if current >= max_requests {
///             return ResponseBuilder::too_many_requests_json(r#"{"error": "Rate limit exceeded"}"#);
///         }
///         next(ctx).await
///     }
/// }
/// 
/// // 使用：
/// app.use_middleware(rate_limit(100));
/// ```
/// 
/// 这个宏会自动处理返回类型的复杂性，让你专注于实现逻辑。
#[proc_macro_attribute]
pub fn middleware_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    let fn_inputs = &input_fn.sig.inputs;
    
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name(#fn_inputs) -> impl Fn(ree::RequestCtx, ree::Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = ree::Response> + Send>> + Send + Sync + 'static {
            #fn_body
        }
    };
    
    TokenStream::from(expanded)
}
