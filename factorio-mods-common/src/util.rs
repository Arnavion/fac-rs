/// Deserializes a string or a sequence of strings into a vector of the target type.
pub fn deserialize_string_or_seq_string<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
	where T: ::serde::Deserialize<'de>, D: ::serde::Deserializer<'de> {

	struct Visitor<T>(::std::marker::PhantomData<T>);

	impl<'de, T> ::serde::de::Visitor<'de> for Visitor<T>
		where T: ::serde::Deserialize<'de> {

		type Value = Vec<T>;

		fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
			write!(f, "a string or sequence of strings")
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where E: ::serde::de::Error {

			let value = {
				// Try parsing as a newtype
				let deserializer = StringNewTypeStructDeserializer(v, ::std::marker::PhantomData);
				::serde::Deserialize::deserialize(deserializer)
			}.or_else(|_: E| {
				// Try parsing as a str
				let deserializer = ::serde::de::IntoDeserializer::into_deserializer(v);
				::serde::Deserialize::deserialize(deserializer)
			})?;
			Ok(vec![value])
		}

		fn visit_seq<A>(self, visitor: A) -> Result<Self::Value, A::Error>
			where A: ::serde::de::SeqAccess<'de> {

			::serde::Deserialize::deserialize(::serde::de::value::SeqAccessDeserializer::new(visitor))
		}
	}

	deserializer.deserialize_any(Visitor(::std::marker::PhantomData))
}

// Tries to deserialize the given string as a newtype
struct StringNewTypeStructDeserializer<'a, E>(&'a str, ::std::marker::PhantomData<E>);

impl<'de, 'a, E> ::serde::Deserializer<'de> for StringNewTypeStructDeserializer<'a, E> where E: ::serde::de::Error {
	type Error = E;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: ::serde::de::Visitor<'de> {
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: ::serde::de::Visitor<'de> {
		// Called by newtype visitor
		visitor.visit_str(self.0)
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str bytes
		byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
		struct enum identifier ignored_any
	}
}
