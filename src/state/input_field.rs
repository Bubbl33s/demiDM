#[derive(Debug, Clone)]
pub struct InputField {
    content: String,
}

impl InputField {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.content.push(c);
    }

    pub fn pop_char(&mut self) {
        self.content.pop();
    }

    pub fn display_value(&self) -> &str {
        &self.content
    }

    pub fn masked_value(&self, mask: char) -> String {
        self.content.chars().map(|_| mask).collect()
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.content.clear();
    }
}

impl Default for InputField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_char() {
        let mut field = InputField::new();
        field.push_char('a');
        field.push_char('b');
        assert_eq!(field.display_value(), "ab");
    }

    #[test]
    fn test_pop_char() {
        let mut field = InputField::new();
        field.push_char('a');
        field.push_char('b');
        field.pop_char();
        assert_eq!(field.display_value(), "a");
    }

    #[test]
    fn test_display_value() {
        let mut field = InputField::new();
        field.push_char('t');
        field.push_char('e');
        field.push_char('s');
        field.push_char('t');
        assert_eq!(field.display_value(), "test");
    }

    #[test]
    fn test_masked_value() {
        let mut field = InputField::new();
        field.push_char('p');
        field.push_char('a');
        field.push_char('s');
        field.push_char('s');
        assert_eq!(field.masked_value('*'), "****");
    }

    #[test]
    fn test_clear() {
        let mut field = InputField::new();
        field.push_char('x');
        field.push_char('y');
        field.clear();
        assert_eq!(field.display_value(), "");
    }

    #[test]
    fn test_pop_empty() {
        let mut field = InputField::new();
        field.pop_char();
        assert_eq!(field.display_value(), "");
    }
}
