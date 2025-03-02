use syn::{parse_macro_input, DeriveInput, ItemFn};
use quote::{format_ident, quote};
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn event_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let event_type = parse_macro_input!(attr as syn::Path);
    let input = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_sig = &input.sig;
    let fn_block = &input.block;
    
    let register_fn_name = format_ident!("__register_{}", fn_name);
    
    let output = quote! {
        // Original method
        #fn_vis #fn_sig #fn_block
        
        // Registration method using FnMut
        #[doc(hidden)]
        fn #register_fn_name(&mut self, event_bus: &::nexus_events::core::shared::SharedEventBus) 
            -> ::nexus_events::subscriber::subscription::Subscription 
        {
            use ::std::any::TypeId;
            
            // Use FnMut instead of Fn for the handler
            event_bus.subscribe_any_mut(
                TypeId::of::<#event_type>(),
                Box::new(move |event, this: &mut Self| {
                    if let Some(typed_event) = event.downcast_ref::<#event_type>() {
                        this.#fn_name(typed_event);
                    }
                })
            )
        }
    };
    
    output.into()
}

#[proc_macro_attribute]
pub fn event_sender(attr: TokenStream, item: TokenStream) -> TokenStream {
    let event_type = parse_macro_input!(attr as syn::Path);
    let input = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let fn_vis = &input.vis;
    
    // Check if first parameter is &self
    if input.sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            input.sig.clone(),
            "#[event_sender] can only be used on methods with &self parameter"
        ).to_compile_error().into();
    }
    
    if !is_self_parameter(&input.sig.inputs[0]) {
        return syn::Error::new_spanned(
            input.sig.inputs[0].clone(),
            "#[event_sender] requires &self or &mut self as first parameter"
        ).to_compile_error().into();
    }
    
    // Extract parameter names for constructing the event
    // Skip the first parameter (&self)
    let param_names = input.sig.inputs.iter().skip(1).map(|arg| {
        match arg {
            syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => Ok(&pat_ident.ident),
                _ => Err(syn::Error::new_spanned(
                    pat_type.pat.clone(),
                    "Complex patterns not supported in #[event_sender] parameters"
                ))
            },
            _ => Err(syn::Error::new_spanned(
                arg.clone(),
                "Unexpected parameter type in #[event_sender]"
            ))
        }
    }).collect::<Result<Vec<_>, syn::Error>>();

    // Handle extraction errors
    let param_names = match param_names {
        Ok(names) => names,
        Err(err) => return err.to_compile_error().into(),
    };
    
    // Handle the original return type
    let (return_type, return_expr) = match &input.sig.output {
        syn::ReturnType::Default => {
            (
                quote! { -> ::nexus_events::core::error::EventResult<()> },
                quote! { Ok(()) }
            )
        },
        syn::ReturnType::Type(_, ty) => {
            (
                quote! { -> ::nexus_events::core::error::EventResult<#ty> },
                quote! { Ok(result) }
            )
        }
    };
    
    // Generate output that preserves the original function signature but adds event sending
    let all_args = &input.sig.inputs;
    
    // Handle empty parameter list (events with no fields)
    let event_constructor = if param_names.is_empty() {
        quote! { #event_type {} }
    } else {
        quote! { #event_type { #(#param_names: #param_names,)* } }
    };
    
    let output = quote! {
        /// Method generated by #[event_sender] attribute.
        /// 
        /// This method will:
        /// 1. Execute the original method body
        /// 2. Automatically send a `#event_type` event with the parameters
        /// 3. Return the original result wrapped in `EventResult`
        #[allow(clippy::semicolon_if_nothing_returned)]
        #fn_vis fn #fn_name(#all_args) #return_type {
            // Execute the original method body and capture any return value
            let result = {
                #fn_body
            };
            
            // Auto-send the event after method execution
            match self.sender().emit(#event_constructor) {
                Ok(_) => #return_expr,
                Err(e) => Err(e),
            }
        }
    };
    
    output.into()
}

// Helper function to check if parameter is &self or &mut self
fn is_self_parameter(arg: &syn::FnArg) -> bool {
    if let syn::FnArg::Receiver(_) = arg {
        return true;
    }
    false
}

#[proc_macro_derive(EventSubscriber)]
pub fn derive_event_subscriber(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let output = quote! {
        impl ::nexus_events::subscriber::event_subscriber::EventSubscriber for #name {
            fn id(&self) -> &str {
                &self.id
            }
            
            fn is_active(&self) -> bool {
                self.active
            }
            
            fn subscriptions(&mut self) -> &mut ::nexus_events::subscriber::subscription::SubscriptionSet {
                &mut self.subscriptions
            }
            
            fn register_event_handlers(&mut self, event_bus: &::nexus_events::core::shared::SharedEventBus) {
                let mut subscriptions = self.subscriptions();
                
                // Use a dynamic dispatch approach based on method name pattern
                ::nexus_events::_internal::invoke_registration_methods(self, event_bus, &mut subscriptions);
            }
        }
    };
    
    output.into()
}