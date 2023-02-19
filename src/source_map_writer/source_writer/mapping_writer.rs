use crate::base64_vlq::base64_vlq;

pub struct MappingWriter {
    buffer: String,
    last_generated_line: usize,
    last_generated_column: usize,
    last_original_line: usize,
    last_original_column: usize,
    last_name_index: usize,
}

impl MappingWriter {
    pub fn new() -> Self {
        MappingWriter {
            buffer: String::new(),
            last_generated_line: 0,
            last_generated_column: 0,
            last_original_line: 0,
            last_original_column: 0,
            last_name_index: 0,
        }
    }

    pub fn into_buffer(self) -> String {
        self.buffer
    }

    pub fn add_entry(
        &mut self,
        generated_line: usize,
        generated_column: usize,
        original_line: usize,
        original_column: usize,
        name_index: Option<usize>,
    ) {
        let is_newline = self.last_generated_line != generated_line;
        self.buffer
            .push_str(&";".repeat(generated_line - self.last_generated_line));

        if is_newline {
            self.buffer.push_str(&base64_vlq(generated_column as isize));
        } else {
            self.buffer.push(',');
            let column_diff = (generated_column as isize) - (self.last_generated_column as isize);
            self.buffer.push_str(&base64_vlq(column_diff));
        }

        // currently, 'sources' is always 0
        self.buffer.push('A');

        let original_line_diff = (original_line as isize) - (self.last_original_line as isize);
        self.buffer.push_str(&base64_vlq(original_line_diff));
        // Seems like this calc is done in a inter-line manner
        let original_column_diff =
            (original_column as isize) - (self.last_original_column as isize);

        self.buffer.push_str(&base64_vlq(original_column_diff));

        if let Some(name_index) = name_index {
            let name_index_diff = (name_index as isize) - (self.last_name_index as isize);
            self.buffer.push_str(&base64_vlq(name_index_diff));
            self.last_name_index = name_index;
        }

        self.last_generated_line = generated_line;
        self.last_generated_column = generated_column;
        self.last_original_line = original_line;
        self.last_original_column = original_column;
    }
}
