//Simple datatype to produce BCL representations

#[derive(Default)]
pub struct BitWriter {
	bits: Vec<u8>,
	len: usize,
}

impl BitWriter {
	pub fn emit_bit(&mut self, bit: bool) {
		let pow = self.len % 8;

		if pow == 0 {
			self.bits.push(0);
		}

		self.len += 1;

		if bit {
			*self.bits.last_mut().unwrap() |= 1 << (7-pow);
		}
	}

	pub fn finish(self) -> Vec<u8> {
		self.bits
	}
}
