use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemEnum, ItemStruct, Variant, parse_macro_input, parse_str};

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
            fn as_automation_message(&self) -> Option<iced_automation::AutomationMessageView> {
                match self {
                    #enum_name::AutomationBounds(rect) => {
                        Some(iced_automation::AutomationMessageView::Bounds(*rect))
                    }
                    #enum_name::AutomationVirtualTick(duration) => {
                        Some(iced_automation::AutomationMessageView::VirtualTick(*duration))
                    }
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

        let field2: syn::Field = syn::parse_quote! {
            pub demo_root_path: Option<std::path::PathBuf>
        };

        fields.named.push(field1);
        fields.named.push(field2);
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
