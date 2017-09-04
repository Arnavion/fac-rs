//! A helper crate for easily deriving structs.

#![recursion_limit = "200"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

/// Generates getters for every field of a non-tuple struct.
#[proc_macro_derive(getters, attributes(getter))]
pub fn derive_getters(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let source = input.to_string();
	let ast = syn::parse_derive_input(&source).unwrap();
	let struct_name = &ast.ident;

	let getters = match ast.body {
		syn::Body::Struct(syn::VariantData::Struct(fields)) => {
			fields.into_iter().map(|field| {
				let syn::Field { ident, attrs, ty, .. } = field;

				let mut field_doc_attr = None;
				let mut is_copy = false;
				for attr in &attrs {
					match attr.value {
						syn::MetaItem::NameValue(ref ident, _) if ident.as_ref() == "doc" => field_doc_attr = Some(attr),
						syn::MetaItem::List(ref ident, ref nested_meta_items) if ident.as_ref() == "getter" => {
							for nested_meta_item in nested_meta_items {
								match *nested_meta_item {
									syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref ident)) if ident.as_ref() == "copy" => is_copy = true,
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

	result.parse().unwrap()
}

/// Derives `serde::Deserialize` on the newtype.
#[proc_macro_derive(newtype_deserialize)]
pub fn derive_newtype_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	fn generate_semver(struct_name: &syn::Ident, parser_func_name: quote::Tokens) -> quote::Tokens {
		let expecting_str = format!("a string that can be deserialized into a {}", struct_name);
		let error_str = format!("invalid {} {{:?}}: {{}}", struct_name);

		quote! {
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
		}
	}

	let source = input.to_string();
	let ast = syn::parse_derive_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).unwrap_or_else(|| panic!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type: {:?}", ty)) {
				Type::SemverVersion => generate_semver(struct_name, quote!(parse_version)),

				Type::SemverVersionReq => generate_semver(struct_name, quote!(parse_version_req)),

				_ => panic!("#[derive(newtype_deserialize)] cannot be used with tuple structs with this wrapped type: {:?}", ty),
			}
		},

		None => panic!("#[derive(newtype_deserialize)] can only be used with tuple structs of one field"),
	};

	result.parse().unwrap()
}

/// Derives `std::fmt::Display` on the newtype.
#[proc_macro_derive(newtype_display)]
pub fn derive_newtype_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let source = input.to_string();
	let ast = syn::parse_derive_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).unwrap_or_else(|| panic!("#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type: {:?}", ty)) {
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
				_ => panic!("#[derive(newtype_display)] cannot be used with tuple structs with this wrapped type: {:?}", ty),
			}
		},

		None => panic!("#[derive(newtype_display)] can only be used with tuple structs of one field"),
	};

	result.parse().unwrap()
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

	let source = input.to_string();
	let ast = syn::parse_derive_input(&source).unwrap();

	let result = match as_newtype(&ast) {
		Some(ty) => {
			let struct_name = &ast.ident;

			match identify_type(ty).unwrap_or_else(|| panic!("#[derive(newtype_ref)] cannot be used with tuple structs with this wrapped type: {:?}", ty)) {
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
				_ => panic!("#[derive(newtype_ref)] cannot be used for tuple structs with this wrapped type: {:?}", ty),
			}
		},

		None => panic!("#[derive(newtype_ref)] can only be used with tuple structs of one field"),
	};

	result.parse().unwrap()
}

fn as_newtype(ast: &syn::MacroInput) -> Option<&syn::Ty> {
	match ast.body {
		syn::Body::Struct(syn::VariantData::Tuple(ref fields)) if fields.len() == 1 => Some(&fields[0].ty),
		_ => None,
	}
}

#[derive(Debug)]
enum Type<'a> {
	Bool,
	Option { ty: &'a syn::Ty },
	SemverVersion,
	SemverVersionReq,
	String,
	U64,
	Vec { ty: &'a syn::Ty },
}

fn identify_type<'a>(ty: &'a syn::Ty) -> Option<Type<'a>> {
	let path =
		if let syn::Ty::Path(None, ref path) = *ty {
			path
		}
		else {
			return None;
		};

	match *path {
		syn::Path { global: true, ref segments } if segments.len() == 2 => {
			let first_segment = &segments[0];
			let second_segment = &segments[1];

			if first_segment.parameters.is_empty() && first_segment.ident.as_ref() == "semver" && second_segment.parameters.is_empty() {
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

		syn::Path { global: false, ref segments } if segments.len() == 1 => {
			let segment = &segments[0];

			match segment.parameters {
				ref parameters if parameters.is_empty() => match segment.ident.as_ref() {
					"bool" => Some(Type::Bool),
					"String" => Some(Type::String),
					"u64" => Some(Type::U64),
					_ => None,
				},

				syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData {
					ref lifetimes, ref types, ref bindings,
				}) if lifetimes.is_empty() && types.len() == 1 && bindings.is_empty() => {
					let wrapped_ty = &types[0];

					match segment.ident.as_ref() {
						"Option" => Some(Type::Option { ty: wrapped_ty }),
						"Vec" => Some(Type::Vec { ty: wrapped_ty }),
						_ => None,
					}
				},

				_ => None,
			}
		},

		_ => None,
	}
}
