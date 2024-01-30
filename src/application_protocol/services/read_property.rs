use core::fmt::Debug;

use crate::{
    application_protocol::{
        confirmed::{ComplexAck, ComplexAckService, ConfirmedServiceChoice},
        primitives::data_value::ApplicationDataValue,
    },
    common::{
        error::Error,
        helper::{
            decode_context_object_id, decode_context_property_id, encode_closing_tag,
            encode_context_enumerated, encode_context_object_id, encode_context_unsigned,
            encode_opening_tag, get_tagged_body_for_tag,
        },
        io::{Reader, Writer},
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
    },
    network_protocol::data_link::DataLink,
};

#[derive(Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyValue<'a> {
    pub(crate) object_id: ObjectId,
    pub(crate) property_id: PropertyId,
    pub values: &'a [ApplicationDataValue<'a>],
    pub(crate) buf: &'a [u8],
}

impl<'a> TryFrom<ReadPropertyValue<'a>> for ApplicationDataValue<'a> {
    type Error = Error;

    fn try_from(property_value: ReadPropertyValue<'a>) -> Result<Self, Self::Error> {
        if let Some(value) = property_value.into_iter().next() {
            Ok(value?)
        } else {
            Err(Error::InvalidValue(
                "read property doesn't contain a single value"
            ))
        }
    }
}

// There is no need to log more fields than buf, since object_id and
// property_id are logged as part ReadPropertyAck.
impl<'a> Debug for ReadPropertyValue<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            f.write_str("ReadPropertyValue {\n")?;
            write!(f, "\tvalues: {:#?}\n", &self.values)?;
            write!(f, "\tbuf: {:?}\n", &self.buf)?;
            f.write_str("}\n")
        } else {
            f.debug_struct("ReadPropertyValue")
                .field("values", &self.values)
                .field("buf", &self.buf)
                .finish()
        }
    }
}

impl<'a> IntoIterator for &'_ ReadPropertyValue<'a> {
    type Item = Result<ApplicationDataValue<'a>, Error>;

    type IntoIter = ApplicationDataValueIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ApplicationDataValueIter {
            object_id: self.object_id,
            property_id: self.property_id,
            buf: self.buf,
            reader: Reader::new_with_len(self.buf.len()),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ApplicationDataValueIter<'a> {
    object_id: ObjectId,
    property_id: PropertyId,
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> Iterator for ApplicationDataValueIter<'a> {
    type Item = Result<ApplicationDataValue<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            None
        } else {
            match ApplicationDataValue::decode(
                &self.object_id,
                &self.property_id,
                &mut self.reader,
                self.buf
            ) {
                Ok(value) => Some(Ok(value)),
                Err(e) => Some(Err(e)),
            }
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyAck<'a> {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub property_value: ReadPropertyValue<'a>,
}

impl<'a> TryFrom<DataLink<'a>> for ReadPropertyAck<'a> {
    type Error = Error;

    fn try_from(value: DataLink<'a>) -> Result<Self, Self::Error> {
        let ack: ComplexAck = value.try_into()?;
        match ack.service {
            ComplexAckService::ReadProperty(ack) => Ok(ack),
            _ => Err(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadPropertyAck",
            )),
        }
    }
}

impl<'a> ReadPropertyAck<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        writer.push(ConfirmedServiceChoice::ReadProperty as u8);
        encode_context_object_id(writer, 0, &self.object_id);
        encode_context_enumerated(writer, 1, &self.property_id);
        encode_opening_tag(writer, 3);
        for value in self.property_value.values {
            value.encode(writer);
        }
        encode_closing_tag(writer, 3);
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let object_id =
            decode_context_object_id(reader, buf, 0, "ReadPropertyAck decode object_id")?;
        let property_id =
            decode_context_property_id(reader, buf, 1, "ReadPropertyAck decode property_id")?;

        let buf = get_tagged_body_for_tag(reader, buf, 3, "ReadPropertyAck decode data values")?;
        let property_value = ReadPropertyValue {
            object_id,
            property_id,
            values: &[],
            buf,
        };

        Ok(Self {
            object_id,
            property_id,
            property_value,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadProperty {
    pub object_id: ObjectId,     // e.g ObjectDevice:20088
    pub property_id: PropertyId, // e.g. PropObjectList
    pub array_index: u32,        // use BACNET_ARRAY_ALL for all
}

impl ReadProperty {
    pub fn new(object_id: ObjectId, property_id: PropertyId) -> Self {
        Self {
            object_id,
            property_id,
            array_index: BACNET_ARRAY_ALL,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        // object_id
        encode_context_object_id(writer, 0, &self.object_id);

        // property_id
        encode_context_enumerated(writer, 1, &self.property_id);

        // array_index
        if self.array_index != BACNET_ARRAY_ALL {
            encode_context_unsigned(writer, 2, self.array_index);
        }
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        // object_id
        let object_id = decode_context_object_id(reader, buf, 0, "ReadProperty decode object_id")?;

        // property_id
        let property_id =
            decode_context_property_id(reader, buf, 1, "ReadProperty decode property_id")?;

        Ok(Self {
            object_id,
            property_id,
            array_index: BACNET_ARRAY_ALL,
        })
    }
}
