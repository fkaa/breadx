// MIT/Apache2 License

use super::{
    syn_util::{int_litexpr_int, str_to_path, str_to_ty},
    InputParameter, Method, ParameterUsage, ToSyn, Type,
};
use proc_macro2::Span;
use std::{borrow::Cow, iter, ops::Deref};

#[derive(Debug)]
pub enum Trait {
    Event(u64),
    Error(u64),
    Request(u64, Type),
    Xid,
    EnumDefault(Box<str>),
    FromXid(Box<str>),
}

#[inline]
fn opcode_const(op: u64) -> syn::ImplItem {
    syn::ImplItem::Const(syn::ImplItemConst {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        const_token: Default::default(),
        ident: syn::Ident::new("OPCODE", Span::call_site()),
        colon_token: Default::default(),
        ty: Type::Basic("u8".into()).to_syn_ty(),
        eq_token: Default::default(),
        expr: int_litexpr_int(op),
        semi_token: Default::default(),
    })
}

impl Trait {
    #[inline]
    pub fn to_syn_item(self, tyname: &str) -> Vec<syn::Item> {
        vec![syn::Item::Impl(syn::ItemImpl {
            attrs: vec![],
            defaultness: None,
            unsafety: None,
            impl_token: Default::default(),
            generics: Default::default(),
            trait_: Some((
                None,
                match self {
                    Self::Event(_) => str_to_path("Event"),
                    Self::Error(_) => str_to_path("Error"),
                    Self::Request(_, _) => str_to_path("Request"),
                    Self::Xid => str_to_path("XidType"),
                    Self::EnumDefault(_) => str_to_path("Default"),
                    Self::FromXid(ref from) => syn::Path {
                        leading_colon: None,
                        segments: vec![syn::PathSegment {
                            ident: syn::Ident::new("From", Span::call_site()),
                            arguments: syn::PathArguments::AngleBracketed(
                                syn::AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Default::default(),
                                    args: iter::once(syn::GenericArgument::Type(str_to_ty(from)))
                                        .collect(),
                                    gt_token: Default::default(),
                                },
                            ),
                        }]
                        .into_iter()
                        .collect(),
                    },
                },
                Default::default(),
            )),
            self_ty: Box::new(str_to_ty(tyname)),
            brace_token: Default::default(),
            items: match self {
                Self::Event(opcode) => vec![opcode_const(opcode)],
                Self::Error(opcode) => vec![opcode_const(opcode)],
                Self::Request(opcode, reply_name) => vec![
                    opcode_const(opcode),
                    syn::ImplItem::Type(syn::ImplItemType {
                        attrs: vec![],
                        vis: syn::Visibility::Inherited,
                        defaultness: None,
                        type_token: Default::default(),
                        ident: syn::Ident::new("Reply", Span::call_site()),
                        generics: Default::default(),
                        eq_token: Default::default(),
                        ty: reply_name.to_syn_ty(),
                        semi_token: Default::default(),
                    }),
                ],
                Self::Xid => vec![
                    {
                        let mut method = Method::new(
                            "xid".into(),
                            Some(ParameterUsage::Ref),
                            vec![],
                            Some(Type::Basic("XID".into())),
                        );
                        method.statements = vec![super::GetXidStatement.into()];
                        method.to_syn_impl_item(true)
                    },
                    {
                        let mut method = Method::new(
                            "from_xid".into(),
                            None,
                            vec![InputParameter {
                                name: "xid".into(),
                                ty: Type::Basic("XID".into()),
                                usage: ParameterUsage::Owned,
                            }],
                            Some(Type::Basic("Self".into())),
                        );
                        method.statements = vec![super::CreateXidTypeStatement.into()];
                        method.to_syn_impl_item(true)
                    },
                ],
                Self::EnumDefault(variant) => vec![{
                    let mut method = Method::new(
                        "default".into(),
                        None,
                        vec![],
                        Some(Type::Basic(tyname.to_string().into())),
                    );
                    method.statements = vec![super::ReturnEnumVariant(
                        tyname.to_string().into_boxed_str(),
                        variant,
                    )
                    .into()];
                    method.to_syn_impl_item(true)
                }],
                Self::FromXid(from) => vec![{
                    let mut method = Method::new(
                        "from".into(),
                        None,
                        vec![InputParameter {
                            name: "base".into(),
                            ty: Type::Basic(from.deref().to_string().into()),
                            usage: ParameterUsage::Owned,
                        }],
                        Some(Type::Basic("Self".into())),
                    );
                    method.statements = vec![super::ConvertXids {
                        oldname: from.clone(),
                        newname: tyname.to_string().into_boxed_str(),
                    }
                    .into()];
                    method.to_syn_impl_item(true)
                }],
            },
        })]
    }
}
