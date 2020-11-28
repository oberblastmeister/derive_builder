use std::marker::PhantomData;

use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::MetaList;
use syn::MetaNameValue;
use syn::NestedMeta;
use syn::{Attribute, Data, DataStruct, DeriveInput, Error, Fields, Generics, Ident, Result, Type};

pub enum Input<'a> {
    Struct(Struct<'a>),
}

pub struct Struct<'a> {
    pub ident: &'a Ident,
    pub attrs: Attrs<'a>,
    pub original: &'a DeriveInput,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
}

pub struct Field<'a> {
    pub original: &'a syn::Field,
    pub attrs: Attrs<'a>,
    pub ident: &'a Ident,
    pub ty: &'a Type,
}

#[derive(Default)]
pub struct Attrs<'a> {
    pub each: Option<LitStr>,
    _marker: PhantomData<&'a ()>,
}

pub struct OuterAttrs<'a> {
    pub take: Option<&'a Attribute>,
}

impl<'a> Input<'a> {
    pub fn from_syn(node: &'a DeriveInput) -> Result<Input<'a>> {
        match &node.data {
            Data::Struct(data) => Struct::from_syn(node, data).map(Input::Struct),
            Data::Enum(_) => Err(Error::new_spanned(
                node,
                "Enum builders are not yet supported",
            )),
            Data::Union(_) => Err(Error::new_spanned(
                node,
                "Union builders are not yet supported",
            )),
        }
    }
}

impl<'a> Struct<'a> {
    pub fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Struct<'a>> {
        let attrs = Attrs::get(&node.attrs)?;
        let fields = Field::multiple_from_syn(&data.fields)?;
        let ident = &node.ident;
        Ok(Struct {
            ident,
            attrs,
            original: node,
            generics: &node.generics,
            fields,
        })
    }
}

impl<'a> Attrs<'a> {
    pub fn get(input: &'a [Attribute]) -> Result<Attrs<'a>> {
        let mut attrs = Attrs::default();
        for attr in input {
            let meta = attr.parse_meta()?;

            let meta_list = get_meta_list(meta)?;

            // check if the meta list path is correct
            if !meta_list.path.is_ident("builder") {
                return Err(Error::new_spanned(
                    attr,
                    "The meta attribute path was not `builder`",
                ));
            }

            for nested_meta in meta_list.nested {
                let meta = get_nested_meta_meta(nested_meta)?;
                let meta_name_value = get_meta_name_value(meta)?;

                if meta_name_value.path.is_ident("each") {
                    if attrs.each.is_some() {
                        return Err(Error::new_spanned(
                            meta_name_value,
                            "Duplicate meta key each",
                        ));
                    }
                    attrs.each = Some(expect_str_from_lit(meta_name_value.lit)?)
                } else {
                    return Err(Error::new_spanned(meta_name_value, "Unrecognized key"));
                }
            }
        }

        Ok(attrs)
    }
}

fn get_meta_list(meta: Meta) -> Result<MetaList> {
    match meta {
        Meta::List(meta_list) => Ok(meta_list),
        _ => {
            return Err(Error::new_spanned(
                meta,
                "The meta attribute must be a MetaList",
            ))
        }
    }
}

fn get_meta_name_value(meta: Meta) -> Result<MetaNameValue> {
    match meta {
        Meta::NameValue(meta_name_value) => Ok(meta_name_value),
        _ => {
            return Err(Error::new_spanned(
                meta,
                "The meta attribute must be a MetaNameValue",
            ))
        }
    }
}

fn get_nested_meta_meta(nested_meta: NestedMeta) -> Result<Meta> {
    match nested_meta {
        NestedMeta::Meta(meta) => Ok(meta),
        _ => {
            return Err(Error::new_spanned(
                nested_meta,
                "The nested meta must be another meta, not a literal",
            ))
        }
    }
}

fn expect_str_from_lit(lit: Lit) -> Result<LitStr> {
    match lit {
        Lit::Str(litstr) => Ok(litstr),
        _ => return Err(Error::new_spanned(lit, "Expect a LitStr from the Lit")),
    }
}

impl<'a> OuterAttrs<'a> {
    pub fn get(input: &[Attribute]) -> Result<OuterAttrs> {
        let mut attrs = OuterAttrs { take: None };
        Ok(attrs)
    }
}

impl<'a> Field<'a> {
    fn multiple_from_syn(fields: &'a Fields) -> Result<Vec<Self>> {
        fields.iter().map(|field| Field::from_syn(field)).collect()
    }

    fn from_syn(node: &'a syn::Field) -> Result<Self> {
        Ok(Field {
            original: node,
            attrs: Attrs::get(&node.attrs)?,
            ident: node
                .ident
                .as_ref()
                .ok_or_else(|| Error::new_spanned(node, "The struct's fields must be named"))?,
            ty: &node.ty,
        })
    }
}
