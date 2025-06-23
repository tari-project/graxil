use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

fn bits_to_target(bits: u32) -> U256 {
    let exponent = ((bits >> 24) & 0xFF) as usize;
    let mantissa = bits & 0x00FFFFFF;
    
    if exponent == 0 {
        return U256::zero();
    }
    
    if mantissa & 0x00800000 != 0 {
        return U256::zero();
    }
    
    let mut bytes = [0u8; 32];
    
    if exponent <= 3 {
        let shift = 3 - exponent;
        let mantissa_shifted = mantissa >> (8 * shift);
        bytes[28..32].copy_from_slice(&mantissa_shifted.to_be_bytes());
    } else {
        if exponent <= 34 {
            let mantissa_bytes = mantissa.to_be_bytes();
            let pos = if exponent >= 32 { 0 } else { 32 - exponent };
            
            for i in 0..3 {
                if pos + i < 32 {
                    bytes[pos + i] = mantissa_bytes[i + 1];
                }
            }
        }
    }
    
    U256::from_big_endian(&bytes)
}

fn main() {
    let bits = 0x1d00d8df;
    let target = bits_to_target(bits);
    let target_bytes = target.to_big_endian();
    
    println!("nbits: 0x{:08x}", bits);
    println!("Target bytes: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}...", 
        target_bytes[0], target_bytes[1], target_bytes[2], target_bytes[3],
        target_bytes[4], target_bytes[5], target_bytes[6], target_bytes[7]);
    println!("Full target: {}", hex::encode(&target_bytes));
}
