use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;

use reqwest::header::{HeaderName, HeaderValue};
use serde::*;

use super::{Cookie, SecTrailPuppeteer};

impl<'de> Deserialize<'de> for SecTrailPuppeteer {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[doc(hidden)]
    enum Key {
      Success,
      Message,
      Cookies,
      Headers,
      Unique,
      Ignore,
    }
    #[doc(hidden)]
    struct KeyVisitor;

    impl<'de> de::Visitor<'de> for KeyVisitor {
      type Value = Key;
      fn expecting(&self, __formatter: &mut Formatter) -> fmt::Result {
        Formatter::write_str(__formatter, "field identifier")
      }
      fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        match value {
          0u64 => Ok(Key::Success),
          1u64 => Ok(Key::Message),
          2u64 => Ok(Key::Cookies),
          3u64 => Ok(Key::Headers),
          4u64 => Ok(Key::Unique),
          _ => Ok(Key::Ignore),
        }
      }
      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        match value {
          "success" => Ok(Key::Success),
          "message" => Ok(Key::Message),
          "cookies" => Ok(Key::Cookies),
          "headers" => Ok(Key::Headers),
          "unique" => Ok(Key::Unique),
          _ => Ok(Key::Ignore),
        }
      }
      fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        match value {
          b"success" => Ok(Key::Success),
          b"message" => Ok(Key::Message),
          b"cookies" => Ok(Key::Cookies),
          b"headers" => Ok(Key::Headers),
          b"unique" => Ok(Key::Unique),
          _ => Ok(Key::Ignore),
        }
      }
    }

    impl<'de> Deserialize<'de> for Key {
      #[inline]
      fn deserialize<D>(deserialize: D) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
      {
        Deserializer::deserialize_identifier(deserialize, KeyVisitor)
      }
    }
    #[doc(hidden)]
    struct Visitor<'de> {
      marker: PhantomData<SecTrailPuppeteer>,
      lifetime: PhantomData<&'de ()>,
    }

    impl<'de> de::Visitor<'de> for Visitor<'de> {
      type Value = SecTrailPuppeteer;
      fn expecting(&self, __formatter: &mut Formatter) -> fmt::Result {
        Formatter::write_str(__formatter, "struct SecTrailPuppeteer")
      }
      #[inline]
      fn visit_seq<A>(self, mut __seq: A) -> Result<Self::Value, A::Error>
      where
        A: de::SeqAccess<'de>,
      {
        let success = de::SeqAccess::next_element::<bool>(&mut __seq)?.unwrap_or_default();
        let message = de::SeqAccess::next_element::<String>(&mut __seq)?.unwrap_or_default();
        let cookies = de::SeqAccess::next_element::<Vec<Cookie>>(&mut __seq)?.unwrap_or_default();
        let headers = de::SeqAccess::next_element::<BTreeMap<String, String>>(&mut __seq)?
          .unwrap_or_default()
          .iter()
          .filter_map(header_map)
          .collect();
        let unique = de::SeqAccess::next_element::<String>(&mut __seq)?.unwrap_or_default();

        Ok(SecTrailPuppeteer {
          success,
          message,
          cookies,
          headers,
          unique,
        })
      }
      #[inline]
      fn visit_map<A>(self, mut __map: A) -> Result<Self::Value, A::Error>
      where
        A: de::MapAccess<'de>,
      {
        let mut success: Option<bool> = None;
        let mut message: Option<String> = None;
        let mut cookies: Option<Vec<Cookie>> = None;
        let mut headers: Option<BTreeMap<String, String>> = None;
        let mut unique: Option<String> = None;

        while let Some(key) = de::MapAccess::next_key::<Key>(&mut __map)? {
          match key {
            Key::Success => {
              if Option::is_some(&success) {
                return Err(<A::Error as de::Error>::duplicate_field("success"));
              }
              success = Some(de::MapAccess::next_value::<bool>(&mut __map)?);
            }
            Key::Message => {
              if Option::is_some(&message) {
                return Err(<A::Error as de::Error>::duplicate_field("message"));
              }
              message = Some(de::MapAccess::next_value::<String>(&mut __map)?);
            }
            Key::Cookies => {
              if Option::is_some(&cookies) {
                return Err(<A::Error as de::Error>::duplicate_field("cookies"));
              }
              cookies = Some(de::MapAccess::next_value::<Vec<Cookie>>(&mut __map)?);
            }
            Key::Headers => {
              if Option::is_some(&headers) {
                return Err(<A::Error as de::Error>::duplicate_field("headers"));
              }
              headers = Some(de::MapAccess::next_value::<BTreeMap<String, String>>(&mut __map)?);
            }
            Key::Unique => {
              if Option::is_some(&unique) {
                return Err(<A::Error as de::Error>::duplicate_field("unique"));
              }
              unique = Some(de::MapAccess::next_value::<String>(&mut __map)?);
            }
            _ => _ = de::MapAccess::next_value::<de::IgnoredAny>(&mut __map)?,
          }
        }

        let success = success.unwrap_or_default();
        let message = message.unwrap_or_default();
        let cookies = cookies.unwrap_or_default();
        let headers = headers.unwrap_or_default().iter().filter_map(header_map).collect();
        let unique = unique.unwrap_or_default();

        Ok(SecTrailPuppeteer {
          success,
          message,
          cookies,
          headers,
          unique,
        })
      }
    }
    #[doc(hidden)]
    const FIELDS: &[&str] = &["success", "message", "cookies", "headers", "unique"];
    Deserializer::deserialize_struct(
      deserializer,
      "SecTrailPuppeteer",
      FIELDS,
      Visitor {
        marker: PhantomData::<SecTrailPuppeteer>,
        lifetime: PhantomData,
      },
    )
  }
}

fn header_map(key_val: (impl AsRef<str>, impl AsRef<str>)) -> Option<(HeaderName, HeaderValue)> {
  let name: HeaderName = key_val.0.as_ref().parse().ok()?;
  let value: HeaderValue = key_val.1.as_ref().parse().ok()?;
  Some((name, value))
}
