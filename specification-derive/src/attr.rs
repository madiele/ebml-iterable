use proc_macro2::TokenStream;
use std::str::FromStr;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{Attribute, ItemEnum, Result, Error, Visibility, Fields, FieldsUnnamed, Path, Ident, Variant};
use quote::{quote, quote_spanned};
use ebml_iterable_specification::TagDataType;

use super::ast::Enum;

pub fn impl_ebml_specification(original: &mut ItemEnum) -> Result<TokenStream> {
    let input = Enum::from_syn(original)?;

    let mut used_ids = HashMap::<u64, &Variant>::new();
    for var in &input.variants {
        if let Some(original) = used_ids.insert(var.id_attr.0, var.original) {
            let mut err = Error::new_spanned(var.original, "duplicate #[id()] detected");
            err.combine(Error::new_spanned(original, "#[id()] already used previously"));
            return Err(err);
        } 
    }

    let ebml_specification_impl = get_impl(input)?;
    let modified_orig = modify_orig(original)?;

    Ok(quote!(
        #modified_orig

        #ebml_specification_impl
    ))
}

fn modify_orig(original: &mut ItemEnum) -> Result<TokenStream> {
    let spanned_master_enum = spanned_master_enum(original).clone();
    for var in original.variants.iter_mut() {
        let data_type_attribute: &Attribute = var
            .attrs
            .iter()
            .find(|a| a.path.is_ident("data_type"))
            .expect("#[data_type()] attribute required for variants under #[ebml_specification]");
            
        let data_type_path = data_type_attribute.parse_args::<Path>().map_err(|err| Error::new(err.span(), "#[data_type()] requires `ebml_iterable::TagDataType`"))?;
        let data_type = get_last_path_ident(&data_type_path).ok_or(Error::new_spanned(data_type_attribute.clone(), "#[data_type()] requires `ebml_iterable::TagDataType`"))?;
        
        let data_type = if data_type == "Master" {
            let orig_ident = &original.ident;
            quote!( (#spanned_master_enum<#orig_ident>) )
        } else if data_type == "UnsignedInt" {
            quote!( (u64) )
        } else if data_type == "Integer" {
            quote!( (i64) )
        } else if data_type == "Utf8" {
            quote!( (String) )
        } else if data_type == "Binary" {
            quote!( (::std::vec::Vec<u8>) )
        } else if data_type == "Float" {
            quote!( (f64) )
        } else {
            return Err(Error::new_spanned(data_type_attribute.clone(), format!("unknown data_type \"{}\"", data_type)));
        };

        var.attrs.retain(|a| {
            if a.path.is_ident("id") || a.path.is_ident("data_type") {
                false
            } else {
                true
            }
        });
        var.fields = Fields::Unnamed(syn::parse2::<FieldsUnnamed>(data_type)?);
    }
    original.variants.push(syn::parse_str::<Variant>("RawTag(u64, ::std::vec::Vec<u8>)")?);

    Ok(quote!(#original))
}

fn get_impl(input: Enum) -> Result<TokenStream> {
    let ty = &input.ident;
    let spanned_master_enum = spanned_master_enum(input.original);

    let get_tag_data_type = input.variants.iter()
        .filter(|v| !matches!(&v.data_type_attr.0, TagDataType::Binary))
        .map(|var: &crate::ast::Variant| {
            let id = &var.id_attr.0;
            let data_type = &var.data_type_attr.1;

            quote! {
                if id == #id {
                    #data_type
                }
            }
        });

    let get_id = input.variants.iter().map(|var: &crate::ast::Variant| {
        let name = &var.ident;
        let id = &var.id_attr.0;

        quote! {
            #ty::#name(_) => #id,
        }
    });

    let get_tag = |ret_val: String| {
        move |var: &crate::ast::Variant| {
            let name = &var.ident;
            let id = &var.id_attr.0;
            let ret_val = TokenStream::from_str(&ret_val).expect("Misuse of get_tag function in ebml_iterable_specification_derive_attr");
    
            quote! {
                if id == #id {
                    Some(#ty::#name(#ret_val))
                }
            }
        }
    };

    let get_unsigned_int_tag = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::UnsignedInt))
        .map(get_tag(String::from("data")));

    let get_signed_int_tag = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Integer))
        .map(get_tag(String::from("data")));

    let get_utf8_tag = input.variants.iter()
    .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Utf8))
        .map(get_tag(String::from("data")));

    let get_binary_tag = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Binary))
        .map(get_tag(String::from("data.to_vec()")));

    let get_float_tag = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Float))
        .map(get_tag(String::from("data")));

    let get_master_tag = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Master))
        .map(get_tag(String::from("data")));

    let as_data = |var: &crate::ast::Variant| {
        let name = &var.ident;

        quote! {
            #ty::#name(val) => Some(val),
        }
    };

    let as_unsigned_int = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::UnsignedInt))
        .map(as_data);

    let as_signed_int = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Integer))
        .map(as_data);

    let as_utf8 = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Utf8))
        .map(as_data);

    let as_binary = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Binary))
        .map(as_data);

    let as_float = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Float))
        .map(as_data);

    let as_master = input.variants.iter()
        .filter(|v| matches!(&v.data_type_attr.0, TagDataType::Master))
        .map(as_data);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ebml_spec_trait = spanned_ebml_specification_trait(input.original);
    let ebml_tag_trait = spanned_ebml_tag_trait(input.original);
    let tag_data_type = spanned_tag_data_type(input.original);

    Ok(quote! {        
        impl #impl_generics #ebml_spec_trait <#ty> for #ty #ty_generics #where_clause {
            fn get_tag_data_type(id: u64) -> #tag_data_type {
                #(#get_tag_data_type else)* {
                    #tag_data_type::Binary
                }
            }

            fn get_unsigned_int_tag(id: u64, data: u64) -> Option<#ty> {
                #(#get_unsigned_int_tag else)* {
                    None
                }
            }

            fn get_signed_int_tag(id: u64, data: i64) -> Option<#ty> {
                #(#get_signed_int_tag else)* {
                    None
                }
            }

            fn get_utf8_tag(id: u64, data: String) -> Option<#ty> {
                #(#get_utf8_tag else)* {
                    None
                }
            }

            fn get_binary_tag(id: u64, data: &[u8]) -> Option<#ty> {
                #(#get_binary_tag else)* {
                    None
                }
            }

            fn get_float_tag(id: u64, data: f64) -> Option<#ty> {
                #(#get_float_tag else)* {
                    None
                }
            }

            fn get_master_tag(id: u64, data: #spanned_master_enum<#ty>) -> Option<#ty> {
                #(#get_master_tag else)* {
                    None
                }
            }

            fn get_raw_tag(id: u64, data: &[u8]) -> #ty {
                #ty::RawTag(id, data.to_vec())
            }
        }

        impl #impl_generics #ebml_tag_trait <#ty> for #ty #ty_generics #where_clause {

            fn get_id(&self) -> u64 {
                match self {
                    #(#get_id)*
                    #ty::RawTag(id, _data) => *id,
                }
            }

            fn as_unsigned_int(&self) -> Option<&u64> {
                match self {
                    #(#as_unsigned_int)*
                    _ => None,
                }
            }

            fn as_signed_int(&self) -> Option<&i64> {
                match self {
                    #(#as_signed_int)*
                    _ => None,
                }
            }

            fn as_utf8(&self) -> Option<&str> {
                match self {
                    #(#as_utf8)*
                    _ => None,
                }
            }

            fn as_binary(&self) -> Option<&[u8]> {
                match self {
                    #(#as_binary)*
                    #ty::RawTag(_id, data) => Some(data),
                    _ => None,
                }
            }

            fn as_float(&self) -> Option<&f64> {
                match self {
                    #(#as_float)*
                    _ => None,
                }
            }

            fn as_master(&self) -> Option<&#spanned_master_enum<#ty>> {
                match self {
                    #(#as_master)*
                    _ => None,
                }
            }
        }
    })
}

