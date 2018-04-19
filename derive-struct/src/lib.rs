//! A helper crate for easily deriving structs.

#![recursion_limit = "200"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

/// Generates getters for every field of a non-tuple struct.
#[proc_macro_derive(getters, attributes(getter))]
pub fn derive_getters(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();
	let struct_name = &ast.ident;

	let getters = match ast.data {
		syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(syn::FieldsNamed { named, .. }), .. }) => {
			named.into_iter().map(|syn::Field { ident, attrs, ty, .. }: syn::Field| {
				let mut field_doc_attr = None;
				let mut is_copy = false;
				for attr in &attrs {
					match attr.interpret_meta() {
						Some(syn::Meta::NameValue(syn::MetaNameValue { ident, .. })) if ident.as_ref() == "doc" => field_doc_attr = Some(attr),
						Some(syn::Meta::List(syn::MetaList { ident, ref nested, .. })) if ident.as_ref() == "getter" => {
							for nested_meta in nested {
								match *nested_meta {
									syn::NestedMeta::Meta(syn::Meta::Word(ref ident)) if ident.as_ref() == "copy" => is_copy = true,
									_ => panic!("Unrecognized meta item on field {}", ident),
								}
							}
						},
						_ => (),
					}
				}

				match identify_type(&ty) {
					Some(Type::Bool) => quote! {
						#field_doc_attr
						pub fn #ident(&self) -> bool {
							self.#ident
						}
					},

					Some(Type::Option { ty }) => quote! {
						#field_doc_attr
						pub fn #ident(&self) -> Option<&#ty> {
							self.#ident.as_ref()
						}
					},

					Some(Type::String) => quote! {
						#field_doc_attr
						pub fn #ident(&self) -> &str {
							&self.#ident
						}
					},

					Some(Type::Vec { ty }) => quote! {
						#field_doc_attr
						pub fn #ident(&self) -> &[#ty] {
							&self.#ident
						}
					},

					_ if is_copy => quote! {
						#field_doc_attr
						pub fn #ident(&self) -> #ty {
							self.#ident
						}
					},

					_ => quote! {
						#field_doc_attr
						pub fn #ident(&self) -> &#ty {
							&self.#ident
						}
					},
				}
			})
		},

		_ => panic!("#[derive(getters)] can only be used with non-tuple structs"),
	};

	let result = quote! {
		impl #struct_name {
			#(#getters)*
		}
	};

	result.into()
}

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
				_ => panic!("#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type"),
			}
		},

		None => panic!("#[derive(newtype_display)] can only be used with tuple structs of one field"),
	};

	result.into()
}

/// Derives `std::ops::Deref` and `std::ops::DerefMut` on the newtype.
#[proc_macro_derive(newtype_ref)]
pub fn derive_newtype_ref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	fn generate(struct_name: &syn::Ident, wrapped_type: quote::Tokens) -> quote::Tokens {
		quote! {
			impl ::std::ops::Deref for #struct_name {
				type Target = #wrapped_type;

				fn deref(&self) -> &Self::Target {
					&self.0
				}
			}

			impl ::std::ops::DerefMut for #struct_name {
				fn deref_mut(&mut self) -> &mut Self::Target {
					&mut self.0
				}
			}
		}
	}

	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).unwrap_or_else(|| panic!("#[derive(newtype_ref)] cannot be used with tuple structs with this wrapped type")) {
				Type::SemverVersion => generate(struct_name, quote!(::semver::Version)),
				Type::SemverVersionReq => generate(struct_name, quote!(::semver::VersionReq)),
				Type::String => {
					let deref = generate(struct_name, quote!(str));

					quote! {
						#deref

						impl<T: ?Sized> AsRef<T> for #struct_name where str: AsRef<T> {
							fn as_ref(&self) -> &T {
								(&self.0 as &str).as_ref()
							}
						}
					}
				},
				Type::U64 => generate(struct_name, quote!(u64)),
				Type::Vec { ty } => generate(struct_name, quote!([#ty])),
				_ => panic!("#[derive(newtype_ref)] cannot be used for tuple structs with this wrapped type"),
			}
		},

		None => panic!("#[derive(newtype_ref)] can only be used with tuple structs of one field"),
	};

	result.into()
}

fn as_newtype(ast: &syn::DeriveInput) -> Option<&syn::Type> {
	match ast.data {
		syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }), .. }) if unnamed.len() == 1 => Some(&unnamed[0].ty),
		_ => None,
	}
}

enum Type<'a> {
	Bool,
	Option { ty: &'a syn::Type },
	SemverVersion,
	SemverVersionReq,
	String,
	U64,
	Vec { ty: &'a syn::Type },
}

fn identify_type<'a>(ty: &'a syn::Type) -> Option<Type<'a>> {
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
					"bool" => Some(Type::Bool),
					"String" => Some(Type::String),
					"u64" => Some(Type::U64),
					_ => None,
				},

				syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { ref args, .. }) if args.len() == 1 => {
					if let syn::GenericArgument::Type(ref wrapped_ty) = args[0] {
						match segment.ident.as_ref() {
							"Option" => Some(Type::Option { ty: wrapped_ty }),
							"Vec" => Some(Type::Vec { ty: wrapped_ty }),
							_ => None,
						}
					}
					else {
						None
					}
				},

				_ => None,
			}
		},

		_ => None,
	}
}
