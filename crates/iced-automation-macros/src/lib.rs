use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Fields, ItemEnum, ItemStruct, LitStr, Token, Variant, parse_macro_input, parse_str,
};

#[proc_macro_attribute]
pub fn message(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_enum = parse_macro_input!(input as ItemEnum);

    let variant1: Variant = syn::parse_quote! {
        #[serde(skip_deserializing)]
        AutomationBounds(Option<iced::Rectangle>)
    };

    let variant2: Variant = syn::parse_quote! {
        #[serde(skip_deserializing)]
        AutomationVirtualTick(std::time::Duration)
    };

    item_enum.variants.push(variant1);
    item_enum.variants.push(variant2);

    let enum_name = &item_enum.ident;

    let expanded = quote! {
        #item_enum

        impl iced_automation::AutomationMessage for #enum_name {
            fn is_automation(&self) -> bool {
                matches!(
                    self,
                    #enum_name::AutomationBounds(_) | #enum_name::AutomationVirtualTick(_)
                )
            }

            fn as_bounds(&self) -> Option<iced::Rectangle> {
                match self {
                    #enum_name::AutomationBounds(rect_opt) => *rect_opt,
                    _ => None,
                }
            }

            fn as_virtual_tick(&self) -> Option<std::time::Duration> {
                match self {
                    #enum_name::AutomationVirtualTick(delta) => Some(*delta),
                    _ => None,
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn state(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let msg_type_str = args.to_string().trim().to_string();
    if msg_type_str.is_empty() {
        panic!(
            "state attribute macro expects the message enum type name as an argument, e.g. #[state(Message)]"
        );
    }
    let msg_type: syn::Path = parse_str(&msg_type_str).unwrap();

    if let Fields::Named(ref mut fields) = item_struct.fields {
        let field1: syn::Field = syn::parse_quote! {
            pub automation: Option<iced_automation::AutomationState<#msg_type>>
        };

        fields.named.push(field1);
    } else {
        panic!("state macro only supports structs with named fields");
    }

    let struct_name = &item_struct.ident;

    let expanded = quote! {
        #item_struct

        impl iced_automation::AutomationStateTrait<#msg_type> for #struct_name {
            fn automation(&self) -> &Option<iced_automation::AutomationState<#msg_type>> {
                &self.automation
            }
            fn automation_mut(&mut self) -> &mut Option<iced_automation::AutomationState<#msg_type>> {
                &mut self.automation
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(AutomationKeycap, attributes(automation))]
pub fn derive_automation_keycap(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        syn::Data::Enum(data) => &data.variants,
        _ => panic!("AutomationKeycap can only be derived for enums"),
    };

    let match_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_name = &v.ident;
            let keycap: Option<String> = v
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("automation"))
                .and_then(|attr| {
                    attr.parse_args_with(|input: syn::parse::ParseStream| {
                        let ident: syn::Ident = input.parse()?;
                        if ident != "keycap" {
                            return Err(input.error("expected `keycap`"));
                        }
                        input.parse::<Token![=]>()?;
                        let label: LitStr = input.parse()?;
                        Ok(label.value())
                    })
                    .ok()
                });

            match keycap {
                Some(label) => {
                    let (pat, _guard) = build_pattern(&v.fields);
                    quote! {
                        #name::#variant_name #pat => Some(#label)
                    }
                }
                _ => {
                    let (_pat, _guard) = build_pattern(&v.fields);
                    quote! {}
                }
            }
        })
        .filter(|arm| !arm.is_empty())
        .collect();

    let expanded = if match_arms.is_empty() {
        quote! {
            impl #name {
                #[allow(dead_code)]
                pub fn automation_keycap(&self) -> Option<&'static str> {
                    None
                }
            }
        }
    } else {
        let fallback = quote! { _ => None };
        quote! {
            impl #name {
                #[allow(dead_code)]
                pub fn automation_keycap(&self) -> Option<&'static str> {
                    match self {
                        #(#match_arms,)*
                        #fallback
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn build_pattern(fields: &syn::Fields) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    match fields {
        Fields::Unit => (quote! {}, quote! {}),
        Fields::Unnamed(_) => (quote! { (..) }, quote! {}),
        Fields::Named(_) => (quote! { { .. } }, quote! {}),
    }
}

#[proc_macro_derive(AutomationKeycapDispatch, attributes(automation))]
pub fn derive_automation_keycap_dispatch(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        syn::Data::Enum(data) => &data.variants,
        _ => panic!("AutomationKeycapDispatch can only be derived for enums"),
    };

    let mut dispatch_arms = Vec::new();
    let mut has_any = false;

    for variant in variants {
        let variant_name = &variant.ident;
        let is_dispatch = variant.attrs.iter().any(|attr| {
            attr.path().is_ident("automation") && {
                attr.parse_args_with(|input: syn::parse::ParseStream| {
                    let ident: syn::Ident = input.parse()?;
                    Ok(ident == "dispatch")
                })
                .unwrap_or(false)
            }
        });

        if is_dispatch {
            has_any = true;
            let binding_name = quote! { __inner };
            let pat = match &variant.fields {
                Fields::Unnamed(_) => quote! { (#binding_name) },
                _ => panic!(
                    "AutomationKeycapDispatch requires single unnamed field variants, found {:?}",
                    variant_name
                ),
            };
            dispatch_arms.push(quote! {
                #name::#variant_name #pat => #binding_name.automation_keycap()
            });
        }
    }

    let expanded = if has_any {
        quote! {
            impl #name {
                #[allow(dead_code)]
                pub fn automation_keycap(&self) -> Option<&'static str> {
                    match self {
                        #(#dispatch_arms,)*
                        _ => None,
                    }
                }
            }
        }
    } else {
        quote! {
            impl #name {
                #[allow(dead_code)]
                pub fn automation_keycap(&self) -> Option<&'static str> {
                    None
                }
            }
        }
    };

    TokenStream::from(expanded)
}
