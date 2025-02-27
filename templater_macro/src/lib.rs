use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, DeriveInput, Field, Fields,
    FieldsNamed, Path,
};

#[proc_macro_derive(Filter, attributes(filter))]
pub fn filter_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut stream = TokenStream::new();
    stream.extend(create_impl(input.clone()));
    stream.extend(convert_struct(input.clone(), false));
    stream.extend(convert_struct(input, true));

    stream
}

fn create_impl(mut input: DeriveInput) -> TokenStream {
    let name_main = name(&input.ident, false);
    let name_filter = name(&input.ident, true);

    let fields = get_fields(&mut input);
    filter_fields(&mut fields.named);
    let vals = fields
        .named
        .iter()
        .map(|field| field.ident.as_ref().unwrap());
    let vals_i = vals.clone();
    let except_vals = vals
        .clone()
        .map(|f| Ident::new(&format!("except_{}", f), Span::call_site()));

    TokenStream::from(quote! {
        impl PartialEq<#name_main> for #name_filter {
            fn eq(&self, other: &#name_main) -> bool {
                #(check(self.#vals.as_ref(), other.#vals.as_ref(), true))&&*
                && #(!check(self.#except_vals.as_ref(), other.#vals_i.as_ref(), false))&&*
            }
        }
    })
}

fn name(name: &Ident, filter: bool) -> Ident {
    let suffix = if filter { "Filter" } else { "Main" };
    Ident::new(&format!("{}{}", name, suffix), Span::call_site())
}

fn convert_struct(mut input: DeriveInput, filter: bool) -> TokenStream {
    input.ident = name(&input.ident, filter);
    let fields = get_fields(&mut input);

    if filter {
        filter_fields(&mut fields.named);
    }
    fields.named = convert_fields(&fields.named, filter);
    quote_struct(input)
}

fn get_fields(input: &mut DeriveInput) -> &mut FieldsNamed {
    let syn::Data::Struct(ref mut data) = input.data else {
        panic!()
    };
    let Fields::Named(ref mut fields) = data.fields else {
        panic!()
    };
    fields
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

fn convert_fields(fields: &Punctuated<Field, Comma>, filter: bool) -> Punctuated<Field, Comma> {
    let mut new_fields = Punctuated::new();
    for field in fields {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let new_ty = if filter {
            quote! { Option<Vec<#ty>> }
        } else {
            quote! { Option<#ty> }
        };
        new_fields.push(quote_field(name, &new_ty));
        let name = Ident::new(&format!("except_{}", name), Span::call_site());
        new_fields.push(quote_field(&name, &new_ty));
    }
    new_fields
}

fn quote_field(name: &Ident, ty: &TokenStream2) -> Field {
    parse_quote! {
        #[arg(long, value_delimiter = ',')]
        pub #name: #ty
    }
}
