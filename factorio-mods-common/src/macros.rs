#[macro_export]
macro_rules! make_struct {
	(pub struct $struct_name:ident {
		$($fields:tt)*
	}) => {
		#[derive(Clone, Debug, Deserialize, new)]
		pub struct $struct_name {
			$($fields)*
		}

		impl $struct_name {
			impl_struct_getters!($($fields)*);
		}
	};

	(struct $struct_name:ident {
		$($fields:tt)*
	}) => {
		#[derive(Clone, Debug, Deserialize)]
		struct $struct_name {
			$($fields)*
		}
	};

	(pub $struct_name:ident(String)) => {
		#[derive(Clone, Debug, Deserialize, new, PartialEq, Eq, Hash, PartialOrd, Ord)]
		pub struct $struct_name(String);

		impl_newtype_deref!($struct_name, str);

		impl_newtype_display!($struct_name);
	};

	(pub $struct_name:ident(u64)) => {
		#[derive(Clone, Debug, Deserialize, new, PartialEq, Eq, Hash, PartialOrd, Ord)]
		pub struct $struct_name(u64);

		impl_newtype_deref!($struct_name, u64);

		impl_newtype_display!($struct_name);
	};

	(struct $struct_name:ident(u64)) => {
		#[derive(Clone, Debug, Deserialize)]
		struct $struct_name(u64);
	};

	(pub struct $struct_name:ident(Vec<String>)) => {
		#[derive(Clone, Debug, new)]
		pub struct $struct_name(Vec<String>);

		impl_newtype_deserialize_string_or_seq_string!($struct_name);

		impl_newtype_deref!($struct_name, [String]);
	};

	(pub struct $struct_name:ident(Vec<$wrapped_type:ty>)) => {
		#[derive(Clone, Debug, Deserialize, new)]
		pub struct $struct_name(Vec<$wrapped_type>);

		impl_newtype_deref!($struct_name, [$wrapped_type]);
	};

	($struct_name:ident(Vec<$wrapped_type:ty>)) => {
		#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
		struct $struct_name(Vec<$wrapped_type>);
	};

	(pub $struct_name:ident(::semver::Version)) => {
		#[derive(Clone, Debug, new, PartialEq, Eq, Hash, PartialOrd, Ord)]
		pub struct $struct_name(::semver::Version);

		impl_newtype_deserialize_semver_version!($struct_name);

		impl_newtype_deref!($struct_name, ::semver::Version);

		impl_newtype_display!($struct_name);
	};

	(pub $struct_name:ident(::semver::VersionReq)) => {
		#[derive(Clone, Debug, new, PartialEq)]
		pub struct $struct_name(::semver::VersionReq);

		impl_newtype_deserialize_semver_versionreq!($struct_name);

		impl_newtype_deref!($struct_name, ::semver::VersionReq);

		impl_newtype_display!($struct_name);
	};
}

#[macro_export]
macro_rules! impl_struct_getters {
	() => { };

	(#[serde(rename(deserialize = $str:expr))] $($fields:tt)*) => {
		impl_struct_getters!($($fields)*);
	};

	($field_name:ident : String, $($fields:tt)*) => {
		pub fn $field_name(&self) -> &str {
			&self.$field_name
		}

		impl_struct_getters!($($fields)*);
	};

	($field_name:ident : Vec<String>, $($fields:tt)*) => {
		pub fn $field_name(&self) -> &[&str] {
			&self.$field_name
		}

		impl_struct_getters!($($fields)*);
	};

	($field_name:ident : Vec<$wrapped_type:ty>, $($fields:tt)*) => {
		pub fn $field_name(&self) -> &[$wrapped_type] {
			&self.$field_name
		}

		impl_struct_getters!($($fields)*);
	};

	($field_name:ident : $field_type:ty, $($fields:tt)*) => {
		pub fn $field_name(&self) -> &$field_type {
			&self.$field_name
		}

		impl_struct_getters!($($fields)*);
	};
}

#[macro_export]
macro_rules! impl_newtype_deref {
	($struct_name:ty, $wrapped_type:ty) => {
		impl ::std::ops::Deref for $struct_name {
			type Target = $wrapped_type;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl ::std::ops::DerefMut for $struct_name {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}
	}
}

#[macro_export]
macro_rules! impl_newtype_display {
	($struct_name:ty) => {
		impl ::std::fmt::Display for $struct_name {
			fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				self.0.fmt(f)
			}
		}
	}
}

#[macro_export]
macro_rules! impl_newtype_deserialize_semver_version {
	($struct_name:ident) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_str<E>(&mut self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
						let version =
							if let Ok(version) = ::semver::Version::parse(v) {
								version
							}
							else  {
								let fixed_version = fixup_version(v);
								::semver::Version::parse(&fixed_version).map_err(|err| ::serde::Error::invalid_value(::std::error::Error::description(&err)))?
							};

						Ok($struct_name(version))
					}
				}

				deserializer.deserialize_u64(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! impl_newtype_deserialize_semver_versionreq {
	($struct_name:ident) => {
		impl ::serde::Deserialize for $struct_name {
			fn deserialize<D>(deserializer: &mut D) -> ::std::result::Result<Self, D::Error> where D: ::serde::Deserializer {
				struct Visitor;

				impl ::serde::de::Visitor for Visitor {
					type Value = $struct_name;

					fn visit_str<E>(&mut self, v: &str) -> ::std::result::Result<Self::Value, E> where E: ::serde::Error {
						let version =
							if let Ok(version) = ::semver::VersionReq::parse(v) {
								version
							}
							else {
								let fixed_version = ::itertools::join(v.split(' ').map(fixup_version), " ");
								::semver::VersionReq::parse(&fixed_version).map_err(|err| ::serde::Error::invalid_value(::std::error::Error::description(&err)))?
							};

						Ok($struct_name(version))
					}
				}

				deserializer.deserialize_u64(Visitor)
			}
		}
	};
}

#[macro_export]
macro_rules! impl_newtype_deserialize_string_or_seq_string {
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

						while let Some(value) = visitor.visit()? {
							result.push(value);
						}

						visitor.end()?;

						Ok($struct_name(result))
					}
				}

				deserializer.deserialize_string(Visitor)
			}
		}
	};
}
