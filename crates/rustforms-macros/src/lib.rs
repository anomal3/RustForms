//! Макросы RustForms (`#[derive(Control)]`).

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Index, Member, parse_macro_input};

/// Связывает контрол с его родителем.
///
/// С полем `#[base]` контрол получает всё поведение этого поля («наследует» от него):
/// не переопределённые методы `ControlImpl` уходят туда, а свойства доступны через `Deref`.
/// Без `#[base]` это контрол с нуля.
#[proc_macro_derive(Control, attributes(base))]
pub fn derive_control(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input).unwrap_or_else(Error::into_compile_error).into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(&input.ident, "Control выводится только для структур"));
    };
    let mut marked = data
        .fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field.attrs.iter().any(|a| a.path().is_ident("base")));
    let base = marked.next();
    if let Some((_, second)) = marked.next() {
        return Err(Error::new_spanned(second, "поле #[base] может быть только одно"));
    }

    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let Some((index, field)) = base else {
        return Ok(quote! {
            impl #impl_generics ::rustforms::Control for #name #type_generics #where_clause {}
        });
    };
    let member = match &field.ident {
        Some(ident) => Member::Named(ident.clone()),
        None => Member::Unnamed(Index::from(index)),
    };
    let ty = &field.ty;

    Ok(quote! {
        impl #impl_generics ::rustforms::Control for #name #type_generics #where_clause {
            fn parent(&self) -> ::core::option::Option<&dyn ::rustforms::ControlImpl> {
                ::core::option::Option::Some(&self.#member)
            }

            fn parent_mut(
                &mut self,
            ) -> ::core::option::Option<&mut dyn ::rustforms::ControlImpl> {
                ::core::option::Option::Some(&mut self.#member)
            }
        }

        impl #impl_generics ::core::ops::Deref for #name #type_generics #where_clause {
            type Target = #ty;

            fn deref(&self) -> &#ty {
                &self.#member
            }
        }

        impl #impl_generics ::core::ops::DerefMut for #name #type_generics #where_clause {
            fn deref_mut(&mut self) -> &mut #ty {
                &mut self.#member
            }
        }
    })
}
