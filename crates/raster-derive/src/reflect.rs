use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Expr, Fields, Lit, Type, parse2, spanned::Spanned};

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "Reflect can only be derived for structs; enums and unions are not supported yet",
        ));
    };

    let Fields::Named(named) = &data.fields else {
        return Err(syn::Error::new(
            input.span(),
            "Reflect requires named fields; tuple structs have no field names to reflect",
        ));
    };

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(
            input.generics.span(),
            "Reflect does not support generic types: a generic has no single TypeInfo",
        ));
    }

    let mut fields = Vec::new();
    for field in &named.named {
        let attrs = parse_attrs(field)?;
        if attrs.skip {
            continue;
        }
        let ident = field.ident.clone().expect("named fields checked above");
        fields.push(ReflectedField {
            ident,
            ty: field.ty.clone(),
            attrs,
        });
    }

    let fields_const = format_ident!("__RASTER_FIELDS_{}", name);
    let info_const = format_ident!("__RASTER_TYPEINFO_{}", name);
    let field_infos = fields.iter().map(field_info);
    let getters = fields.iter().map(getter_arm);
    let setters = fields.iter().map(|f| setter_arm(f, true));
    let unchecked_setters = fields.iter().map(|f| setter_arm(f, false));
    let serialized_setters = fields.iter().map(serialized_setter_arm);
    let to_value_entries = fields.iter().map(to_value_entry);
    let name_str = name.to_string();
    let count = fields.len();

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        static #fields_const: [::raster_core::reflect::FieldInfo; #count] = [#(#field_infos),*];

        #[allow(non_upper_case_globals)]
        static #info_const: ::raster_core::reflect::TypeInfo = ::raster_core::reflect::TypeInfo {
            name: #name_str,
            fields: &#fields_const,
        };

        impl ::raster_core::reflect::Reflect for #name {
            fn type_info() -> &'static ::raster_core::reflect::TypeInfo {
                &#info_const
            }

            fn type_info_dyn(&self) -> &'static ::raster_core::reflect::TypeInfo {
                &#info_const
            }

            fn get_field(&self, name: &str) -> ::core::option::Option<::raster_core::reflect::Value> {
                match name {
                    #(#getters,)*
                    _ => ::core::option::Option::None,
                }
            }

            fn set_field(
                &mut self,
                name: &str,
                value: ::raster_core::reflect::Value,
            ) -> ::core::result::Result<(), ::raster_core::reflect::ReflectError> {
                match name {
                    #(#setters,)*
                    _ => ::core::result::Result::Err(
                        ::raster_core::reflect::ReflectError::UnknownField {
                            type_name: #name_str,
                            field: name.to_owned(),
                        },
                    ),
                }
            }

            fn set_field_by_serialized_name(
                &mut self,
                name: &str,
                value: ::raster_core::reflect::Value,
            ) -> ::core::result::Result<(), ::raster_core::reflect::ReflectError> {
                match name {
                    #(#serialized_setters,)*
                    _ => ::core::result::Result::Err(
                        ::raster_core::reflect::ReflectError::UnknownField {
                            type_name: #name_str,
                            field: name.to_owned(),
                        },
                    ),
                }
            }

            fn set_field_unchecked(
                &mut self,
                name: &str,
                value: ::raster_core::reflect::Value,
            ) -> ::core::result::Result<(), ::raster_core::reflect::ReflectError> {
                match name {
                    #(#unchecked_setters,)*
                    _ => ::core::result::Result::Err(
                        ::raster_core::reflect::ReflectError::UnknownField {
                            type_name: #name_str,
                            field: name.to_owned(),
                        },
                    ),
                }
            }

            fn to_value(&self) -> ::raster_core::reflect::Value {
                let mut fields = ::std::collections::BTreeMap::new();
                #(#to_value_entries)*
                ::raster_core::reflect::Value::Struct(fields)
            }
        }
    })
}

struct ReflectedField {
    ident: syn::Ident,
    ty: Type,
    attrs: Attrs,
}

#[derive(Default)]
struct Attrs {
    skip: bool,
    readonly: bool,
    min: Option<f64>,
    max: Option<f64>,
    rename: Option<String>,
    tooltip: Option<String>,
}

fn parse_attrs(field: &syn::Field) -> syn::Result<Attrs> {
    let mut attrs = Attrs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("property") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            let key = meta
                .path
                .get_ident()
                .ok_or_else(|| meta.error("expected a property attribute name"))?
                .to_string();

            match key.as_str() {
                "skip" => attrs.skip = true,
                "readonly" => attrs.readonly = true,
                "min" => attrs.min = Some(number(&meta)?),
                "max" => attrs.max = Some(number(&meta)?),
                "rename" => attrs.rename = Some(string(&meta)?),
                "tooltip" => attrs.tooltip = Some(string(&meta)?),
                other => {
                    return Err(meta.error(format!(
                        "unknown property attribute `{other}`; expected one of skip, readonly, \
                         min, max, rename, tooltip"
                    )));
                }
            }
            Ok(())
        })?;
    }

    if let (Some(min), Some(max)) = (attrs.min, attrs.max) {
        if min > max {
            return Err(syn::Error::new(
                field.span(),
                format!("min ({min}) is greater than max ({max})"),
            ));
        }
    }

    Ok(attrs)
}

