mod letters;

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

/// Returned when it was impossible to modify timestamp value
/// in the buffer.
pub enum AdjustError {
    /// Data was too short, `struct timeval` couldn't possibly
    /// fit in there.
    TooShort,
    /// Unable to autodetect `struct timeval` encoding, buffer
    /// likely doesn't contain a timestamp.
    NotATimeval,
}

/// An interface for various algorithms that increment
/// timestamp inside ping packets, confusing RTT calculation.
pub trait TimevalAdder {
    /// Calculate timestamp delta based on sequence number.
    ///
    /// Sequence number typically starts with 1 in most `ping`
    /// implementations.
    ///
    /// The increment is measured in whole seconds.
    fn get_increment(&mut self, seq: u16) -> i64;
}

/// Adds a constant value to timestamp, making RTT appear
/// this amount longer.
pub struct ConstantTimevalAdder(i64);

impl ConstantTimevalAdder {
    pub fn new(x: i64) -> Self {
        Self(x)
    }
}

impl TimevalAdder for ConstantTimevalAdder {
    fn get_increment(&mut self, _seq: u16) -> i64 {
        self.0
    }
}

/// Adds a delta to timestamp in such way it would
/// look like a 2D monochrome image.
///
/// ```text
/// time=700000000777041 ms
/// time=700777770077059 ms (DUP!)
/// time=700777770077042 ms (DUP!)
/// time=700000000777039 ms (DUP!)
/// time=700777770077061 ms (DUP!)
/// time=700777770077161 ms (DUP!)
/// time=700000000777042 ms (DUP!)
/// ```
pub struct BannerTimevalAdder {
    deltas: Vec<i64>,
}

impl BannerTimevalAdder {
    /// Creates a new `BannerTimevalAdder` displaying the specified
    /// message.
    pub fn new(msg: &str) -> Result<Self, letters::UnknownLetter> {
        Ok(Self {
            deltas: letters::get_word(msg)?,
        })
    }
}

impl TimevalAdder for BannerTimevalAdder {
    fn get_increment(&mut self, seq: u16) -> i64 {
        let i = seq as usize - 1;
        self.deltas[i % self.deltas.len()]
    }
}

fn try_adjust<T, F>(
    b: &mut [u8],
    endianness: Endianness,
    seq: u16,
    f: &mut F,
) -> Result<(), AdjustError>
where
    T: TimevalLike + std::fmt::Debug,
    F: TimevalAdder + ?Sized,
{
    if b.len() <= std::mem::size_of::<T>() {
        return Err(AdjustError::TooShort);
    }
    let mut tv = unsafe { (b.as_ptr() as *const T).read_unaligned() };
    tv.bswap_endianness(endianness);
    if tv.get_usec() >= 0 && tv.get_usec() <= 999999 {
        log::trace!(
            " looks like a timestamp: endianness={:?} {:?}",
            endianness,
            tv
        );
        if T::IS_64 {
            tv.set_sec(tv.get_sec() - f.get_increment(seq));
        } else {
            // We can't really do much for 32 bit systems,
            // since we lack the number of digits. At least
            // confuse ping to display 1337___ ms latency.
            tv.set_sec(tv.get_sec() - 1337);
        }
        tv.bswap_endianness(endianness);
        unsafe {
            std::ptr::write_unaligned(b.as_mut_ptr() as *mut T, tv);
        }
        return Ok(());
    } else {
        return Err(AdjustError::NotATimeval);
    };
}

/// Modify `ping` payload, trying to adjust timestamp value in it.
///
/// `f` specifies the algorithm that would modify the timestamp value.
/// See [`TimevalAdder`] for more details.
///
/// This function autodetects the specific encoding of a timestamp: 64-bit vs 32-bit,
/// big-endian vs little-endian. For 32-bit, `f` is not used, because there's too few
/// bits to draw something useful. A constant 1337 is added instead.
pub fn modify_icmp_payload<F>(b: &mut [u8], seq: u16, f: &mut F) -> Result<(), ()>
where
    F: TimevalAdder + ?Sized,
{
    log::trace!(" icmp payload: {:?}", U8DebugWrapper(b));

    // TODO maybe approx half RTT?
    //let now = nix::time::ClockId::CLOCK_REALTIME.now().expect("clock_gettime isn't supposed to fail");

    if try_adjust::<Timeval64, _>(b, Endianness::Little, seq, f).is_ok() {
        return Ok(());
    }
    // NOTE big endian 64 bit has false positives for 32 bit little endian, breaking it
    //if try_adjust::<Timeval64>(b, Endianness::Big).is_ok() {
    //    return Ok(())
    //}
    if try_adjust::<Timeval32, _>(b, Endianness::Little, seq, f).is_ok() {
        return Ok(());
    }
    if try_adjust::<Timeval32, _>(b, Endianness::Big, seq, f).is_ok() {
        return Ok(());
    }
    log::trace!(" no timestamp found");
    Err(())
}

struct U8DebugWrapper<'a>(&'a [u8]);

impl std::fmt::Debug for U8DebugWrapper<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in self.0 {
            write!(formatter, "{:x}", c)?;
        }
        Ok(())
    }
}
