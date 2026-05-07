use crate::Utf8;
use mser::V32;

#[derive(Clone, Serialize, Deserialize)]
pub struct Intention<'a> {
    pub protocol_version: V32,
    pub host_name: Utf8<'a>, // bungeecord
    pub port: u16,
    pub intention: ClientIntent,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ClientIntent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

impl mser::Write for ClientIntent {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe {
            w.write_byte(*self as u8);
        }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl<'a> mser::Read<'a> for ClientIntent {
    fn read(buf: &mut mser::Reader) -> Result<Self, mser::Error> {
        match mser::V21::read(buf)?.0 {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(mser::Error),
        }
    }
}
