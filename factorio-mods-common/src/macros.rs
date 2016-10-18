#[macro_export]
macro_rules! impl_deserialize_struct {
	(struct $struct_name:ident {
		$($fields:tt)*
	}) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				impl_deserialize_struct!(@enum_field enum Field {
				} $($fields)*);

				impl ::serde::Deserialize for Field {
					fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
						struct FieldVisitor;

						impl ::serde::de::Visitor for FieldVisitor {
							type Value = Field;

							fn visit_str<E>(&mut self, value: &str) -> ::std::result::Result<Field, E> where E: ::serde::Error {
								impl_deserialize_struct!(@match_field match value {
								} $($fields)*)
							}
						}

						deserializer.deserialize_struct_field(FieldVisitor)
					}
				}

				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_map<V>(&mut self, mut visitor: V) -> ::std::result::Result<Self::Value, V::Error> where V: ::serde::de::MapVisitor {
						impl_deserialize_struct!(@declare_values $($fields)*);

						while let Some(key) = try!(visitor.visit_key::<Field>()) {
							impl_deserialize_struct!(@match_value visitor match key {
							} $($fields)*);
						}

						try!(visitor.end());

						impl_deserialize_struct!(@check_value visitor APIError $($fields)*);

						impl_deserialize_struct!(@assign_value Ok($struct_name {
						}) $($fields)*)
					}
				}

				impl_deserialize_struct!(@fields_array const FIELDS: &'static [&'static str] = &[] $($fields)*);
				deserializer.deserialize_struct(stringify!($struct_name), FIELDS, Visitor)
			}
		}
	};

	(@enum_field enum Field {
		$($existing_members:ident,)*
	}) => {
		#[allow(non_camel_case_types)]
		enum Field {
			$($existing_members,)*
			Unknown,
		}
	};

	(@enum_field enum Field {
		$($existing_members:ident,)*
	} pub $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@enum_field enum Field {
			$($existing_members,)*
			$field_name,
		} $($fields)*);
	};

	(@enum_field enum Field {
		$($existing_members:ident,)*
	} $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@enum_field enum Field {
			$($existing_members,)*
			$field_name,
		} $($fields)*);
	};

	(@match_field match $value:ident {
		$($existing_case:pat => $existing_block:expr,)*
	}) => {
		match $value {
			$($existing_case => $existing_block,)*
			_ => Ok(Field::Unknown),
		}
	};

	(@match_field match $value:ident {
		$($existing_case:pat => $existing_block:expr,)*
	} pub type_name : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@match_field match $value {
			$($existing_case => $existing_block,)*
			"type" => ::std::result::Result::Ok::<Field, E>(Field::type_name),
		} $($fields)*);
	};

	(@match_field match $value:ident {
		$($existing_case:pat => $existing_block:expr,)*
	} service_token : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@match_field match $value {
			$($existing_case => $existing_block,)*
			"service-token" => ::std::result::Result::Ok::<Field, E>(Field::service_token),
		} $($fields)*);
	};

	(@match_field match $value:ident {
		$($existing_case:pat => $existing_block:expr,)*
	} service_username : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@match_field match $value {
			$($existing_case => $existing_block,)*
			"service-username" => ::std::result::Result::Ok::<Field, E>(Field::service_username),
		} $($fields)*);
	};

	(@match_field match $value:ident {
		$($existing_case:pat => $existing_block:expr,)*
	} pub $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@match_field match $value {
			$($existing_case => $existing_block,)*
			stringify!($field_name) => ::std::result::Result::Ok::<Field, E>(Field::$field_name),
		} $($fields)*);
	};

	(@match_field match $value:ident {
		$($existing_case:pat => $existing_block:expr,)*
	} $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@match_field match $value {
			$($existing_case => $existing_block,)*
			stringify!($field_name) => ::std::result::Result::Ok::<Field, E>(Field::$field_name),
		} $($fields)*);
	};

	(@declare_values pub $field_name:ident : Option<$field_type:ty>, $($fields:tt)*) => {
		let mut $field_name: Option<$field_type> = None;
		impl_deserialize_struct!(@declare_values $($fields)*)
	};

	(@declare_values pub $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		let mut $field_name: Option<$field_type> = None;
		impl_deserialize_struct!(@declare_values $($fields)*)
	};

	(@declare_values $field_name:ident : Option<$field_type:ty>, $($fields:tt)*) => {
		let mut $field_name: Option<$field_type> = None;
		impl_deserialize_struct!(@declare_values $($fields)*)
	};

	(@declare_values $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		let mut $field_name: Option<$field_type> = None;
		impl_deserialize_struct!(@declare_values $($fields)*)
	};

	(@declare_values) => { };

	(@match_value $visitor:ident match $key:ident {
		$($existing_case:pat => $existing_block:block,)*
	}) => {
		match $key {
			$($existing_case => $existing_block,)*

			Field::Unknown => {
				try!($visitor.visit_value::<::serde_json::Value>());
			},
		}
	};

	(@match_value $visitor:ident match $key:ident {
		$($existing_case:pat => $existing_block:block,)*
	} pub $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@match_value $visitor match $key {
			$($existing_case => $existing_block,)*

			Field::$field_name => {
				if $field_name.is_some() {
					return Err(<V::Error as ::serde::Error>::duplicate_field(stringify!($field_name)))
				}

				$field_name = try!($visitor.visit_value());
			},
		} $($fields)*);
	};

	(@match_value $visitor:ident match $key:ident {
		$($existing_case:pat => $existing_block:block,)*
	} $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@match_value $visitor match $key {
			$($existing_case => $existing_block,)*

			Field::$field_name => {
				if $field_name.is_some() {
					return Err(<V::Error as ::serde::Error>::duplicate_field(stringify!($field_name)))
				}

				$field_name = try!($visitor.visit_value());
			},
		} $($fields)*);
	};

	(@check_value $visitor:ident $APIError:ident pub $field_name:ident : Option<$field_type:ty>, $($fields:tt)*) => {
		impl_deserialize_struct!(@check_value $visitor $APIError $($fields)*)
	};

	(@check_value $visitor:ident $APIError:ident pub $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		let $field_name = match $field_name {
			Some(x) => x,
			None => try!($visitor.missing_field(stringify!($field_name)))
		};

		impl_deserialize_struct!(@check_value $visitor $APIError $($fields)*)
	};

	(@check_value $visitor:ident $APIError:ident $field_name:ident : Option<$field_type:ty>, $($fields:tt)*) => {
		impl_deserialize_struct!(@check_value $visitor $APIError $($fields)*)
	};

	(@check_value $visitor:ident $APIError:ident $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		let $field_name = match $field_name {
			Some(x) => x,
			None => try!($visitor.missing_field(stringify!($field_name)))
		};

		impl_deserialize_struct!(@check_value $visitor $APIError $($fields)*)
	};

	(@check_value $visitor:ident $APIError:ident) => { };

	(@assign_value Ok($struct_name:ident {
		$($existing_member:ident : $existing_value:expr,)*
	})) => {
		Ok($struct_name {
			$($existing_member: $existing_value,)*
		})
	};

	(@assign_value Ok($struct_name:ident {
		$($existing_member:ident : $existing_value:expr,)*
	}) pub $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@assign_value Ok($struct_name {
			$($existing_member: $existing_value,)*
			$field_name: $field_name,
		}) $($fields)*);
	};

	(@assign_value Ok($struct_name:ident {
		$($existing_member:ident : $existing_value:expr,)*
	}) $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@assign_value Ok($struct_name {
			$($existing_member: $existing_value,)*
			$field_name: $field_name,
		}) $($fields)*);
	};

	(@assign_value) => { };

	(@fields_array const $array_name:ident : &'static [&'static str] = &[
		$($existing_elements:expr,)*
	]) => {
		const $array_name: &'static [&'static str] = &[
			$($existing_elements,)*
		];
	};

	(@fields_array const $array_name:ident : &'static [&'static str] = &[
		$($existing_elements:expr,)*
	] pub $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@fields_array const $array_name: &'static [&'static str] = &[
			$($existing_elements,)*
			stringify!($field_name),
		] $($fields)*);
	};

	(@fields_array const $array_name:ident : &'static [&'static str] = &[
		$($existing_elements:expr,)*
	] $field_name:ident : $field_type:ty, $($fields:tt)*) => {
		impl_deserialize_struct!(@fields_array const $array_name: &'static [&'static str] = &[
			$($existing_elements,)*
			stringify!($field_name),
		] $($fields)*);
	};

	(@fields_array) => { };
}

