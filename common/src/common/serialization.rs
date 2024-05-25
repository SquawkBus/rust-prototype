use std::collections::HashSet;
use std::io::{prelude::*, ErrorKind};
use std::io;
use std::hash::Hash;

use uuid::Uuid;

pub trait Serializable where Self: Sized {
    fn write<W: Write>(&self, writer: W) -> io::Result<()>;
    fn read<R: Read>(reader: R) -> io::Result<Self>;
}

impl Serializable for u8 {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let buf = [*self];
        writer.write(&buf)?;
    
        Ok(())            
    }

    fn read<R: Read>(mut reader: R) -> io::Result<u8> {
        let mut buf: [u8; 1] = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Serializable for bool {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        if *self {
            1u8.write(&mut writer)?;
        } else {
            1u8.write(&mut writer)?;
        }
    
        Ok(())            
    }

    fn read<R: Read>(mut reader: R) -> io::Result<bool> {
        match u8::read(&mut reader) {
            Ok(value) => if value == 1 { Ok(true) } else { Ok(false) },
            Err(error) => Err(error)
        }
    }

}

impl Serializable for u32 {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let buf: [u8; 4] = [
            *self as u8,
            (*self >> 8) as u8,
            (*self >> 16) as u8,
            (*self >> 24) as u8
        ];
        writer.write(&buf)?;
    
        Ok(())            
    }

    fn read<R: Read>(mut reader: R) -> io::Result<u32> {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        reader.read_exact(&mut buf)?;
        let value: u32 = buf[0] as u32 |
            (buf[1] as u32) << 8 |
            (buf[2] as u32) << 16 |
            (buf[3] as u32) << 24;
        Ok(value)
    }
}

impl Serializable for i32 {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (*self as u32).write(&mut writer)
    }

    fn read<R: Read>(mut reader: R) -> io::Result<i32> {
        match u32::read(&mut reader) {
            Ok(num) => Ok(num as i32),
            Err(error) => Err(error)
        }
    }
}

impl Serializable for String {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (self.len() as u32).write(&mut writer)?;
        writer.write(self.as_bytes())?;
        Ok(())
    }

    fn read<R: Read>(mut reader: R) -> io::Result<String> {
        let len = u32::read(&mut reader)?;
        let mut buf = vec![0u8; len as usize];
        reader.read(&mut buf)?;
        match String::from_utf8(buf) {
            Ok(value) => Ok(value),
            Err(error) => Err(io::Error::new(ErrorKind::Other, error)),
        }
    }
}

impl<T> Serializable for Vec<T> where T: Serializable {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (self.len() as u32).write(&mut writer)?;
        for value in self {
            value.write(&mut writer)?;
        }
        Ok(())
    }

    fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut len = u32::read(&mut reader)?;
        let mut values: Self = Vec::new();
        values.reserve_exact(len as usize);
        while len > 0 {
            let value = T::read(&mut reader)?;
            values.push(value);
            len = len - 1;
        }
        Ok(values)
    }
}

impl<T> Serializable for HashSet<T> where T: Serializable + Eq + Hash {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        (self.len() as u32).write(&mut writer)?;
        for value in self {
            value.write(&mut writer)?;
        }
        Ok(())
    }

    fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut len = u32::read(&mut reader)?;
        let mut values: Self = HashSet::new();
        while len > 0 {
            let value = T::read(&mut reader)?;
            values.insert(value);
            len = len - 1;
        }
        Ok(values)
    }
}

impl Serializable for Uuid {
    fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut buf = [0u8; 16];
        reader.read_exact(&mut buf)?;
        let value = Uuid::from_bytes(buf);
        Ok(value)
    }

    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write(self.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Seek};

    #[test]
    fn should_roundtrip_u32() {
        let mut buf = Cursor::new(Vec::new());

        let actual: u32 = 12345678;
        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match u32::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_pos_i32() {
        let mut buf = Cursor::new(Vec::new());

        let actual: i32 = 12345678;
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match i32::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_neg_i32() {
        let mut buf = Cursor::new(Vec::new());

        let actual: i32 = -12345678;
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match i32::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_string() {
        let mut buf = Cursor::new(Vec::new());

        let actual = String::from("Hello, World!");
        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match String::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_u32_vec() {
        let mut buf = Cursor::new(Vec::new());

        let actual: Vec<u32> = vec![1, 10, 100, 1000, 10000];
        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match Vec::<u32>::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_i32_vec() {
        let mut buf = Cursor::new(Vec::new());

        let actual: Vec<i32> = vec![-10000, -100, -10, -1, 0, 1, 10, 100, 1000, 10000];
        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match Vec::<i32>::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_i32_hashset() {
        let mut buf = Cursor::new(Vec::new());

        let mut actual: HashSet<i32> = HashSet::new();
        actual.insert(1);
        actual.insert(3);
        actual.insert(5);
        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match HashSet::<i32>::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }

    #[test]
    fn should_roundtrip_uuid() {
        let mut buf = Cursor::new(Vec::new());

        let actual = Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").expect("Should parse");
        actual.write(&mut buf).expect("should serialize");
        buf.seek(std::io::SeekFrom::Start(0)).unwrap();
        match Uuid::read(&mut buf) {
            Ok(expected) => assert_eq!(actual, expected),
            Err(error) => panic!("Failed to serialize: {:?}", error),
        }
    }
}
