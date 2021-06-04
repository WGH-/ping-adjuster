use std::io::Write;

#[repr(C)]
#[derive(Debug)]
struct Timeval64 {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

#[repr(C)]
#[derive(Debug)]
struct Timeval32 {
    pub tv_sec: i32,
    pub tv_usec: i32,
}

trait TimevalLike {
    const IS_64: bool;
    fn get_sec(&self) -> i64;
    fn get_usec(&self) -> i64;
    fn set_sec(&mut self, val: i64);
    fn set_usec(&mut self, val: i64);
    fn bswap(&mut self);
    fn bswap_endianness(&mut self, endianness: Endianness) {
        let doit = if cfg!(target_endian = "big") {
            matches!(endianness, Endianness::Little)
        } else {
            matches!(endianness, Endianness::Big)
        };
        if doit {
            self.bswap()
        }
    }
}

impl TimevalLike for Timeval32 {
    const IS_64: bool = false;
    fn get_sec(&self) -> i64 {
        self.tv_sec.into()
    }
    fn get_usec(&self) -> i64 {
        self.tv_usec.into()
    }
    fn set_sec(&mut self, val: i64) {
        self.tv_sec = val as i32
    }
    fn set_usec(&mut self, val: i64) {
        self.tv_usec = val as i32
    }
    fn bswap(&mut self) {
        self.tv_sec = self.tv_sec.swap_bytes();
        self.tv_usec = self.tv_usec.swap_bytes();
    }
}

impl TimevalLike for Timeval64 {
    const IS_64: bool = true;
    fn get_sec(&self) -> i64 {
        self.tv_sec
    }
    fn get_usec(&self) -> i64 {
        self.tv_usec
    }
    fn set_sec(&mut self, val: i64) {
        self.tv_sec = val
    }
    fn set_usec(&mut self, val: i64) {
        self.tv_usec = val
    }
    fn bswap(&mut self) {
        self.tv_sec = self.tv_sec.swap_bytes();
        self.tv_usec = self.tv_usec.swap_bytes();
    }
}

#[derive(Clone, Copy, Debug)]
enum Endianness {
    Big,
    Little,
}

pub enum FuckupError {
    TooShort,
    NotATimeval,
}

fn try_fuckup<T: TimevalLike + std::fmt::Debug>(b: &mut [u8], endianness: Endianness) -> Result<(), FuckupError> {
    if b.len() <= std::mem::size_of::<T>() {
        return Err(FuckupError::TooShort);
    }
    let mut tv = unsafe { (b.as_ptr() as *const T).read_unaligned() };
    tv.bswap_endianness(endianness);
    if tv.get_usec() >= 0 && tv.get_usec() <= 999999 {
        eprintln!(" looks like a timestamp: endianness={:?} {:?}", endianness, tv);
        if T::IS_64 {
            tv.set_sec(tv.get_sec() - 133713371337);
        } else {
            tv.set_sec(tv.get_sec() - 1337);
        }
        tv.bswap_endianness(endianness);
        unsafe {
            std::ptr::write_unaligned(b.as_mut_ptr() as *mut T, tv);
        }
        return Ok(());
    } else {
        return Err(FuckupError::NotATimeval);
    };
}

pub fn fuckup_icmp_payload_buffer(b: &mut [u8]) -> Result<(), ()> {
    write!(std::io::stderr(), " icmp payload: ");
    hexdump(std::io::stderr(), &b).unwrap();
    write!(std::io::stderr(), "\n");

    // TODO maybe approx half RTT?
    //let now = nix::time::ClockId::CLOCK_REALTIME.now().expect("clock_gettime isn't supposed to fail");

    if try_fuckup::<Timeval64>(b, Endianness::Little).is_ok() {
        return Ok(());
    }
    // NOTE big endian 64 bit has false positives for 32 bit little endian, breaking it
    //if try_fuckup::<Timeval64>(b, Endianness::Big).is_ok() {
    //    return Ok(())
    //}
    if try_fuckup::<Timeval32>(b, Endianness::Little).is_ok() {
        return Ok(());
    }
    if try_fuckup::<Timeval32>(b, Endianness::Big).is_ok() {
        return Ok(());
    }
    eprintln!(" no timestamp found");
    Err(())
}

fn hexdump<W: Write>(mut w: W, b: &[u8]) -> std::io::Result<()> {
    for c in b {
        write!(w, "{:x}", c)?;
    }
    Ok(())
}