fn number(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<f64> {
    let value: Expr = meta.value()?.parse()?;
    let (lit, negative) = match &value {
        Expr::Lit(lit) => (&lit.lit, false),
        // A negative literal parses as a unary expression, not a literal.
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => match &*unary.expr {
            Expr::Lit(lit) => (&lit.lit, true),
            other => return Err(syn::Error::new(other.span(), "expected a number")),
        },
        other => return Err(syn::Error::new(other.span(), "expected a number")),
    };

    let magnitude = match lit {
        Lit::Float(f) => f.base10_parse::<f64>()?,
        Lit::Int(i) => i.base10_parse::<f64>()?,
        other => return Err(syn::Error::new(other.span(), "expected a number")),
    };

    Ok(if negative { -magnitude } else { magnitude })
}

fn string(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    let value: Expr = meta.value()?.parse()?;
    match &value {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Str(s) => Ok(s.value()),
            other => Err(syn::Error::new(other.span(), "expected a string literal")),
        },
        other => Err(syn::Error::new(other.span(), "expected a string literal")),
    }
}

fn field_info(field: &ReflectedField) -> TokenStream {
    let name = field.ident.to_string();
    let serialized = field.attrs.rename.clone().unwrap_or_else(|| name.clone());
    let ty = &field.ty;
    let readonly = field.attrs.readonly;

    let min = option_f64(field.attrs.min);
    let max = option_f64(field.attrs.max);
    let tooltip = match &field.attrs.tooltip {
        Some(t) => quote!(::core::option::Option::Some(#t)),
        None => quote!(::core::option::Option::None),
    };

    quote! {
        ::raster_core::reflect::FieldInfo {
            name: #name,
            serialized_name: #serialized,
            kind: <#ty as ::raster_core::reflect::ReflectValue>::KIND,
            attrs: ::raster_core::reflect::PropertyAttrs {
                readonly: #readonly,
                min: #min,
                max: #max,
                tooltip: #tooltip,
            },
        }
    }
}

fn option_f64(value: Option<f64>) -> TokenStream {
    match value {
        Some(v) => quote!(::core::option::Option::Some(#v)),
        None => quote!(::core::option::Option::None),
    }
}

fn getter_arm(field: &ReflectedField) -> TokenStream {
    let ident = &field.ident;
    let name = ident.to_string();
    quote! {
        #name => ::core::option::Option::Some(
            ::raster_core::reflect::ReflectValue::to_reflect_value(&self.#ident)
        )
    }
}

fn setter_arm(field: &ReflectedField, honour_readonly: bool) -> TokenStream {
    let name = field.ident.to_string();

    if honour_readonly && field.attrs.readonly {
        return quote! {
            #name => ::core::result::Result::Err(
                ::raster_core::reflect::ReflectError::Readonly { field: #name.to_owned() }
            )
        };
    }

    let body = setter_body(field);
    quote! { #name => { #body } }
}

/// Le corps partage par les trois setters generes : conversion, bornage, ecriture.
fn setter_body(field: &ReflectedField) -> TokenStream {
    let ident = &field.ident;
    let name = ident.to_string();
    let ty = &field.ty;

    let clamp = match (field.attrs.min, field.attrs.max) {
        (Some(min), Some(max)) => quote!(let parsed = parsed.clamp(#min as #ty, #max as #ty);),
        (Some(min), None) => quote!(let parsed = parsed.max(#min as #ty);),
        (None, Some(max)) => quote!(let parsed = parsed.min(#max as #ty);),
        (None, None) => quote!(),
    };

    quote! {
        let parsed = <#ty as ::raster_core::reflect::ReflectValue>::from_reflect_value(&value)
            .map_err(|detail| ::raster_core::reflect::ReflectError::TypeMismatch {
                field: #name.to_owned(),
                expected: detail,
                got: value.kind_name(),
            })?;
        #clamp
        self.#ident = parsed;
        ::core::result::Result::Ok(())
    }
}

/// Comme `setter_arm(field, false)`, mais matche sur le nom serialise : c'est
/// celui que porte un fichier de scene.
fn serialized_setter_arm(field: &ReflectedField) -> TokenStream {
    let serialized = field
        .attrs
        .rename
        .clone()
        .unwrap_or_else(|| field.ident.to_string());
    let body = setter_body(field);
    quote! { #serialized => { #body } }
}

fn to_value_entry(field: &ReflectedField) -> TokenStream {
    let ident = &field.ident;
    let serialized = field
        .attrs
        .rename
        .clone()
        .unwrap_or_else(|| ident.to_string());
    quote! {
        fields.insert(
            #serialized.to_owned(),
            ::raster_core::reflect::ReflectValue::to_reflect_value(&self.#ident),
        );
    }
}
