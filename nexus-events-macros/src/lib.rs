extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, Fields, ItemFn, Type};

/// Marks a struct so users can put `#[event_component]` above it.
/// For this simplified broadcast approach, we do nothing except
/// confirm we can place it on a named or unit struct.
#[proc_macro_attribute]
pub fn event_component(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as ItemStruct);
    match &mut ast.fields {
        Fields::Named(_) | Fields::Unit => { /* Allowed */ }
        Fields::Unnamed(_) => {
            return syn::Error::new_spanned(
                &ast.fields,
                "Cannot use `#[event_component]` on a tuple struct"
            )
            .to_compile_error()
            .into();
        }
    }
    // Just return the struct as-is
    TokenStream::from(quote! { #ast })
}

/// Marks a method as an event handler. On the first call,
/// it uses a local `static ONCE: Once` to do the subscription.
#[proc_macro_attribute]
pub fn event_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let event_ty = parse_macro_input!(attr as Type);
    let method = parse_macro_input!(item as ItemFn);

    let _fn_name = &method.sig.ident;
    let fn_vis = &method.vis;
    let fn_attrs = &method.attrs;
    let fn_block = &method.block;
    let fn_sig = &method.sig;

    // We define a local static Once inside the user’s method body,
    // so there's no associated static or nested module.
    // The subscription is effectively a "type-level" broadcast approach (no per-instance).
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // do the subscription once
            {
                use std::sync::Once;
                static INIT: Once = Once::new();
                INIT.call_once(|| {
                    use ::nexus_events::core::{subscribe, Event};
                    // Subscribe a broadcast closure. Right now, it's effectively no-op or a global approach.
                    subscribe::<#event_ty, _>(move |_evt: &#event_ty| {
                        // No instance-based logic – you might store a global list if needed
                    });
                });
            }
            // Now run the user’s actual method body
            #fn_block
        }
    };
    TokenStream::from(expanded)
}

/// Marks a method as an event sender. It builds an event
/// from the method parameters, dispatches it, and returns the user's result.
#[proc_macro_attribute]
pub fn event_sender(attr: TokenStream, item: TokenStream) -> TokenStream {
    let event_ty = parse_macro_input!(attr as Type);
    let input_fn = parse_macro_input!(item as ItemFn);

    let _fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_block = &input_fn.block;
    let fn_sig = &input_fn.sig;
    let fn_inputs = &input_fn.sig.inputs;

    // Gather parameter names for building the event
    let param_idents: Vec<_> = fn_inputs.iter()
        .skip(1) // skip &self
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg {
                if let syn::Pat::Ident(ref pat_ident) = *pt.pat {
                    return Some(pat_ident.ident.clone());
                }
            }
            None
        })
        .collect();

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            // run the user’s original method body
            let __user_result = {
                #fn_block
            };

            {
                use ::nexus_events::core::dispatch;
                // build an event from the method params
                let evt = #event_ty {
                    #(#param_idents: #param_idents),*
                };
                // dispatch it to the global bus
                dispatch(evt);
            }

            __user_result
        }
    };
    TokenStream::from(expanded)
}
