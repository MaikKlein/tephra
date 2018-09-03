#![recursion_limit = "128"]
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;
use proc_macro2::TokenStream;
use syn::{Data, DataStruct, DeriveInput, Ident};
#[proc_macro_derive(VertexInput)]
pub fn derive_vertex_inputsize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let derive = match &input.data {
        Data::Struct(_struct) => gen_vertex_input(&input.ident, _struct),
        _ => panic!("Must be a struct"),
    };
    derive.into()
}

fn gen_vertex_input(
    ident: &Ident,
    input: &DataStruct,
) -> TokenStream {
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
