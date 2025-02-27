use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, DeriveInput, Error, Field,
    Fields, Path,
};

#[proc_macro_derive(Filter, attributes(filter))]
pub fn filter_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut stream = convert_struct(input.clone(), false);
    stream.extend(convert_struct(input, true));
    stream
}

fn convert_struct(mut input: DeriveInput, filter: bool) -> TokenStream {
    let suffix = if filter { "Filter" } else { "Main" };
    input.ident = Ident::new(&format!("{}{}", input.ident, suffix), Span::call_site());

    if let syn::Data::Struct(ref mut data) = input.data {
        if let Fields::Named(ref mut fields) = data.fields {
            if filter {
                filter_fields(&mut fields.named);
            }
            convert_fields(&mut fields.named, filter);
            return quote_struct(input);
        }
    }

    TokenStream::from(Error::new(input.ident.span(), "Not named struct").to_compile_error())
}

fn quote_struct(input: DeriveInput) -> TokenStream {
    TokenStream::from(quote! {
        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, clap::Args)]
        #input
    })
}

fn filter_fields(fields: &mut Punctuated<Field, Comma>) {
    let mut new_fields = Punctuated::new();
    while let Some(field) = fields.pop() {
        let field = field.into_value();
        if !field
            .attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("filter") {
                    let filter: Path = attr.parse_args().unwrap();
                    return Some(filter.segments);
                }
                None
            })
            .flatten()
            .any(|seg| seg.ident == "skip")
        {
            new_fields.push(field);
        }
    }
    *fields = new_fields;
}

fn convert_fields(fields: &mut Punctuated<Field, Comma>, filter: bool) {
    for field in fields {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let new_ty = if filter {
            quote! { Option<Vec<#ty>> }
        } else {
            quote! { Option<#ty> }
        };
        let vis = field.vis.clone();
        *field = parse_quote! {
            #[arg(long, value_delimiter = ',')]
            #name: #new_ty
        };
        field.vis = vis;
    }
}