#[macro_export]
macro_rules! impl_deserialize_u64 {
	($struct_name:ident) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_u64<E>(&mut self, v: u64) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
						Ok($struct_name(v))
					}
				}

				deserializer.deserialize_u64(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! impl_deserialize_string {
	($struct_name:ident) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_str<E>(&mut self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
						Ok($struct_name(v.to_string()))
					}
				}

				deserializer.deserialize_string(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! impl_deserialize_seq {
	($struct_name:ident, $wrapped_type:ty) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_seq<V>(&mut self, mut visitor: V) -> ::std::result::Result<Self::Value, V::Error> where V: ::serde::de::SeqVisitor {
						let mut result: Vec<$wrapped_type> = vec![];

						while let Some(value) = try!(visitor.visit()) {
							result.push(value);
						}

						try!(visitor.end());

						Ok($struct_name(result))
					}
				}

				deserializer.deserialize_string(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! impl_deserialize_string_or_seq_string {
	($struct_name:ident) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_str<E>(&mut self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
						Ok($struct_name(vec![v.to_string()]))
					}

					fn visit_seq<V>(&mut self, mut visitor: V) -> ::std::result::Result<Self::Value, V::Error> where V: ::serde::de::SeqVisitor {
						let mut result: Vec<String> = vec![];

						while let Some(value) = try!(visitor.visit()) {
							result.push(value);
						}

						try!(visitor.end());

						Ok($struct_name(result))
					}
				}

				deserializer.deserialize_string(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! impl_deserialize_semver_version {
	($struct_name:ident) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_str<E>(&mut self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
						let version = try!({
							match ::semver::Version::parse(v) {
								Ok(version) => Ok(version),
								Err(_) => {
									let fixed_version = fixup_version(v);

									::semver::Version::parse(&fixed_version).map_err(|err| ::serde::Error::invalid_value(::std::error::Error::description(&err)))
								}
							}
						});

						Ok($struct_name(version))
					}
				}

				deserializer.deserialize_u64(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! impl_deserialize_semver_versionreq {
	($struct_name:ident) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_str<E>(&mut self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
						let version = try!({
							match ::semver::VersionReq::parse(v) {
								Ok(version) => Ok(version),
								Err(_) => {
									let fixed_version = ::itertools::join(v.split(' ').map(fixup_version), " ");

									::semver::VersionReq::parse(&fixed_version).map_err(|err| ::serde::Error::invalid_value(::std::error::Error::description(&err)))
								}
							}
						});

						Ok($struct_name(version))
					}
				}

				deserializer.deserialize_u64(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! make_deserializable {
	(struct $struct_name:ident {
		$($fields:tt)*
	}) => {
		#[derive(Debug)]
		struct $struct_name {
			$($fields)*
		}

		impl_deserialize_struct!(struct $struct_name {
			$($fields)*
		});
	};

	(pub struct $struct_name:ident {
		$($fields:tt)*
	}) => {
		#[derive(Debug)]
		pub struct $struct_name {
			$($fields)*
		}

		impl_deserialize_struct!(struct $struct_name {
			$($fields)*
		});
	};

	(struct $struct_name:ident(u64)) => {
		#[derive(Debug)]
		struct $struct_name(u64);

		impl_deserialize_u64!($struct_name);
	};

	(pub struct $struct_name:ident(pub Vec<String>)) => {
		#[derive(Debug)]
		pub struct $struct_name(pub Vec<String>);

		impl_deserialize_string_or_seq_string!($struct_name);
	};

	(pub struct $struct_name:ident(pub Vec<$wrapped_type:ty>)) => {
		#[derive(Debug)]
		pub struct $struct_name(pub Vec<$wrapped_type>);

		impl_deserialize_seq!($struct_name, $wrapped_type);
	};
}

#[macro_export]
macro_rules! make_newtype_derefable {
	($struct_name:ty, $wrapped_type:ty) => {
		impl ::std::ops::Deref for $struct_name {
			type Target = $wrapped_type;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
	}
}

#[macro_export]
macro_rules! make_newtype_displayable {
	($struct_name:ty) => {
		impl ::std::fmt::Display for $struct_name {
			fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				self.0.fmt(f)
			}
		}
	}
}

#[macro_export]
macro_rules! make_newtype {
	(pub $struct_name:ident(String)) => {
		#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
		pub struct $struct_name(pub String);

		impl_deserialize_string!($struct_name);

		make_newtype_derefable!($struct_name, String);

		make_newtype_displayable!($struct_name);
	};

	(pub $struct_name:ident(u64)) => {
		#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
		pub struct $struct_name(pub u64);

		impl_deserialize_u64!($struct_name);

		make_newtype_derefable!($struct_name, u64);

		make_newtype_displayable!($struct_name);
	};

	(pub $struct_name:ident(::semver::Version)) => {
		#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
		pub struct $struct_name(pub ::semver::Version);

		impl_deserialize_semver_version!($struct_name);

		make_newtype_derefable!($struct_name, ::semver::Version);

		make_newtype_displayable!($struct_name);
	};

	(pub $struct_name:ident(::semver::VersionReq)) => {
		#[derive(Debug, PartialEq, Clone)]
		pub struct $struct_name(pub ::semver::VersionReq);

		impl_deserialize_semver_versionreq!($struct_name);

		make_newtype_derefable!($struct_name, ::semver::VersionReq);

		make_newtype_displayable!($struct_name);
	};

	($struct_name:ident(Vec<$wrapped_type:ty>)) => {
		#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
		struct $struct_name(Vec<$wrapped_type>);

		impl_deserialize_seq!($struct_name, $wrapped_type);
	};
}
