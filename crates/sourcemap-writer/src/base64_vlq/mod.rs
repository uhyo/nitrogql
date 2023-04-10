const BASE64_CHARS: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '+', '/',
];

pub fn base64_vlq(input: isize) -> String {
    let sign_bit = input < 0;
    let mut value = input.unsigned_abs();

    if value < 16 {
        // one char exception (first character has 4 bit value space)
        let base64_code = (sign_bit as usize) | (value << 1);
        return BASE64_CHARS[base64_code].into();
    }

    let first_byte_code = (sign_bit as usize) | ((value & 0b1111) << 1) | 0b100000;
    let mut result: String = BASE64_CHARS[first_byte_code].into();
    value >>= 4;

    while value > 0 {
        let this_char_value = value & 0b11111;
        value >>= 5;
        let continuation_bit = if value > 0 { 0b100000 } else { 0 };
        let byte_code = continuation_bit | this_char_value;
        result.push(BASE64_CHARS[byte_code]);
    }
    result
}

#[cfg(test)]
mod test {
    use super::base64_vlq;

    #[test]
    fn base64() {
        assert_eq!(base64_vlq(0), "A");
        assert_eq!(base64_vlq(1), "C");
        assert_eq!(base64_vlq(2), "E");
        assert_eq!(base64_vlq(3), "G");
        assert_eq!(base64_vlq(4), "I");
        assert_eq!(base64_vlq(5), "K");
        assert_eq!(base64_vlq(6), "M");
        assert_eq!(base64_vlq(7), "O");
        assert_eq!(base64_vlq(8), "Q");
        assert_eq!(base64_vlq(9), "S");
        assert_eq!(base64_vlq(10), "U");
        assert_eq!(base64_vlq(11), "W");
        assert_eq!(base64_vlq(12), "Y");
        assert_eq!(base64_vlq(13), "a");
        assert_eq!(base64_vlq(14), "c");
        assert_eq!(base64_vlq(15), "e");
        assert_eq!(base64_vlq(16), "gB");
        assert_eq!(base64_vlq(17), "iB");

        assert_eq!(base64_vlq(175), "+K");

        assert_eq!(base64_vlq(-1), "D");
        assert_eq!(base64_vlq(-15), "f");
        assert_eq!(base64_vlq(-16), "hB");
    }
}
