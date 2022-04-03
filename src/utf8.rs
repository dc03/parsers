enum UTF8HeadType {
    Single = 0,
    Double = 0b110,
    Triple = 0b1110,
    Quad = 0b11110,
    Unknown,
}

enum UTF8DataMask {
    SingleData = 0b01111111,
    DoubleData = 0b00011111,
    TripleData = 0b00001111,
    QuadData = 0b00000111,
    TailData = 0b00111111,
}

enum _UTF8HeaderMask {
    SingleMask = 0b00000000,
    DoubleMask = 0b11000000,
    TripleMask = 0b11100000,
    QuadMask = 0b11110000,
    TailMask = 0b10000000,
}

macro_rules! u8_ {
    ($v:expr) => {
        ($v as u8)
    };
}

fn get_head_type(head: u8) -> UTF8HeadType {
    if head >> 7 == u8_!(UTF8HeadType::Single) {
        UTF8HeadType::Single
    } else if head >> 5 == u8_!(UTF8HeadType::Double) {
        UTF8HeadType::Double
    } else if head >> 4 == u8_!(UTF8HeadType::Triple) {
        UTF8HeadType::Triple
    } else if head >> 3 == u8_!(UTF8HeadType::Quad) {
        UTF8HeadType::Quad
    } else {
        UTF8HeadType::Unknown
    }
}

fn get_head_data(head: u8) -> Option<u8> {
    match get_head_type(head) {
        UTF8HeadType::Single => Some(head & u8_!(UTF8DataMask::SingleData)),
        UTF8HeadType::Double => Some(head & u8_!(UTF8DataMask::DoubleData)),
        UTF8HeadType::Triple => Some(head & u8_!(UTF8DataMask::TripleData)),
        UTF8HeadType::Quad => Some(head & u8_!(UTF8DataMask::QuadData)),
        UTF8HeadType::Unknown => None,
    }
}

fn tail_data(tail: u32) -> u32 {
    (tail as u8 & u8_!(UTF8DataMask::TailData)) as u32
}

fn tail_masked(tail: u32, pos: u8) -> u32 {
    tail_data(tail) << (6 * (pos - 1))
}

#[allow(dead_code)]
pub(crate) fn next_codepoint<T, U>(file: &mut T, get_next: fn(&mut T) -> u8) -> Option<char>
where
    T: Iterator<Item = U>,
{
    let head = get_next(file);
    next_codepoint_head(file, head, get_next)
}

pub(crate) fn next_codepoint_head<T, U>(
    file: &mut T,
    head: u8,
    get_next: fn(&mut T) -> u8,
) -> Option<char>
where
    T: Iterator<Item = U>,
{
    if head >> 7 == u8_!(UTF8HeadType::Single) {
        Some(head.into())
    } else if head >> 5 == u8_!(UTF8HeadType::Double) {
        let tail = get_next(file) as u32;

        char::from_u32((get_head_data(head)? << 6) as u32 | tail_masked(tail, 1))
    } else if head >> 4 == u8_!(UTF8HeadType::Triple) {
        let tail1 = get_next(file) as u32;
        let tail2 = get_next(file) as u32;

        char::from_u32(
            ((get_head_data(head)? as u32) << 12) | tail_masked(tail1, 2) | tail_masked(tail2, 1),
        )
    } else if head >> 3 == u8_!(UTF8HeadType::Quad) {
        let tail1 = get_next(file) as u32;
        let tail2 = get_next(file) as u32;
        let tail3 = get_next(file) as u32;

        char::from_u32(
            ((get_head_data(head)? as u32) << 12)
                | tail_masked(tail1, 3)
                | tail_masked(tail2, 2)
                | tail_masked(tail3, 1),
        )
    } else {
        None
    }
}
