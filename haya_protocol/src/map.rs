use crate::Component;
use minecraft_data::map_decoration_type;
use mser::{ByteArray, Read, Write};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MapId(#[mser(varint)] pub u32);

#[derive(Clone, Serialize, Deserialize)]
pub struct MapDecoration {
    pub ty: map_decoration_type,
    pub x: u8,
    pub y: u8,
    pub rot: u8,
    pub name: Option<Component>,
}

#[derive(Clone)]
pub struct MapPatch<'a> {
    pub width: u8,
    pub height: u8,
    pub start_x: u8,
    pub start_y: u8,
    pub map_colors: ByteArray<'a>,
}

impl<'a> Read<'a> for MapPatch<'a> {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        let width = u8::read(buf)?;
        if width == 0 {
            Ok(Self {
                width: 0,
                height: 0,
                start_x: 0,
                start_y: 0,
                map_colors: ByteArray(&[]),
            })
        } else {
            Ok(Self {
                width,
                height: Read::read(buf)?,
                start_x: Read::read(buf)?,
                start_y: Read::read(buf)?,
                map_colors: Read::read(buf)?,
            })
        }
    }
}

impl<'a> Write for MapPatch<'a> {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe {
            if self.width == 0 {
                0u8.write(w);
            } else {
                self.width.write(w);
                self.height.write(w);
                self.start_x.write(w);
                self.start_y.write(w);
                self.map_colors.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        if self.width == 0 {
            0u8.len_s()
        } else {
            self.width.len_s()
                + self.height.len_s()
                + self.start_x.len_s()
                + self.start_y.len_s()
                + self.map_colors.len_s()
        }
    }
}
