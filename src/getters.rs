// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use proc_macro2::TokenStream as TokenStream2;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Fields, Result};
use quote::ToTokens;

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident_name = &input.ident;

    let data = match input.data {
        Data::Struct(ref data) => data,
        Data::Enum(_) => Err(Error::new_spanned(
            &input,
            "Deriving getters is not supported in enums",
        ))?,
        //strict_encode_inner_enum(&input, &data),
        Data::Union(_) => Err(Error::new_spanned(
            &input,
            "Deriving getters is not supported in unions",
        ))?,
    };

    let recurse = match data.fields {
        Fields::Named(ref fields) => fields.named.iter().map(|f| {
            let name = f.ident.as_ref().expect("named field always has a name");
            let doc = f
                .attrs
                .iter()
                .find(|a| {
                    a.path
                        .segments
                        .first()
                        .map(|p| p.ident.to_string() == "doc")
                        .unwrap_or(false)
                })
                .map(|doc| doc.into_token_stream())
                .unwrap_or_else(|| {
                    let doc = format!("Method for accessing [`{}::{}`] field", ident_name, name);
                    quote! {
                        #[doc = #doc]
                    }
                });
            let ty = &f.ty;
            quote_spanned! { f.span() =>
                #doc
                #[inline]
                pub fn #name(&self) -> &#ty {
                    &self.#name
                }
            }
        }),
        Fields::Unnamed(_) => Err(Error::new_spanned(
            &input,
            "Deriving getters is not supported for tuple-bases structs",
        ))?,
        Fields::Unit => Err(Error::new_spanned(
            &input,
            "Deriving getters is meaningless for unit structs",
        ))?,
    };

    Ok(quote! {
        impl #impl_generics #ident_name #ty_generics #where_clause {
            #( #recurse )*
        }
    })
}