fn spanned_ebml_iterable_specs(input: &ItemEnum) -> TokenStream {
    let vis_span = match &input.vis {
        Visibility::Public(vis) => Some(vis.pub_token.span()),
        Visibility::Crate(vis) => Some(vis.crate_token.span()),
        Visibility::Restricted(vis) => Some(vis.pub_token.span()),
        Visibility::Inherited => None,
    };
    let data_span = input.enum_token.span();
    let first_span = vis_span.unwrap_or(data_span);
    quote_spanned!(first_span=> ebml_iterable::specs::)
}

fn spanned_master_enum(input: &ItemEnum) -> TokenStream {
    let path = spanned_ebml_iterable_specs(input);
    let last_span = input.ident.span();
    let r#enum = quote_spanned!(last_span=> Master);
    quote!(#path #r#enum)
}

fn spanned_ebml_specification_trait(input: &ItemEnum) -> TokenStream {
    let path = spanned_ebml_iterable_specs(input);
    let last_span = input.ident.span();
    let spec = quote_spanned!(last_span=> EbmlSpecification);
    quote!(#path #spec)
}

fn spanned_ebml_tag_trait(input: &ItemEnum) -> TokenStream {
    let path = spanned_ebml_iterable_specs(input);
    let last_span = input.ident.span();
    let spec = quote_spanned!(last_span=> EbmlTag);
    quote!(#path #spec)
}

fn spanned_tag_data_type(input: &ItemEnum) -> TokenStream {
    let path = spanned_ebml_iterable_specs(input);
    let last_span = input.ident.span();
    let r#type = quote_spanned!(last_span=> TagDataType);
    quote!(#path #r#type)
}

fn get_last_path_ident(path: &Path) -> Option<&Ident> {
    let seg = path.segments.iter().last();
    if seg.is_none() {
        None
    } else {
        Some(&seg.unwrap().ident)
    }
}