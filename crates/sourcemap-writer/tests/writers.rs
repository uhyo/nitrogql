//! Writers share the handling of indentation. These tests check the output of each writer.

use nitrogql_ast::base::{Ident, Pos};
use sourcemap_writer::{JsStringWriter, JustWriter, SourceMapWriter, SourceWriter, SpanWriter};

/// Text written by `write_content`.
const TEXT: &str =
    "type FooResult = {\n  Foo: \"ä`${x}\\\";\n\n  nested: {\n    Bär: 1;\n    baz: 2;\n  }\n};\n";

fn ident(name: &str, line: usize, column: usize) -> Ident<'_> {
    Ident {
        name,
        position: Pos {
            line,
            column,
            start: 0,
            end: name.len(),
            file: 0,
            builtin: false,
        },
    }
}

/// Writes the same content with chunks that end in and contain newlines, empty lines,
/// non-ASCII text and mapped nodes at the start of indented lines.
fn write_content(writer: &mut impl SourceMapWriter) {
    let name = ident("Foo", 1, 2);
    let other = ident("Bär", 3, 4);
    writer.write("type ");
    writer.write_for("FooResult", &name);
    writer.write(" = {\n");
    writer.indent();
    writer.write_for("Foo", &name);
    writer.write(": \"ä`${x}\\\";\n\n");
    writer.write("nested: {\n");
    writer.indent();
    writer.write_for("Bär", &other);
    writer.write(": 1;\nbaz: 2;");
    writer.dedent();
    writer.write("\n}\n");
    writer.dedent();
    writer.write_for("", &name);
    writer.write("};\n");
}

#[test]
fn just_writer() {
    let mut buffer = String::new();
    write_content(&mut JustWriter::new(&mut buffer));
    assert_eq!(buffer, TEXT);
}

#[test]
fn js_string_writer() {
    let mut buffer = String::new();
    write_content(&mut JsStringWriter::new(&mut buffer));
    assert_eq!(
        buffer,
        r#"`
type FooResult = {
  Foo: "ä\`$\{x}\\";

  nested: {
    Bär: 1;
    baz: 2;
  }
};
`"#
    );
}

#[test]
fn span_writer() {
    let mut writer = SpanWriter::new();
    write_content(&mut writer);
    let (buffer, spans) = writer.into_parts();
    let spans = spans
        .iter()
        .map(|span| {
            format!(
                "{} {}",
                &buffer[span.generated_start..span.generated_start + span.generated_len],
                span.original_name
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(buffer, TEXT);
    assert_eq!(spans, vec!["FooResult Foo", "Foo Foo", "Bär Bär"]);
}

#[test]
fn source_writer() {
    let mut writer = SourceWriter::new();
    write_content(&mut writer);
    let buffers = writer.into_buffers();
    assert_eq!(buffers.buffer, TEXT);
    assert_eq!(
        buffers.source_map,
        ",KACEA,SAAG;EAAHA,GAAG;;;IAEDC,GAAG;;;AAFLD,AAAG"
    );
    assert_eq!(buffers.names, vec!["Foo", "Bär"]);
}
