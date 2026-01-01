use unicode_width::UnicodeWidthStr;

pub struct CursorBuffer {
    content: String,
    cursor_char_pos: usize,
}

impl CursorBuffer {
    pub fn new(content: String) -> Self {
        let cursor_char_pos = content.chars().count();
        Self {
            content,
            cursor_char_pos,
        }
    }

    pub fn empty() -> Self {
        Self {
            content: String::new(),
            cursor_char_pos: 0,
        }
    }

    pub fn cursor_byte_pos(&self) -> usize {
        self.content
            .char_indices()
            .nth(self.cursor_char_pos)
            .map_or(self.content.len(), |(i, _)| i)
    }

    #[cfg(test)]
    pub fn cursor_char_pos(&self) -> usize {
        self.cursor_char_pos
    }

    pub fn cursor_display_pos(&self) -> usize {
        self.content
            .chars()
            .take(self.cursor_char_pos)
            .collect::<String>()
            .width()
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn insert_char(&mut self, c: char) {
        let byte_pos = self.cursor_byte_pos();
        self.content.insert(byte_pos, c);
        self.cursor_char_pos += 1;
    }

    pub fn delete_char_before(&mut self) -> bool {
        if self.cursor_char_pos == 0 {
            return false;
        }
        let byte_pos = self.cursor_byte_pos();
        let prev_char_start = self.content[..byte_pos]
            .char_indices()
            .last()
            .map_or(0, |(i, _)| i);
        self.content.remove(prev_char_start);
        self.cursor_char_pos -= 1;
        true
    }

    pub fn move_left(&mut self) {
        if self.cursor_char_pos > 0 {
            self.cursor_char_pos -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let char_count = self.content.chars().count();
        if self.cursor_char_pos < char_count {
            self.cursor_char_pos += 1;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn into_content(self) -> String {
        self.content
    }
}

/// Calculates the (row, column) position of a cursor within word-wrapped text.
/// Used to position the terminal cursor correctly when editing wrapped content.
#[must_use]
pub fn cursor_position_in_wrap(
    text: &str,
    cursor_display_pos: usize,
    max_width: usize,
) -> (usize, usize) {
    if max_width == 0 {
        return (0, cursor_display_pos);
    }

    let mut row = 0;
    let mut line_width = 0;
    let mut total_width = 0;

    for word in text.split_inclusive(' ') {
        let word_width = word.width();
        let word_start = total_width;

        let (word_row, word_line_start) = if line_width + word_width <= max_width {
            let start_col = line_width;
            line_width += word_width;
            (row, start_col)
        } else if line_width == 0 {
            (row, 0)
        } else {
            row += 1;
            line_width = word_width;
            (row, 0)
        };

        let word_end = word_start + word_width;
        if cursor_display_pos >= word_start && cursor_display_pos < word_end {
            if line_width == 0 || word_width <= max_width || word_line_start > 0 {
                return (
                    word_row,
                    word_line_start + (cursor_display_pos - word_start),
                );
            } else {
                let mut char_row = word_row;
                let mut char_col = word_line_start;
                let mut char_pos = word_start;

                for ch in word.chars() {
                    let ch_width = ch.to_string().width();

                    if char_pos == cursor_display_pos {
                        return (char_row, char_col);
                    }

                    if char_col + ch_width > max_width && char_col > 0 {
                        char_row += 1;
                        char_col = 0;
                    }

                    char_col += ch_width;
                    char_pos += ch_width;
                }
            }
        }

        if word_width > max_width && word_line_start == 0 {
            let mut char_col = 0;
            for ch in word.chars() {
                let ch_width = ch.to_string().width();
                if char_col + ch_width > max_width && char_col > 0 {
                    row += 1;
                    char_col = 0;
                }
                char_col += ch_width;
            }
            line_width = char_col;
        }

        total_width = word_end;
    }

    (row, line_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_places_cursor_at_end() {
        let buf = CursorBuffer::new("hello".to_string());
        assert_eq!(buf.cursor_char_pos(), 5);
        assert_eq!(buf.content(), "hello");
    }

    #[test]
    fn test_empty_buffer() {
        let buf = CursorBuffer::empty();
        assert_eq!(buf.cursor_char_pos(), 0);
        assert!(buf.is_empty());
        assert_eq!(buf.content(), "");
    }

    #[test]
    fn test_insert_char_ascii() {
        let mut buf = CursorBuffer::empty();
        buf.insert_char('a');
        buf.insert_char('b');
        buf.insert_char('c');
        assert_eq!(buf.content(), "abc");
        assert_eq!(buf.cursor_char_pos(), 3);
    }

    #[test]
    fn test_insert_char_middle() {
        let mut buf = CursorBuffer::new("ac".to_string());
        buf.move_left(); // cursor after 'a'
        buf.insert_char('b');
        assert_eq!(buf.content(), "abc");
    }

    #[test]
    fn test_delete_char_before() {
        let mut buf = CursorBuffer::new("abc".to_string());
        assert!(buf.delete_char_before());
        assert_eq!(buf.content(), "ab");
        assert!(buf.delete_char_before());
        assert_eq!(buf.content(), "a");
        assert!(buf.delete_char_before());
        assert_eq!(buf.content(), "");
        assert!(!buf.delete_char_before()); // Returns false when empty
    }

    #[test]
    fn test_delete_at_start_returns_false() {
        let mut buf = CursorBuffer::new("hello".to_string());
        // Move cursor to start
        for _ in 0..5 {
            buf.move_left();
        }
        assert_eq!(buf.cursor_char_pos(), 0);
        assert!(!buf.delete_char_before());
        assert_eq!(buf.content(), "hello"); // Unchanged
    }

    #[test]
    fn test_move_left_right() {
        let mut buf = CursorBuffer::new("abc".to_string());
        assert_eq!(buf.cursor_char_pos(), 3);

        buf.move_left();
        assert_eq!(buf.cursor_char_pos(), 2);

        buf.move_left();
        buf.move_left();
        assert_eq!(buf.cursor_char_pos(), 0);

        buf.move_left(); // Should not go negative
        assert_eq!(buf.cursor_char_pos(), 0);

        buf.move_right();
        assert_eq!(buf.cursor_char_pos(), 1);
    }

    #[test]
    fn test_move_right_boundary() {
        let mut buf = CursorBuffer::new("ab".to_string());
        assert_eq!(buf.cursor_char_pos(), 2);

        buf.move_right(); // Already at end
        assert_eq!(buf.cursor_char_pos(), 2); // Should not exceed length
    }

    // UTF-8 multi-byte character tests

    #[test]
    fn test_multibyte_emoji() {
        let mut buf = CursorBuffer::new("a🎉b".to_string());
        // "a🎉b" has 3 characters but 6 bytes (a=1, 🎉=4, b=1)
        assert_eq!(buf.cursor_char_pos(), 3);

        buf.move_left(); // Before 'b'
        assert_eq!(buf.cursor_char_pos(), 2);

        buf.move_left(); // Before '🎉'
        assert_eq!(buf.cursor_char_pos(), 1);

        buf.move_left(); // Before 'a'
        assert_eq!(buf.cursor_char_pos(), 0);
    }

    #[test]
    fn test_insert_emoji() {
        let mut buf = CursorBuffer::empty();
        buf.insert_char('👍');
        buf.insert_char('🚀');
        assert_eq!(buf.content(), "👍🚀");
        assert_eq!(buf.cursor_char_pos(), 2);
    }

    #[test]
    fn test_delete_emoji() {
        let mut buf = CursorBuffer::new("a🎉b".to_string());
        buf.move_left(); // Before 'b'
        buf.delete_char_before(); // Delete '🎉'
        assert_eq!(buf.content(), "ab");
    }

    #[test]
    fn test_non_ascii_characters() {
        let mut buf = CursorBuffer::new("café".to_string());
        // 'é' is 2 bytes but 1 character
        assert_eq!(buf.cursor_char_pos(), 4);

        // Delete from end - removes 'é'
        buf.delete_char_before();
        assert_eq!(buf.content(), "caf");
        assert_eq!(buf.cursor_char_pos(), 3);
    }

    #[test]
    fn test_japanese_characters() {
        let mut buf = CursorBuffer::new("日本語".to_string());
        // Each Japanese character is 3 bytes
        assert_eq!(buf.cursor_char_pos(), 3);

        buf.move_left();
        assert_eq!(buf.cursor_char_pos(), 2);

        buf.insert_char('!');
        assert_eq!(buf.content(), "日本!語");
    }

    #[test]
    fn test_cursor_byte_pos_with_multibyte() {
        let buf = CursorBuffer::new("a🎉".to_string());
        // Cursor at end: byte position should be 5 (1 + 4)
        assert_eq!(buf.cursor_byte_pos(), 5);

        let mut buf2 = CursorBuffer::new("a🎉".to_string());
        buf2.move_left(); // Before '🎉'
        assert_eq!(buf2.cursor_byte_pos(), 1);

        buf2.move_left(); // Before 'a'
        assert_eq!(buf2.cursor_byte_pos(), 0);
    }

    #[test]
    fn test_into_content() {
        let buf = CursorBuffer::new("test".to_string());
        let content = buf.into_content();
        assert_eq!(content, "test");
    }
}
