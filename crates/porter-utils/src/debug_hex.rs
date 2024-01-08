/// Utility trait used to print hex blocks to the console for easy debugging.
pub trait DebugHex {
    /// Prints the value to the console using pretty printed hex values.
    fn debug_hex(&self);
}

#[cfg(debug_assertions)]
impl DebugHex for &[u8] {
    fn debug_hex(&self) {
        let remainder = self.len() % 16;

        for chunk in self.chunks(16) {
            if chunk.len() == 16 {
                for byte in chunk.iter().take(16) {
                    print!("{:02X} ", byte);
                }
                println!()
            } else {
                for byte in chunk.iter() {
                    print!("{:02X} ", byte);
                }
            }
        }

        for _ in 0..remainder {
            print!("00 ");
        }

        if remainder > 0 {
            println!();
        }
    }
}

#[cfg(debug_assertions)]
impl DebugHex for Vec<u8> {
    fn debug_hex(&self) {
        (&self[0..]).debug_hex()
    }
}
