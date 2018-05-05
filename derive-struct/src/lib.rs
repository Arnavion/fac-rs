//! A helper crate for easily deriving structs.

#![recursion_limit = "200"]

#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

/// Derives `serde::Deserialize` on the newtype.
#[proc_macro_derive(newtype_deserialize)]
pub fn derive_newtype_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();
	let struct_name = &ast.ident;

	let parser_func_name: syn::Ident = (match as_newtype(&ast) {
		Some(ty) => match identify_type(ty).unwrap_or_else(|| panic!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type")) {
			Type::SemverVersion => "parse_version",
			Type::SemverVersionReq => "parse_version_req",
			_ => panic!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type"),
		},

		None => panic!("#[derive(newtype_deserialize)] can only be used with tuple structs of one field"),
	}).into();

	let expecting_str = format!("a string that can be deserialized into a {}", struct_name);
	let error_str = format!("invalid {} {{:?}}: {{}}", struct_name);

	let result = quote! {
		impl<'de> ::serde::Deserialize<'de> for #struct_name {
			fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer<'de> {
				struct Visitor;

				impl<'de> ::serde::de::Visitor<'de> for Visitor {
					type Value = #struct_name;

					fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
						formatter.write_str(#expecting_str)
					}

					fn visit_str<E>(self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::de::Error {
						Ok(#struct_name(#parser_func_name(v).map_err(|err| ::serde::de::Error::custom(format!(#error_str, v, ::std::error::Error::description(&err))))?))
					}
				}

				deserializer.deserialize_any(Visitor)
			}
		}
	};

	result.into()
}

/// Derives `std::fmt::Display` on the newtype.
#[proc_macro_derive(newtype_display)]
pub fn derive_newtype_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).unwrap_or_else(|| panic!("#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type")) {
				Type::SemverVersion |
				Type::SemverVersionReq |
				Type::String |
				Type::U64 => quote! {
					impl ::std::fmt::Display for #struct_name {
						fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
							self.0.fmt(f)
						}
					}
				},
			}
		},

		None => panic!("#[derive(newtype_display)] can only be used with tuple structs of one field"),
	};

	result.into()
}

fn as_newtype(ast: &syn::DeriveInput) -> Option<&syn::Type> {
	match ast.data {
		syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }), .. }) if unnamed.len() == 1 => Some(&unnamed[0].ty),
		_ => None,
	}
}

enum Type {
	SemverVersion,
	SemverVersionReq,
	String,
	U64,
}

fn identify_type(ty: &syn::Type) -> Option<Type> {
	let path =
		if let syn::Type::Path(syn::TypePath { qself: None, ref path }) = *ty {
			path
		}
		else {
			return None;
		};

	match *path {
		syn::Path { leading_colon: Some(_), ref segments } if segments.len() == 2 => {
			let first_segment = &segments[0];
			let second_segment = &segments[1];

			if first_segment.arguments.is_empty() && first_segment.ident.as_ref() == "semver" && second_segment.arguments.is_empty() {
				match second_segment.ident.as_ref() {
					"Version" => Some(Type::SemverVersion),
					"VersionReq" => Some(Type::SemverVersionReq),
					_ => None,
				}
			}
			else {
				None
			}
		},

		syn::Path { leading_colon: None, ref segments } if segments.len() == 1 => {
			let segment = &segments[0];

			match segment.arguments {
				syn::PathArguments::None => match segment.ident.as_ref() {
					"String" => Some(Type::String),
					"u64" => Some(Type::U64),
					_ => None,
				},

				_ => None,
			}
		},

		_ => None,
	}
}
