use crate::error::{Error, Result};
use concat_idents::concat_idents;
use serde::{Serialize, ser};

#[derive(PartialEq)]
enum SerializerMode {
    Normal,
    Key,
}

#[derive(PartialEq)]
pub enum DelimiterType {
    Newline,
    Whitespace,
    Colon,
}

pub struct PrettyConfig {
    pub delimiter: DelimiterType,
    pub indent_width: u8,
}

impl Default for PrettyConfig {
    fn default() -> PrettyConfig {
        PrettyConfig {
            delimiter: DelimiterType::Colon,
            indent_width: 0,
        }
    }
}

pub struct Serializer {
    pub output: String,
    mode: SerializerMode,
    pub pretty: PrettyConfig,
}

macro_rules! write_types {
    () => {
        type Ok = ();
        type Error = Error;

        type SerializeSeq = Self;
        type SerializeTuple = Self;
        type SerializeTupleStruct = Self;
        type SerializeTupleVariant = Self;
        type SerializeMap = Self;
        type SerializeStruct = Self;
        type SerializeStructVariant = Self;
    };
}

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    let mut serializer: Serializer = Serializer {
        output: String::new(),
        mode: SerializerMode::Normal,
        pretty: PrettyConfig::default(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub fn to_string_pretty<T: Serialize>(value: &T, pretty: PrettyConfig) -> Result<String> {
    let mut serializer: Serializer = Serializer {
        output: String::new(),
        mode: SerializerMode::Normal,
        pretty,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl ser::Serializer for &mut Serializer {
    write_types!();

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        let mut buffer: itoa::Buffer = itoa::Buffer::new();
        self.output += buffer.format(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        let mut buffer: itoa::Buffer = itoa::Buffer::new();
        self.output += buffer.format(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let mut buffer: ryu::Buffer = ryu::Buffer::new();
        self.output += buffer.format(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.output += format!("'{v}'").as_str();
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        if self.mode == SerializerMode::Key {
            self.output += v;
        } else {
            self.output += format!("\"{v}\"").as_str();
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq: &mut Serializer = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.output += "/";
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.output += format!("${}", variant).as_str();
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        self.output += format!("${}(", variant).as_str();
        value.serialize(&mut *self)?;
        self.output += ")";
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output += "[";
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        self.output += "(";
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output += format!("${}", variant).as_str();
        self.output += "(";
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output += "{";
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output += format!("${}{{", variant).as_str();
        Ok(self)
    }
}

macro_rules! delimiter {
    ($x: expr) => {
        match $x.pretty.delimiter {
            DelimiterType::Colon => ":",
            DelimiterType::Whitespace => " ",
            DelimiterType::Newline => "\n",
        }
    };
}

macro_rules! serialize_iterable {
    ($start: expr, $end: expr, $suffix: ident) => {
        type Ok = ();
        type Error = Error;

        concat_idents!(fn_name = serialize_, $suffix {
            fn fn_name<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
                if !self.output.ends_with($start) {
                    let mut char_data: &str = delimiter!(self);
                    if ($start == "(" || $start == "[") && char_data == "\n" {
                        char_data = " ";
                    }
                    self.output += char_data;
                }
                value.serialize(&mut **self)
            }
        });

        fn end(self) -> Result<()> {
            self.output += $end;
            Ok(())
        }
    };
}

impl ser::SerializeSeq for &mut Serializer {
    serialize_iterable!("[", "]", element);
}

impl ser::SerializeTuple for &mut Serializer {
    serialize_iterable!("(", ")", element);
}

impl ser::SerializeTupleStruct for &mut Serializer {
    serialize_iterable!("(", ")", field);
}

impl ser::SerializeTupleVariant for &mut Serializer {
    serialize_iterable!("(", ")", field);
}

macro_rules! serialize_key {
    ($x: expr, $key: expr) => {
        $x.mode = SerializerMode::Key;
        $key.serialize(&mut **$x)?;
        $x.mode = SerializerMode::Normal;
    };
}

macro_rules! indents {
    ($x: expr) => {
        if !$x.output.ends_with('{') || $x.pretty.delimiter == DelimiterType::Newline {
            $x.output += delimiter!($x);
        }
        if $x.pretty.delimiter == DelimiterType::Newline {
            let mut indent: String = String::new();
            for _ in 0..$x.pretty.indent_width {
                indent.push(' ');
            }
            $x.output += indent.as_str();
        }
    };
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        indents!(self);
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.output += "->";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.output += "}";
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        indents!(self);
        serialize_key!(self, key);
        self.output += "->";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += if self.pretty.delimiter == DelimiterType::Newline {
            "\n}"
        } else {
            "}"
        };
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += delimiter!(self);
        }
        serialize_key!(self, key);
        self.output += "->";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "}";
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    enum DistributionMode {
        Stable,
        Alpha,
        Nightly(u8, u8, u8),
        ReleaseCandidate { changelog: &'static str },
    }

    #[derive(Serialize)]
    struct Plugin {
        name: String,
        version: [u8; 3],
        api_compat: [u8; 3],
        distribution: DistributionMode,
        compat: HashMap<&'static str, [u8; 3]>,
    }

    #[test]
    fn serialize_simple_struct() {
        let data: Plugin = Plugin {
            name: "my-plugin".into(),
            version: [1, 0, 0],
            api_compat: [1, 0, 1],
            distribution: DistributionMode::Stable,
            compat: HashMap::new(),
        };
        println!("{}", to_string(&data).unwrap());
    }

    #[test]
    fn serialize_hashmap() {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("one", "value_one");
        println!("{}", to_string(&map).unwrap());
    }

    #[test]
    fn custom_pretty() {
        let data: Plugin = Plugin {
            name: "my-plugin".into(),
            version: [2, 2, 1],
            api_compat: [2, 0, 0],
            distribution: DistributionMode::Alpha,
            compat: [].into(),
        };
        println!(
            "{}",
            to_string_pretty(
                &data,
                PrettyConfig {
                    delimiter: DelimiterType::Newline,
                    indent_width: 4,
                }
            )
            .unwrap()
        );
    }

    #[test]
    fn tuple_variant() {
        let data: Plugin = Plugin {
            name: "my-plugin".into(),
            version: [1, 0, 0],
            api_compat: [2, 0, 0],
            distribution: DistributionMode::Nightly(12, 13, 25),
            compat: [].into(),
        };
        println!("{}", to_string(&data).unwrap());
    }

    #[test]
    fn whitespace_delimiter() {
        let tuple: (u8, bool, &str) = (1, true, "test");
        println!(
            "{}",
            to_string_pretty(
                &tuple,
                PrettyConfig {
                    delimiter: DelimiterType::Whitespace,
                    ..Default::default()
                }
            )
            .unwrap()
        );
    }

    #[test]
    fn none() {
        let none: Option<()> = None;
        println!("{}", to_string(&none).unwrap());
    }

    #[test]
    fn struct_variant() {
        let distribution: DistributionMode = DistributionMode::ReleaseCandidate {
            changelog: "Added \"switch\" subcommand",
        };
        println!("{}", to_string(&distribution).unwrap());
    }
}
