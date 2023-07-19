fn pair(num: u16) -> (u8, u8) {
    let hi = ((num >> 8) & 0xff) as u8;
    let lo = (num & 0xff) as u8;
    (hi, lo)
}

pub fn push_u16(buf: &mut Vec<u8>, num: u16) {
    let (a, b) = pair(num);
    buf.push(a);
    buf.push(b);
}
