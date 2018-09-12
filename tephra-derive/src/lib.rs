#![recursion_limit = "128"]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;
use proc_macro2::TokenStream;
use syn::{Data, DataStruct, DeriveInput, Ident, Meta, NestedMeta};
#[proc_macro_derive(Descriptor, attributes(descriptor))]
pub fn derive_descriptor_info(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let derive = match &input.data {
        Data::Struct(_struct) => gen_descriptor_info(&input.ident, _struct),
        _ => panic!("Must be a struct"),
    };
    derive.into()
}
fn gen_descriptor_info(ident: &Ident, input: &DataStruct) -> TokenStream {
    let path = quote!{tephra::descriptor};
    let field_names = input
        .fields
        .iter()
        .map(|field| field.ident.as_ref().expect("name"));
    let descriptor = parse_descriptor_attributes(input)
        .zip(field_names)
        .map(|(desc, field)| {
            let binding = desc.binding;
            let ty = match desc.ty {
                DescriptorType::Storage => {
                    quote!{
                        #path::DescriptorResource::Storage(self.#field.to_generic_buffer())
                    }
                }
                DescriptorType::Uniform => {
                    quote!{
                        #path::DescriptorResource::Uniform(self.#field.to_generic_buffer())
                    }
                }
            };
            quote!{
                 #path::Binding {
                     binding: #binding,
                     data: #ty
                 }
            }
        });
    let layout = parse_descriptor_attributes(input).map(|desc| {
        let binding = desc.binding;
        let ty = match desc.ty {
            DescriptorType::Storage => quote!{#path::DescriptorType::Storage},
            DescriptorType::Uniform => quote!{#path::DescriptorType::Uniform},
        };
        quote!{
            #path::Binding {
                binding: #binding,
                data: #ty
            }
        }
    });
    quote!{
        impl #path::DescriptorInfo for #ident {
            fn descriptor_data(&self) -> Vec<#path::Binding<#path::DescriptorResource>> {
                vec![
                    #(#descriptor),*
                ]
            }
            fn layout() -> Vec<#path::Binding<#path::DescriptorType>> {
                vec![
                    #(#layout),*
                ]
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Descriptor {
    binding: u32,
    ty: DescriptorType,
}

#[derive(Debug, Copy, Clone)]
enum DescriptorType {
    Storage,
    Uniform,
}
impl DescriptorType {
    pub fn from_meta(meta: &Meta) -> Self {
        let nested = match meta {
            Meta::List(list) => &list.nested,
            _ => panic!("Only Metalist is supported"),
        };
        match nested[0] {
            NestedMeta::Meta(ref meta) => match meta {
                Meta::Word(ident) => {
                    let name = ident.to_string();
                    match name.as_str() {
                        "Storage" => DescriptorType::Storage,
                        "Uniform" => DescriptorType::Uniform,
                        _ => panic!("Unknown type"),
                    }
                }
                _ => panic!("Expected Word"),
            },
            _ => panic!("Expected Meta"),
        }
    }
}

fn parse_descriptor_attributes<'input>(
    input: &'input DataStruct,
) -> impl Iterator<Item = Descriptor> + 'input {
    input.fields.iter().enumerate().map(|(idx, field)| {
        let descriptor = field
            .attrs
            .iter()
            .filter_map(|attr| attr.interpret_meta())
            .filter(|meta| meta.name() == "descriptor")
            .map(|meta| {
                let ty = DescriptorType::from_meta(&meta);
                Descriptor {
                    binding: idx as _,
                    ty: ty,
                }
            })
            .nth(0)
            .expect("meta");
        descriptor
    })
}

#[proc_macro_derive(VertexInput)]
pub fn derive_vertex_inputsize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let derive = match &input.data {
        Data::Struct(_struct) => gen_vertex_input(&input.ident, _struct),
        _ => panic!("Must be a struct"),
    };
    derive.into()
}

fn gen_vertex_input(ident: &Ident, input: &DataStruct) -> TokenStream {
    let tys = input.fields.iter().map(|field| &field.ty);
    quote!{
        impl tephra::renderpass::VertexInput for #ident {
            fn vertex_input_data() -> Vec<tephra::renderpass::VertexInputData> {
                use tephra::renderpass::VertexTypeData;
                let mut data = Vec::new();
                let mut offset = 0;
                // TODO: Explicit location and binding
                let mut location = 0;
                #(
                    {
                        let ty = <#tys>::vertex_type();
                        let vertex_input_data = tephra::renderpass::VertexInputData {
                            binding: 0,
                            location,
                            offset: offset,
                            vertex_type: ty,
                        };
                        location += 1;
                        offset += ty.size() as u32;
                        data.push(vertex_input_data);
                    }
                )*
                data
            }
        }
    }
}
