use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
pub struct GapBuffer {
	document: VecDeque<char>,
	typing_buf: VecDeque<char>,
	typing_buf_start: usize,
	typing_buf_end: usize,
}

// TODO: Think about whether or not you could move the cursor back if the typing_buf spanned a gap larger than 0.
// COULD YOU DO THAT?!?!?! HOW DO YOU KNOW?

impl GapBuffer {
	pub fn move_cursor_to(&mut self, i: usize) {
		self.move_cursor(i as isize - (self.typing_buf_start + self.typing_buf.len()) as isize);
	}

	/// This may copy shit unexpectedly- haven't gone through the logic at all.
	pub fn move_cursor(&mut self, n: isize) {
		for _ in 0..n.abs() {
			if n.is_negative() {
				if let Some(yes) = self.document.get(self.typing_buf_start) {
					self.typing_buf.push_front(*yes);
				}
				self.typing_buf_start = self.typing_buf_start.saturating_sub(1);
				if let Some(yes) = self.typing_buf.pop_back() {
					if self.typing_buf_end == 0 {
						self.document.push_front(yes);
					} else if !self.document.is_empty() {
						self.document[self.typing_buf_end] = yes;
					} else {
						// This might not ever happen if typing_buf_end gets modified correctly.
						self.document.push_back(yes);
					}
				}
				self.typing_buf_end = self.typing_buf_end.saturating_sub(1);
			} else {
				if let Some(yes) = self.document.get(self.typing_buf_end) {
					self.typing_buf.push_back(*yes);
				}
				self.typing_buf_end = (self
					.typing_buf_end
					.checked_add(1)
					.expect("Get a bigger pointer size, scrub."))
				.min(self.document.len());
				if let Some(yes) = self.typing_buf.pop_front() {
					if self.typing_buf_start == self.document.len() {
						self.document.push_back(yes);
					} else if !self.document.is_empty() {
						self.document[self.typing_buf_start] = yes;
					} else {
						self.document.push_front(yes);
					}
				}
				self.typing_buf_start =
					self.typing_buf_start.saturating_add(1).min(self.document.len());
			}
		}
	}

	pub fn insert_at_cursor(&mut self, s: String) {
		self.typing_buf.append(&mut s.chars().collect());
	}

	pub fn delete(&mut self, n: isize) {
		for _ in 0..n.abs() {
			if n.is_negative() {
				if self.typing_buf.pop_back().is_none() {
					self.typing_buf_start = self.typing_buf_start.saturating_sub(1);
				}
			} else {
				self.typing_buf_end += 1;
				if self.typing_buf_end == self.document.len() {
					return;
				}
			}
		}
		// self.cursor_pos = (self.cursor_pos as isize + n) as usize;
	}

	pub fn flush_typing_buf(&mut self) {
		self.document = self
			.document
			.range(0..self.typing_buf_start)
			.chain(self.typing_buf.iter())
			.chain(self.document.range(self.typing_buf_end..))
			.copied()
			.collect();
		self.typing_buf = VecDeque::new();
		self.typing_buf_start = self.typing_buf_end;
	}

	pub fn len(&self) -> usize {
		self.typing_buf.len() + self.typing_buf_start + self.document.len() - self.typing_buf_end
	}

	pub fn is_empty(&self) -> bool {
		self.document.is_empty() && self.typing_buf.is_empty()
	}

	fn make_into_string_by_consuming(self) -> String {
		self.document
			.range(0..self.typing_buf_start)
			.chain(self.typing_buf.iter())
			.chain(self.document.range(self.typing_buf_end..))
			.collect()
	}
}

impl From<String> for GapBuffer {
	fn from(s: String) -> Self {
		Self {
			document: s.chars().collect(),
			typing_buf: VecDeque::new(),
			typing_buf_start: s.len(),
			typing_buf_end: s.len(),
		}
	}
}

impl From<GapBuffer> for String {
	fn from(rs: GapBuffer) -> Self {
		rs.make_into_string_by_consuming()
	}
}

#[cfg(test)]
mod sweet {
	use super::*;
	#[test]
	fn from_string_and_len() {
		let rs = GapBuffer::from(String::from("Able doesn't pay me."));

		assert_eq!["Able doesn't pay me.", String::from(rs)];
	}
	#[test]
	fn delete_leftward() {
		let mut rs = GapBuffer::from(String::from("Able doesn't pay me."));

		rs.delete(-1);

		assert_eq![
			"Able doesn't pay me.".strip_suffix('.').unwrap(),
			String::from(rs)
		];
	}

	#[test]
	fn scroll() {
		let mut rs = GapBuffer::from(String::from("Able doesn't pay me."));

		rs.move_cursor_to(0);

		assert_eq!["Able doesn't pay me.", String::from(rs)];
	}

	#[test]
	fn scroll_beyond_tm() {
		let mut rs = GapBuffer::from(String::from("Able doesn't pay me."));

		rs.move_cursor_to(21);

		assert_eq!["Able doesn't pay me.", String::from(rs)];
	}

	#[test]
	fn scroll_and_delete() {
		let mut rs = GapBuffer::from(String::from("Able doesn't pay me."));

		rs.move_cursor_to(10);
		rs.delete(1);

		assert_eq!["Able doesnt pay me.", String::from(rs)];
	}

	#[test]
	fn scroll_and_put() {
		let mut rs = GapBuffer::from(String::from("Able doesn't pay me."));

		rs.move_cursor_to(10);
		rs.delete(1);
		rs.insert_at_cursor("'".into());
		rs.move_cursor_to(19);
		rs.insert_at_cursor("n".into());

		assert_eq!["Able doesn't pay men.", String::from(rs)];
	}
	#[test]
	fn type_and_flush() {
		let mut rs = GapBuffer::from(String::from("Able doesn't pay me."));

		rs.delete(-1);
		rs.insert_at_cursor(", and yet I work for him.".into());
		rs.flush_typing_buf();

		assert_eq![
			"Able doesn't pay me, and yet I work for him.",
			String::from(rs)
		];
	}
	
	#[test]
	fn scroll_past_start() {
		let mut rs = GapBuffer::from(String::from("Able doesn't pay me, and yet I work for him."));

		rs.move_cursor_to(0);
		rs.move_cursor(-1);

		assert_eq![
			"Able doesn't pay me, and yet I work for him.",
			String::from(rs)
		];
	}
}
